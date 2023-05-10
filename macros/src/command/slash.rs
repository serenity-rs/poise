use super::Invocation;
use crate::util::{extract_type_parameter, wrap_option_to_string};
use syn::spanned::Spanned as _;

pub fn generate_parameters(inv: &Invocation) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut parameter_structs = Vec::new();
    for param in &inv.parameters {
        // no #[description] check here even if slash_command set, so users can programatically
        // supply descriptions later (e.g. via translation framework like fluent)
        let description = match &param.args.description {
            Some(x) => quote::quote! { Some(#x.to_string()) },
            None => quote::quote! { None },
        };

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

        let param_name = match &param.args.rename {
            Some(rename) => wrap_option_to_string(Some(rename)),
            None => wrap_option_to_string(param.name.as_ref().map(|x| x.to_string())),
        };
        let name_locales = param.args.name_localized.iter().map(|x| &x.0);
        let name_localized_values = param.args.name_localized.iter().map(|x| &x.1);
        let description_locales = param.args.description_localized.iter().map(|x| &x.0);
        let description_localized_values = param.args.description_localized.iter().map(|x| &x.1);

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
                    let choices_json = choices_stream
                        .take(25)
                        // T or AutocompleteChoice<T> -> AutocompleteChoice<T>
                        .map(|value| poise::AutocompleteChoice::from(value))
                        // AutocompleteChoice<T> -> serde_json::Value
                        .map(|choice| poise::serenity_prelude::json::json!({
                            "name": choice.label,
                            "value": choice.value,
                        }))
                        .collect()
                        .await;

                    let mut response = poise::serenity_prelude::CreateAutocompleteResponse::default();
                    response.set_choices(poise::serenity_prelude::json::Value::Array(choices_json));
                    Ok(response)
                })) }
            }
            None => quote::quote! { None },
        };

        // We can just cast to f64 here because Discord only uses f64 precision anyways
        // TODO: move this to poise::CommandParameter::{min, max} fields
        let min_value_setter = match &param.args.min {
            Some(x) => quote::quote! { o.min_number_value(#x as f64); },
            None => quote::quote! {},
        };
        let max_value_setter = match &param.args.max {
            Some(x) => quote::quote! { o.max_number_value(#x as f64); },
            None => quote::quote! {},
        };
        // TODO: move this to poise::CommandParameter::{min_length, max_length} fields
        let min_length_setter = match &param.args.min_length {
            Some(x) => quote::quote! { o.min_length(#x); },
            None => quote::quote! {},
        };
        let max_length_setter = match &param.args.max_length {
            Some(x) => quote::quote! { o.max_length(#x); },
            None => quote::quote! {},
        };
        let type_setter = match inv.args.slash_command {
            true => quote::quote! { Some(|o| {
                poise::create_slash_argument!(#type_, o);
                #min_value_setter #max_value_setter
                #min_length_setter #max_length_setter
            }) },
            false => quote::quote! { None },
        };
        // TODO: theoretically a problem that we don't store choices for non slash commands
        // TODO: move this to poise::CommandParameter::choices (is there a reason not to?)
        let choices = match inv.args.slash_command {
            true => quote::quote! { poise::slash_argument_choices!(#type_) },
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
                    name: #param_name,
                    name_localizations: vec![
                        #( (#name_locales.to_string(), #name_localized_values.to_string()) ),*
                    ].into_iter().collect(),
                    description: #description,
                    description_localizations: vec![
                        #( (#description_locales.to_string(), #description_localized_values.to_string()) ),*
                    ].into_iter().collect(),
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

    let mut param_identifiers: Vec<syn::Ident> = Vec::new();
    let mut param_names: Vec<syn::Ident> = Vec::new();
    let mut param_types: Vec<syn::Type> = Vec::new();
    for p in &inv.parameters {
        let param_ident = p.name.clone().ok_or_else(|| {
            syn::Error::new(p.span, "parameter must have a name in slash commands")
        })?;

        param_identifiers.push(param_ident.clone());
        param_names.push(match &p.args.rename {
            Some(rename) => syn::Ident::new(rename, p.name.span()),
            None => param_ident,
        });
        param_types.push(match p.args.flag {
            true => syn::parse_quote! { FLAG },
            false => p.type_.clone(),
        });
    }

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            // idk why this can't be put in the macro itself (where the lint is triggered) and
            // why clippy doesn't turn off this lint inside macros in the first place
            #[allow(clippy::needless_question_mark)]

            let ( #( #param_identifiers, )* ) = ::poise::parse_slash_args!(
                ctx.serenity_context, ctx.interaction, ctx.args =>
                #( (#param_names: #param_types), )*
            ).await.map_err(|error| match error {
                poise::SlashArgError::CommandStructureMismatch { description, .. } => {
                    poise::FrameworkError::new_command_structure_mismatch(ctx, description)
                },
                poise::SlashArgError::Parse { error, input, .. } => {
                    poise::FrameworkError::new_argument_parse(
                        ctx.into(),
                        Some(input),
                        error,
                    )
                },
                poise::SlashArgError::__NonExhaustive => unreachable!(),
            })?;

            if !ctx.framework.options.manual_cooldowns {
                ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.into());
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
                    ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.into());
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
