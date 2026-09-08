use quote::format_ident;
use syn::spanned::Spanned as _;

use super::{unwrap_generic, CommandParameter, Invocation};

enum Modifier {
    Lazy,
    Flag,
    Rest,
    String,
}

struct PrefixParameter<'a> {
    idx: usize,
    modifier: Option<Modifier>,
    ty: syn::Type,
    name: &'a str,
}

fn get_modifier(param: &CommandParameter) -> Result<Option<Modifier>, syn::Error> {
    match (param.args.lazy, param.args.rest, param.args.flag, param.args.string) {
        (false, false, false, false) => Ok(None),
        (true, false, false, false) => Ok(Some(Modifier::Lazy)),
        (false, true, false, false) => Ok(Some(Modifier::Rest)),
        (false, false, true, false) => {
            if param.type_ == syn::parse_quote! { bool } {
                Ok(Some(Modifier::Flag))
            } else {
                Err(syn::Error::new(param.type_.span(), "Must use bool for flags"))
            }
        },
        (false, false, false, true) => Ok(Some(Modifier::String)),
        _ => Err(syn::Error::new(
            param.span,
            "modifiers like #[lazy] or #[rest] cannot be used together",
        )),
    }
}

fn parse_prefix_params(
    params: &[PrefixParameter],
) -> Result<(proc_macro2::TokenStream, Vec<syn::Type>), syn::Error> {
    let Some((first, rest)) = params.split_first() else {
        // If the input is exhausted, we output success.
        return Ok((
            quote::quote! {
                if args.is_empty() {
                    return Ok(());
                }
                Err((None, None))
            },
            vec![],
        ));
    };

    // Recursively parse the parameters after this one, and wrap the generated code in a function to
    // prevent duplicated copy-pasting.
    let (parsed_rest, mut parsed_types) = parse_prefix_params(rest)?;
    let parser_name = format_ident!("parser_{}", first.idx);
    let parser_def = quote::quote! {
        async fn #parser_name (
            args: &str,
            attachment_idx: usize,
            serenity_ctx: &::poise::serenity_prelude::Context,
            msg: &::poise::serenity_prelude::Message,
        ) -> Result<( #( #parsed_types, )* ), ParseError> {
            #parsed_rest
        }
    };

    let ty = &first.ty;
    let token = format_ident!("token_{}", first.idx);
    let parsed_tokens = (0..parsed_types.len())
        .into_iter()
        .map(|i| format_ident!("token_{}", i + first.idx + 1))
        .collect::<Vec<_>>();
    let tokens = match first.modifier {
        Some(Modifier::Lazy) => {
            if let Some(ty) = unwrap_generic(ty, "Option") {
                // Parse `#[lazy] Option<T>` by first trying `None`, then trying `Some` if it
                // failed.
                quote::quote! {
                    #parser_def

                    match #parser_name(args, attachment_idx, serenity_ctx, msg).await {
                        Ok(( #( #parsed_tokens, )* )) => Ok((None, #( #parsed_tokens, )* )),
                        Err(_) => {
                            let ((args, attachment_idx, #token)) =
                                <#ty as ::poise::PopArgument>::pop_from(
                                    args,
                                    attachment_idx,
                                    serenity_ctx,
                                    msg,
                                )
                                .await
                                .map_err(|(e, input)| (Some(e), input))?;

                            let ( #( #parsed_tokens, )* ) =
                                #parser_name(args, attachment_idx, serenity_ctx, msg).await?;
                            Ok((Some(#token), #( #parsed_tokens, )* ))
                         }
                     }
                }
            } else {
                return Err(syn::Error::new(
                    ty.span(),
                    "can only decorate `Option<T>` with #[lazy]",
                ));
            }
        },
        Some(Modifier::Rest) => {
            let (ty, found, empty) = if let Some(ty) = unwrap_generic(ty, "Option") {
                (ty, quote::quote! { Ok((Some(#token),)) }, quote::quote! { Ok((None,)) })
            } else {
                (
                    ty,
                    quote::quote! { Ok((#token,)) },
                    quote::quote! { Err((Some(::poise::TooFewArguments::default().into()), None)) },
                )
            };

            // Parse `#[rest] T` and `#[rest] Option<T>`. In the former case, the extra tail must
            // exist, or else this will error.
            quote::quote! {
                let input = args.trim_start();
                if input.is_empty() {
                    #empty
                } else {
                    let #token =
                        <#ty as ::poise::ArgumentConvert>::convert(
                            serenity_ctx,
                            msg.guild_id,
                            Some(msg.channel_id),
                            input,
                        )
                        .await
                        .map_err(|e| (Some(e.into()), Some(input.to_owned())))?;

                    #found
                }
            }
        },
        Some(Modifier::Flag) => {
            // Parse `#[flag] name: bool` by checking for the string "name" and converting to `true`
            // if present and `false` otherwise.
            let name = first.name;
            quote::quote! {
                #parser_def

                let mut args = args;
                let mut attachment_idx = attachment_idx;
                let mut error = (None, None);
                let #token = match <String as ::poise::PopArgument>::pop_from(
                    args,
                    attachment_idx,
                    serenity_ctx,
                    msg,
                ).await {
                    Ok((new_args, new_attachment_idx, #token)) => {
                        if #token.eq_ignore_ascii_case(#name) {
                            args = new_args;
                            attachment_idx = new_attachment_idx;
                            true
                        } else {
                            // Don't immediately return the error, given that the string we
                            // found might belong to the parameter after this one.
                            error = (
                                Some(concat!("Must use either `", #name, "` or nothing as a modifier").into()),
                                Some(#token),
                            );
                            false
                        }
                    }
                    Err(_) => false,
                };

                // If we ran out of arguments when parsing without the flag, propagate any
                // previous errors found instead.
                let ( #( #parsed_tokens, )* ) =
                    #parser_name(args, attachment_idx, serenity_ctx, msg)
                        .await
                        .map_err(|(e, input)| match e {
                            Some(e) => (Some(e), input),
                            None => error,
                        })?;
                Ok((#token, #( #parsed_tokens, )* ))
            }
        },
        Some(Modifier::String) => {
            // Parse a type which implements `FromStr` yet doesn't implement `ArgumentConvert`.
            if let Some(ty) = unwrap_generic(ty, "Option") {
                // Greedily parse `Option<T: FromStr>` by first trying to parse for `Some(T)`. If
                // that fails, try again with `None` instead.
                quote::quote! {
                    #parser_def

                    let mut error = (None, None);
                    if let Ok((args, attachment_idx, #token)) =
                        <String as ::poise::PopArgument>::pop_from(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        ).await
                    {
                        match <#ty as ::std::str::FromStr>::from_str(&#token) {
                            Ok(#token) => {
                                if let Ok(( #( #parsed_tokens, )* )) = #parser_name(
                                    args,
                                    attachment_idx,
                                    serenity_ctx,
                                    msg,
                                ).await {
                                    return Ok((Some(#token), #( #parsed_tokens, )* ))
                                }
                            }
                            // Don't immediately return the error, given that the string we
                            // found might belong to the parameter after this one.
                            Err(e) => error = (Some(e.into()), Some(#token)),
                        }
                    }

                    // If we ran out of arguments when parsing without `T`, propagate any
                    // previous errors found instead.
                    let ( #( #parsed_tokens, )* ) =
                        #parser_name(args, attachment_idx, serenity_ctx, msg)
                            .await
                            .map_err(|(e, input)| match e {
                                Some(e) => (Some(e), input),
                                None => error,
                            })?;
                    Ok((None, #( #parsed_tokens, )* ))
                }
            } else {
                // Here, we just have `T: FromStr`.
                quote::quote! {
                    #parser_def

                    let (args, attachment_idx, #token) =
                        <String as ::poise::PopArgument>::pop_from(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        )
                        .await
                        .map_err(|(e, input)| (Some(e), input))?;

                    let #token = <#ty as ::std::str::FromStr>::from_str(&#token)
                        .map_err(|e| (Some(e.into()), Some(#token)))?;

                    let ( #( #parsed_tokens, )* ) =
                        #parser_name(args, attachment_idx, serenity_ctx, msg).await?;
                    Ok((#token, #( #parsed_tokens, )* ))
                }
            }
        },
        None => {
            // Parse a parameter with no modifiers decorating it.
            if let Some(ty) = unwrap_generic(ty, "Option") {
                // Greedily parse `Option<T>` by first trying to parse for `Some(T)`. If that fails,
                // try again with `None` instead.
                quote::quote! {
                    #parser_def

                    if let Ok((args, attachment_idx, #token)) =
                        <#ty as ::poise::PopArgument>::pop_from(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        ).await
                    {
                        if let Ok(( #( #parsed_tokens, )* )) = #parser_name(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        ).await {
                            return Ok((Some(#token), #( #parsed_tokens, )* ))
                        }
                    }

                    let ( #( #parsed_tokens, )* ) =
                        #parser_name(args, attachment_idx, serenity_ctx, msg).await?;
                    Ok((None, #( #parsed_tokens, )* ))
                }
            } else if let Some(ty) = unwrap_generic(ty, "Vec") {
                // Greedily try to parse the longest `Vec<T>` possible. Then, backtrack as needed
                // until we successfully parse the rest of the parameters.
                quote::quote! {
                    #parser_def

                    let mut #token = Vec::new();
                    let mut rest = vec![args.clone()];

                    let mut args = args;
                    let mut attachment_idx = attachment_idx;

                    // We do not propagate errors here because parsing into a Vec<T> parameter with
                    // spare arguments would cause the error from the spare arguments to be the
                    // parse error for Vec<T>, which is confusing
                    while let Ok((new_args, new_attachment_idx, token)) =
                        <#ty as ::poise::PopArgument>::pop_from(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        ).await
                    {
                        #token.push(token);
                        rest.push(new_args.clone());
                        args = new_args;
                        attachment_idx = new_attachment_idx;
                    }

                    let mut error = None;
                    while let Some(args) = rest.pop() {
                        match #parser_name(args, attachment_idx, serenity_ctx, msg,).await {
                            Ok(( #( #parsed_tokens, )* )) => return Ok((#token, #( #parsed_tokens, )* )),
                            Err(e) => error = Some(e),
                        }
                        #token.pop();
                    }
                    Err(error.unwrap_or_default())
                }
            } else {
                // Here, we just have a `T`.
                quote::quote! {
                    #parser_def

                    let (args, attachment_idx, #token) =
                        <#ty as ::poise::PopArgument>::pop_from(
                            args,
                            attachment_idx,
                            serenity_ctx,
                            msg,
                        )
                        .await
                        .map_err(|(e, input)| (Some(e), input))?;

                    let ( #( #parsed_tokens, )* ) =
                        #parser_name(args, attachment_idx, serenity_ctx, msg).await?;
                    Ok((#token, #( #parsed_tokens, )* ))
                }
            }
        },
    };

    parsed_types.insert(0, ty.clone());
    Ok((tokens, parsed_types))
}

pub fn generate_prefix_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_idents =
        (0..inv.parameters.len()).map(|i| format_ident!("poise_param_{i}")).collect::<Vec<_>>();

    let mut params = inv
        .parameters
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            let modifier = get_modifier(param)?;
            if let Some(Modifier::Rest) = modifier {
                if idx != inv.parameters.len() - 1 {
                    return Err(syn::Error::new(
                        param.span,
                        "parameter marked `#[rest]` must come last in the argument list",
                    ));
                }
                if inv.args.discard_spare_arguments {
                    return Err(syn::Error::new(
                        param.span,
                        "cannot combine parameter marked `#[rest]` with `discard_spare_arguments`",
                    ));
                }
            }
            Ok(PrefixParameter { idx, modifier, ty: param.type_.clone(), name: &param.name })
        })
        .collect::<Result<Vec<_>, _>>()?;

    if inv.args.discard_spare_arguments {
        params.push(PrefixParameter {
            idx: params.len(),
            modifier: Some(Modifier::Rest),
            ty: syn::parse_quote! { Option<String> },
            name: "rest",
        })
    }

    let (parsed_params, _) = parse_prefix_params(&params)?;

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            let ( #( #param_idents, )* .. ) = async {
                type ParseError = (Option<Box<dyn std::error::Error + Send + Sync>>, Option<String>);

                let serenity_ctx = ctx.serenity_context();
                let msg = ctx.msg;
                let args = ctx.args;
                let attachment_idx = 0;

                #parsed_params
            }.await.map_err(|(error, input)| poise::FrameworkError::new_argument_parse(
                ctx.into(),
                input,
                error.unwrap_or_else(|| ::poise::TooManyArguments::default().into()),
            ))?;

            let is_framework_cooldown = !ctx.command().manual_cooldowns
                .unwrap_or_else(|| ctx.framework.options.manual_cooldowns);

            if is_framework_cooldown {
                ctx.command().cooldowns.lock().unwrap().start_cooldown(ctx.cooldown_context());
            }

            inner(ctx.into(), #( #param_idents, )* )
                .await
                .map_err(|error| poise::FrameworkError::new_command(
                    ctx.into(),
                    error,
                ))
        })
    })
}
