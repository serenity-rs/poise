use super::{unwrap_generic, CommandParameter, Invocation};
use quote::format_ident;
use syn::spanned::Spanned as _;

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
    match (
        param.args.lazy,
        param.args.rest,
        param.args.flag,
        param.args.string,
    ) {
        (false, false, false, false) => Ok(None),
        (true, false, false, false) => Ok(Some(Modifier::Lazy)),
        (false, true, false, false) => Ok(Some(Modifier::Rest)),
        (false, false, true, false) => {
            if param.type_ == syn::parse_quote! { bool } {
                Ok(Some(Modifier::Flag))
            } else {
                Err(syn::Error::new(
                    param.type_.span(),
                    "Must use bool for flags",
                ))
            }
        }
        (false, false, false, true) => Ok(Some(Modifier::String)),
        _ => Err(syn::Error::new(
            param.span,
            "modifiers like #[lazy] or #[rest] cannot be used together",
        )),
    }
}

fn parse_prefix_params(
    params: &[PrefixParameter],
    num_parsed: usize,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let tokens = if let Some((first, rest)) = params.split_first() {
        // Recursively parse the parameters after this one.
        let parsed_rest = parse_prefix_params(rest, num_parsed + 1)?;

        let ty = &first.ty;
        let token = format_ident!("token_{}", first.idx);
        match first.modifier {
            Some(Modifier::Lazy) => {
                let Some(ty) = unwrap_generic(ty, "Option") else {
                    return Err(syn::Error::new(
                        ty.span(),
                        "can only decorate `Option<T>` with #[lazy]",
                    ));
                };
                parse_lazy(&ty, token, parsed_rest)
            }
            Some(Modifier::Rest) => parse_rest(ty, token, parsed_rest),
            Some(Modifier::Flag) => parse_flag(&first.name, token, parsed_rest),
            Some(Modifier::String) => parse_string(ty, token, parsed_rest),
            None => parse_param(ty, token, parsed_rest),
        }
    } else {
        // Once the input is exhausted and parsing was successful, we output the tokens we parsed.
        let tokens = (0..num_parsed)
            .into_iter()
            .map(|i| format_ident!("token_{}", i))
            .collect::<Vec<_>>();
        quote::quote! {
            if args.is_empty() {
                return Ok(( #( #tokens, )* ));
            }
        }
    };
    Ok(tokens)
}

/// Parse `#[lazy] Option<T>` by first trying `None`, then trying `Some` if it failed.
fn parse_lazy(
    ty: &syn::Type,
    token: syn::Ident,
    parsed_rest: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let #token: Option<#ty> = None;
        #parsed_rest
        match <#ty as ::poise::PopArgument>::pop_from(
            args,
            attachment_idx,
            serenity_ctx,
            msg,
        ).await {
            Ok((args, attachment_idx, #token)) => {
                let #token: Option<#ty> = Some(#token);
                #parsed_rest
            }
            Err(e) => error = e,
        }
    }
}

/// Parses `#[rest] T` and `#[rest] Option<T>`. In the former case, the extra tail _must_ exist, or
/// else this will error.
fn parse_rest(
    ty: &syn::Type,
    token: syn::Ident,
    parsed_rest: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    let (ty, pass, fail) = if let Some(ty) = unwrap_generic(ty, "Option") {
        (
            ty,
            quote::quote! { let #token: Option<#ty> = Some(#token); },
            quote::quote! {
                let #token: Option<#ty> = None;
                #parsed_rest
            },
        )
    } else {
        (
            ty,
            quote::quote! {},
            quote::quote! { error = (::poise::TooFewArguments::default().into(), None); },
        )
    };

    quote::quote! {
        let input = args.trim_start();
        if input.is_empty() {
            #fail
        } else {
            match <#ty as ::poise::serenity_prelude::ArgumentConvert>::convert(
                serenity_ctx,
                msg.guild_id,
                Some(msg.channel_id),
                input,
            ).await {
                Ok(#token) => {
                    let args = "";
                    #pass
                    #parsed_rest
                },
                Err(e) => error = (e.into(), Some(input.to_owned())),
            }
        }
    }
}

/// Parse `#[flag] name: bool` by checking for the string "name" and converting to `true` if
/// present and `false` otherwise.
fn parse_flag(
    name: &str,
    token: syn::Ident,
    parsed_rest: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    quote::quote! {
        let #token = match <String as ::poise::PopArgument>::pop_from(
            args,
            attachment_idx,
            serenity_ctx,
            msg,
        ).await {
            Ok((args, attachment_idx, #token)) if #token.eq_ignore_ascii_case(#name) => true,
            _ => {
                error = (concat!("Must use either `", #name, "` or nothing as a modifier").into(), None);
                false
            }
        };
        #parsed_rest
    }
}

/// Parse a type which implements `FromStr` yet doesn't implement `ArgumentConvert`.
fn parse_string(
    ty: &syn::Type,
    token: syn::Ident,
    parsed_rest: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if let Some(ty) = unwrap_generic(ty, "Option") {
        // Greedily parse `Option<T: FromStr>` by first trying to parse for `Some(T: FromStr)`.
        // If that fails, try again with `None` instead.
        quote::quote! {
            match <String as ::poise::PopArgument>::pop_from(
                args,
                attachment_idx,
                serenity_ctx,
                msg,
            ).await {
                Ok((args, attachment_idx, #token)) => {
                    match <#ty as ::std::str::FromStr>::from_str(&#token) {
                        Ok(#token) => {
                            let #token: Option<#ty> = Some(#token);
                            #parsed_rest
                        },
                        Err(e) => error = (e.into(), Some(#token)),
                    }
                },
                Err(e) => error = e,
            }
            let #token: Option<#ty> = None;
            #parsed_rest
        }
    } else {
        // Here, we just have `T: FromStr`.
        quote::quote! {
            match <String as ::poise::PopArgument>::pop_from(
                args,
                attachment_idx,
                serenity_ctx,
                msg,
            ).await {
                Ok((args, attachment_idx, #token)) => {
                    match <#ty as ::std::str::FromStr>::from_str(&#token) {
                        Ok(#token) => { #parsed_rest },
                        Err(e) => error = (e.into(), Some(#token)),
                    }
                },
                Err(e) => error = e,
            }
        }
    }
}

/// Parse a parameter with no modifiers decorating it.
fn parse_param(
    ty: &syn::Type,
    token: syn::Ident,
    parsed_rest: proc_macro2::TokenStream,
) -> proc_macro2::TokenStream {
    if let Some(ty) = unwrap_generic(ty, "Option") {
        // Greedily parse `Option<T>` by first trying to parse for `Some(T)`. If that fails, try
        // again with `None` instead.
        quote::quote! {
            match <#ty as ::poise::PopArgument>::pop_from(
                args,
                attachment_idx,
                serenity_ctx,
                msg,
            ).await {
                Ok((args, attachment_idx, #token)) => {
                    let #token: Option<#ty> = Some(#token);
                    #parsed_rest
                }
                Err(e) => error = e,
            }
            let #token: Option<#ty> = None;
            #parsed_rest
        }
    } else if let Some(ty) = unwrap_generic(ty, "Vec") {
        // Greedily try to parse the longest `Vec<T>` possible. Then, backtrack as needed until we
        // successfully parse the rest of the parameters.
        quote::quote! {
            let mut #token = Vec::new();
            let mut rest = vec![args.clone()];

            let mut args = args.clone();
            let mut attachment_idx = attachment_idx;

            // We do not propagate errors here because parsing into a Vec<T> parameter with spare
            // arguments would cause the error from the spare arguments to be the parse error for
            // Vec<T>, which is confusing
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

            while let Some(args) = rest.pop() {
                #parsed_rest
                #token.pop();
            }
        }
    } else {
        // Here, we just have a `T`.
        quote::quote! {
            match <#ty as ::poise::PopArgument>::pop_from(
                args,
                attachment_idx,
                serenity_ctx,
                msg,
            ).await {
                Ok((args, attachment_idx, #token)) => { #parsed_rest },
                Err(e) => error = e,
            }
        }
    }
}

pub fn generate_prefix_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_idents = (0..inv.parameters.len())
        .map(|i| format_ident!("poise_param_{i}"))
        .collect::<Vec<_>>();

    let mut params = inv
        .parameters
        .iter()
        .enumerate()
        .map(|(idx, param)| {
            get_modifier(param).map(|modifier| PrefixParameter {
                idx,
                modifier,
                ty: param.type_.clone(),
                name: &param.name,
            })
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

    let parsed_params = parse_prefix_params(&params, 0)?;

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            let ( #( #param_idents, )* .. ) = async {
                let serenity_ctx = ctx.serenity_context();
                let msg = ctx.msg;
                let args = ctx.args;
                let attachment_idx = 0;

                let mut error: (Box<dyn std::error::Error + Send + Sync>, Option<String>)
                    = (::poise::TooManyArguments::default().into(), None);

                #parsed_params

                Err(error)
            }.await.map_err(|(error, input)| poise::FrameworkError::new_argument_parse(
                ctx.into(),
                input,
                error,
            ))?;

            let is_framework_cooldown = !ctx.command.manual_cooldowns
                .unwrap_or_else(|| ctx.framework.options.manual_cooldowns);

            if is_framework_cooldown {
                ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.cooldown_context());
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
