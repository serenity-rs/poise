use super::{Invocation, ParamArgs};
use crate::util::{
    extract_type_parameter, iter_tuple_2_to_vec_map, tuple_2_iter_deref, wrap_option,
    wrap_option_to_string,
};
use quote::{format_ident, quote};
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

fn generate_value_limits(
    args: &ParamArgs,
    span: proc_macro2::Span,
) -> syn::Result<Option<proc_macro2::TokenStream>> {
    let limits = match (&args.min, &args.max, &args.min_length, &args.max_length) {
        (None, None, None, Some(max)) => {
            quote!( ::poise::ValueLimits::Length { min: None, max: Some(#max) } )
        }
        (None, None, Some(min), None) => {
            quote!( ::poise::ValueLimits::Length { min: Some(#min), max: None } )
        }
        (None, None, Some(min), Some(max)) => {
            quote!( ::poise::ValueLimits::Length { min: Some(#min), max: Some(#max) } )
        }
        (None, Some(max), None, None) => {
            quote!( ::poise::ValueLimits::Value { min: None, max: Some((#max) as f64) } )
        }
        (Some(min), None, None, None) => {
            quote!( ::poise::ValueLimits::Value { min: Some((#min) as f64), max: None } )
        }
        (Some(min), Some(max), None, None) => {
            quote!( ::poise::ValueLimits::Value { min: Some((#min) as f64), max: Some((#max) as f64) } )
        }

        (None, None, None, None) => return Ok(None),
        _ => {
            let err = "Cannot set both a `min_length/max_length` and a `min/max`";
            return Err(syn::Error::new(span, err));
        }
    };

    Ok(Some(limits))
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
                quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    partial: &str,
                | Box::pin(#autocomplete_fn(ctx.into(), partial))) }
            }
            None => quote! { None },
        };

        let value_limits = wrap_option(generate_value_limits(&param.args, param.span)?);

        let type_setter = match inv.args.slash_command {
            true => {
                if let Some(_choices) = &param.args.choices {
                    quote! { Some(|o| o.kind(::poise::serenity_prelude::CommandOptionType::Integer)) }
                } else {
                    quote! { Some(|o| poise::create_slash_argument!(#type_, o)) }
                }
            }
            false => quote! { None },
        };

        // TODO: theoretically a problem that we don't store choices for non slash commands
        // TODO: move this to poise::CommandParameter::choices (is there a reason not to?)
        let choices = if inv.args.slash_command {
            if let Some(choices) = &param.args.choices {
                let choices_iter = choices.0.iter();
                let choices: Vec<_> = choices_iter.map(lit_to_string).collect::<Result<_, _>>()?;

                quote! { Cow::Borrowed(&[#( ::poise::CommandParameterChoice {
                    name: Cow::Borrowed(#choices),
                    localizations: Cow::Borrowed(&[]),
                    __non_exhaustive: (),
                } ),*]) }
            } else {
                quote! { poise::slash_argument_choices!(#type_) }
            }
        } else {
            quote! { Cow::Borrowed(&[]) }
        };

        let channel_types = match &param.args.channel_types {
            Some(crate::util::List(channel_types)) => quote! { Some(
                Cow::Borrowed(&[ #( poise::serenity_prelude::ChannelType::#channel_types ),* ])
            ) },
            None => quote! { None },
        };

        parameter_structs.push((
            quote! {
                ::poise::CommandParameter {
                    name: ::std::borrow::Cow::Borrowed(#param_name),
                    name_localizations: #name_localizations,
                    description: #description,
                    description_localizations: #desc_localizations,
                    required: #required,
                    channel_types: #channel_types,
                    value_limits: #value_limits,
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
        if desc.len() > 100 {
            return Err(syn::Error::new(
                inv.function.span(),
                format!(
                    "slash command description too long ({} chars, must be max 100)",
                    desc.len()
                ),
            ));
        }
    }

    let param_identifiers = (0..inv.parameters.len())
        .map(|i| format_ident!("poise_param_{i}"))
        .collect::<Vec<_>>();
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();

    let param_types = inv
        .parameters
        .iter()
        .map(|p| {
            let t = &p.type_;
            if p.args.flag {
                quote! { FLAG }
            } else if let Some(choices) = &p.args.choices {
                let choice_indices = (0..choices.0.len()).map(syn::Index::from);
                let choice_vals = &choices.0;
                quote! { INLINE_CHOICE #t [#(#choice_indices: #choice_vals),*] }
            } else {
                quote! { #t }
            }
        })
        .collect::<Vec<_>>();

    Ok(quote! {
        |ctx| Box::pin(async move {
            // idk why this can't be put in the macro itself (where the lint is triggered) and
            // why clippy doesn't turn off this lint inside macros in the first place
            #[allow(clippy::needless_question_mark)]

            let ( #( #param_identifiers, )* ) = ::poise::parse_slash_args!(
                ctx.serenity_context(), ctx.interaction, ctx.args =>
                #( (#param_names: #param_types), )*
            ).await.map_err(|error| error.to_framework_error(ctx))?;

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

pub fn generate_context_menu_action(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_type = match &*inv.parameters {
        [single_param] => &single_param.type_,
        _ => {
            return Err(syn::Error::new(
                inv.function.sig.inputs.span(),
                "Context menu commands require exactly one parameter",
            ))
        }
    };

    Ok(quote! {
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
