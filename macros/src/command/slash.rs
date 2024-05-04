use super::Invocation;
use crate::util::{
    extract_type_parameter, iter_tuple_2_to_hash_map, tuple_2_iter_deref, wrap_option_to_string,
};
use quote::format_ident;
use syn::spanned::Spanned as _;

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
            iter_tuple_2_to_hash_map(tuple_2_iter_deref(&param.args.name_localized));
        let desc_localizations =
            iter_tuple_2_to_hash_map(tuple_2_iter_deref(&param.args.description_localized));

        let autocomplete_callback = match &param.args.autocomplete {
            Some(autocomplete_fn) => {
                quote::quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    partial: &str,
                | Box::pin(async move {
                    use ::poise::futures_util::{Stream, StreamExt};

                    let choices_stream = ::poise::into_stream!(
                        #autocomplete_fn(ctx.into(), partial).await
                    );
                    let choices_vec = choices_stream
                        .take(25)
                        // T or AutocompleteChoice<T> -> AutocompleteChoice<T>
                        .map(poise::serenity_prelude::AutocompleteChoice::from)
                        .collect()
                        .await;

                    let mut response = poise::serenity_prelude::CreateAutocompleteResponse::default();
                    Ok(response.set_choices(choices_vec))
                })) }
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
                } else {
                    quote::quote! { Some(|o| {
                        poise::create_slash_argument!(#type_, o)
                        #min_value_setter #max_value_setter
                        #min_length_setter #max_length_setter
                    }) }
                }
            }
            false => quote::quote! { None },
        };
        // TODO: theoretically a problem that we don't store choices for non slash commands
        // TODO: move this to poise::CommandParameter::choices (is there a reason not to?)
        let choices = match inv.args.slash_command {
            true => {
                if let Some(choices) = &param.args.choices {
                    let choices = &choices.0;
                    quote::quote! { vec![#( ::poise::CommandParameterChoice {
                        name: ToString::to_string(&#choices),
                        localizations: Default::default(),
                        __non_exhaustive: (),
                    } ),*] }
                } else {
                    quote::quote! { poise::slash_argument_choices!(#type_) }
                }
            }
            false => quote::quote! { vec![] },
        };

        let channel_types = match &param.args.channel_types {
            Some(crate::util::List(channel_types)) => quote::quote! { Some(
                vec![ #( poise::serenity_prelude::ChannelType::#channel_types ),* ]
            ) },
            None => quote::quote! { None },
        };

        parameter_structs.push((
            quote::quote! {
                ::poise::CommandParameter {
                    name: #param_name.to_string(),
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
                quote::quote! { FLAG }
            } else if let Some(choices) = &p.args.choices {
                let choice_indices = (0..choices.0.len()).map(syn::Index::from);
                let choice_vals = &choices.0;
                quote::quote! { INLINE_CHOICE #t [#(#choice_indices: #choice_vals),*] }
            } else {
                quote::quote! { #t }
            }
        })
        .collect::<Vec<_>>();

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            // idk why this can't be put in the macro itself (where the lint is triggered) and
            // why clippy doesn't turn off this lint inside macros in the first place
            #[allow(clippy::needless_question_mark)]

            let ( #( #param_identifiers, )* ) = ::poise::parse_slash_args!(
                ctx.serenity_context, ctx.interaction, ctx.args =>
                #( (#param_names: #param_types), )*
            ).await.map_err(|error| error.to_framework_error(ctx))?;

            if !ctx.framework.options.manual_cooldowns {
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

    Ok(quote::quote! {
        <#param_type as ::poise::ContextMenuParameter<_, _>>::to_action(|ctx, value| {
            Box::pin(async move {
                if !ctx.framework.options.manual_cooldowns {
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
