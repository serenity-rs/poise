use super::{unwrap_generic, CommandParameter, Invocation};
use crate::util::{
    extract_type_parameter, iter_tuple_2_to_vec_map, tuple_2_iter_deref, wrap_option_to_string,
};
use quote::format_ident;
use syn::spanned::Spanned as _;

fn lit_to_string(lit: &syn::Lit) -> Result<String, syn::Error> {
    match lit {
        syn::Lit::Str(lit_str) => Ok(lit_str.value()),
        syn::Lit::Char(lit_char) => Ok(lit_char.value().to_string()),
        syn::Lit::Int(lit_int) => Ok(lit_int.base10_digits().to_owned()),
        syn::Lit::Float(lit_float) => Ok(lit_float.token().to_string()),
        syn::Lit::Bool(lit_bool) => Ok(lit_bool.value.to_string()),

        _ => Err(syn::Error::new(
            lit.span(),
            "Inline choice must be convertable to a string at compile time",
        )),
    }
}

pub fn generate_parameters(inv: &Invocation) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut parameter_structs = Vec::new();
    for param in &inv.parameters {
        // no #[description] check here even if slash_command set, so users can programatically
        // supply descriptions later (e.g. via translation framework like fluent)
        let description = wrap_option_to_string(param.args.description.as_ref());

        let (mut required, type_) = match extract_type_parameter("Option", &param.type_)
            .or_else(|| extract_type_parameter("Vec", &param.type_))
        {
            Some(t) => (false, t),
            None => (true, &param.type_),
        };

        // Don't require user to input a value for flags - use false as default value (see below)
        if param.args.flag {
            required = false;
        }

        let param_name = &param.name;
        let name_localizations =
            iter_tuple_2_to_vec_map(tuple_2_iter_deref(&param.args.name_localized));
        let desc_localizations =
            iter_tuple_2_to_vec_map(tuple_2_iter_deref(&param.args.description_localized));

        let autocomplete_callback = match &param.args.autocomplete {
            Some(autocomplete_fn) => {
                quote::quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    partial: &str,
                | Box::pin(#autocomplete_fn(ctx.into(), partial))) }
            }
            None => quote::quote! { None },
        };

        // We can just cast to f64 here because Discord only uses f64 precision anyways
        // TODO: move this to poise::CommandParameter::{min, max} fields
        let min_value_setter = match &param.args.min {
            Some(x) => quote::quote! { .min_number_value(#x as f64) },
            None => quote::quote! {},
        };
        let max_value_setter = match &param.args.max {
            Some(x) => quote::quote! { .max_number_value(#x as f64) },
            None => quote::quote! {},
        };
        // TODO: move this to poise::CommandParameter::{min_length, max_length} fields
        let min_length_setter = match &param.args.min_length {
            Some(x) => quote::quote! { .min_length(#x) },
            None => quote::quote! {},
        };
        let max_length_setter = match &param.args.max_length {
            Some(x) => quote::quote! { .max_length(#x) },
            None => quote::quote! {},
        };
        let type_setter = match inv.args.slash_command {
            true => {
                if let Some(_choices) = &param.args.choices {
                    quote::quote! { Some(|o| o.kind(::poise::serenity_prelude::CommandOptionType::Integer)) }
                } else if param.args.string {
                    quote::quote! { Some(|o| o.kind(::poise::serenity_prelude::CommandOptionType::String)) }
                } else {
                    quote::quote! { Some(|o| {
                        <#type_ as ::poise::SlashArgument>::create(o)
                        #min_value_setter #max_value_setter
                        #min_length_setter #max_length_setter
                    }) }
                }
            }
            false => quote::quote! { None },
        };
        // TODO: theoretically a problem that we don't store choices for non slash commands
        // TODO: move this to poise::CommandParameter::choices (is there a reason not to?)
        let choices = if inv.args.slash_command {
            if let Some(choices) = &param.args.choices {
                let choices_iter = choices.0.iter();
                let choices: Vec<_> = choices_iter.map(lit_to_string).collect::<Result<_, _>>()?;

                quote::quote! { Cow::Borrowed(&[#( ::poise::CommandParameterChoice {
                    name: Cow::Borrowed(#choices),
                    localizations: Cow::Borrowed(&[]),
                    __non_exhaustive: (),
                } ),*]) }
            } else if param.args.string {
                quote::quote! { Cow::Borrowed(&[]) }
            } else {
                quote::quote! { <#type_ as ::poise::SlashArgument>::choices() }
            }
        } else {
            quote::quote! { Cow::Borrowed(&[]) }
        };

        let channel_types = match &param.args.channel_types {
            Some(crate::util::List(channel_types)) => quote::quote! { Some(
                Cow::Borrowed(&[ #( poise::serenity_prelude::ChannelType::#channel_types ),* ])
            ) },
            None => quote::quote! { None },
        };

        parameter_structs.push((
            quote::quote! {
                ::poise::CommandParameter {
                    name: ::std::borrow::Cow::Borrowed(#param_name),
                    name_localizations: #name_localizations,
                    description: #description,
                    description_localizations: #desc_localizations,
                    required: #required,
                    channel_types: #channel_types,
                    type_setter: #type_setter,
                    choices: #choices,
                    autocomplete_callback: #autocomplete_callback,
                    __non_exhaustive: (),
                }
            },
            required,
        ));
    }
    // Sort the parameters so that optional parameters come last - Discord requires this order
    parameter_structs.sort_by_key(|(_, required)| !required);
    Ok(parameter_structs
        .into_iter()
        .map(|(builder, _)| builder)
        .collect::<Vec<_>>())
}

pub fn generate_slash_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    if let Some(desc) = &inv.description {
        let length = desc.chars().count();
        if length > 100 {
            return Err(syn::Error::new(
                inv.function.span(),
                format!(
                    "slash command description too long ({} chars, must be max 100)",
                    length
                ),
            ));
        }
    }

    let param_identifiers = (0..inv.parameters.len())
        .map(|i| format_ident!("poise_param_{i}"))
        .collect::<Vec<_>>();

    let params = inv
        .parameters
        .iter()
        .map(parse_slash_param)
        .collect::<Vec<_>>();

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            let ( #( #param_identifiers, )* ) = async {
                let serenity_ctx = ctx.serenity_context();
                let interaction = ctx.interaction;
                let args = ctx.args;

                Ok::<_, ::poise::SlashArgError>((#( #params, )*))
            }
            .await
            .map_err(|error| error.to_framework_error(ctx))?;

            let is_framework_cooldown = !ctx.command.manual_cooldowns
                .unwrap_or_else(|| ctx.framework.options.manual_cooldowns);

            if is_framework_cooldown {
                ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.cooldown_context());
            }

            inner(ctx.into(), #( #param_identifiers, )*)
                .await
                .map_err(|error| poise::FrameworkError::new_command(
                    ctx.into(),
                    error,
                ))
        })
    })
}

fn parse_slash_param(param: &CommandParameter) -> proc_macro2::TokenStream {
    fn extract_slash_argument(ty: impl quote::ToTokens) -> proc_macro2::TokenStream {
        quote::quote! {
            <#ty as ::poise::SlashArgument>::extract(
                serenity_ctx,
                interaction,
                &arg.value
            )
            .await?
        }
    }

    let name = &param.name;
    let ty = &param.type_;

    if param.args.flag {
        // Extract #[flag]
        let extract = extract_slash_argument(quote::quote! { bool });
        return quote::quote! {
            match args.iter().find(|arg| arg.name == #name) {
                Some(arg) => #extract,
                None => false,
            }
        };
    }

    enum Wrapper {
        Option,
        Vec,
        Plain,
    }

    let (wrapper, ty) = if let Some(ty) = unwrap_generic(ty, "Option") {
        (Wrapper::Option, ty)
    } else if let Some(ty) = unwrap_generic(ty, "Vec") {
        (Wrapper::Vec, ty)
    } else {
        (Wrapper::Plain, ty)
    };

    let extract = if let Some(choices) = &param.args.choices {
        // Extract #[choices(...)]
        let choice_indices = (0..choices.0.len()).map(syn::Index::from);
        let choice_vals = &choices.0;

        quote::quote! {{
            let ::poise::serenity_prelude::ResolvedValue::Integer(index) = arg.value else {
                return Err(::poise::SlashArgError::new_command_structure_mismatch(
                    "expected integer, as the index for an inline choice parameter")
                );
            };
            match index {
                #( #choice_indices => #choice_vals, )*
                _ => {
                    return Err(::poise::SlashArgError::new_command_structure_mismatch(
                        "out of range index for inline choice parameter"
                    ));
                }
            }
        }}
    } else if param.args.string {
        let extract = extract_slash_argument(quote::quote!(String));

        quote::quote! {{
            let input = #extract;
            <#ty as ::std::str::FromStr>::from_str(&input)
                .map_err(|e| ::poise::SlashArgError::new_parse(e.into(), input))?
        }}
    } else {
        // Extract T
        extract_slash_argument(ty)
    };

    match wrapper {
        Wrapper::Plain => quote::quote! {
            match args.iter().find(|arg| arg.name == #name) {
                Some(arg) => #extract,
                None => {
                    return Err(::poise::SlashArgError::new_command_structure_mismatch(
                        "a required argument is missing"
                    ));
                }
            }
        },
        Wrapper::Option => quote::quote! {
            match args.iter().find(|arg| arg.name == #name) {
                Some(arg) => Some(#extract),
                None => None,
            }
        },
        // Slash commands don't support variadic arguments right now
        Wrapper::Vec => quote::quote! {
            match args.iter().find(|arg| arg.name == #name) {
                Some(arg) => vec![#extract],
                None => vec![],
            }
        },
    }
}

pub fn generate_context_menu_action(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_type = match &*inv.parameters {
        [single_param] => &single_param.type_,
        _ => {
            return Err(syn::Error::new(
                inv.function.sig.inputs.span(),
                "Context menu commands require exactly one parameter",
            ));
        }
    };

    Ok(quote::quote! {
        <#param_type as ::poise::ContextMenuParameter<_, _>>::to_action(|ctx, value| {
            Box::pin(async move {
                let is_framework_cooldown = !ctx.command.manual_cooldowns
                    .unwrap_or_else(|| ctx.framework.options.manual_cooldowns);

                if is_framework_cooldown {
                    ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.cooldown_context());
                }

                inner(ctx.into(), value)
                    .await
                    .map_err(|error| poise::FrameworkError::new_command(
                        ctx.into(),
                        error,
                    ))
            })
        })
    })
}
