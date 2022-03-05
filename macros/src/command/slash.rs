use super::Invocation;
use crate::util::{extract_type_parameter, wrap_option};
use syn::spanned::Spanned as _;

pub fn generate_parameters(inv: &Invocation) -> Result<Vec<proc_macro2::TokenStream>, syn::Error> {
    let mut parameter_structs = Vec::new();
    for param in &inv.parameters {
        if inv.args.slash_command && param.args.description.is_none() {
            return Err(syn::Error::new(
                param.span,
                "slash command parameters must have a description",
            ));
        }
        let description = wrap_option(param.args.description.as_ref());

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
        let autocomplete_callback = match &param.args.autocomplete {
            Some(autocomplete_fn) => {
                quote::quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    json_value: &poise::serenity::json::Value,
                | Box::pin(async move {
                    use ::poise::futures_util::{Stream, StreamExt};

                    let partial_input = poise::extract_autocomplete_argument!(#type_, json_value)?;

                    let choices_stream = ::poise::into_stream!(
                        #autocomplete_fn(ctx.into(), partial_input).await
                    );
                    let choices_json = choices_stream
                        .take(25)
                        .map(|value| poise::AutocompleteChoice::from(value))
                        .map(|choice| poise::serenity::json::json!({
                            "name": choice.name,
                            "value": poise::autocomplete_argument_into_json!(#type_, choice.value),
                        }))
                        .collect()
                        .await;

                    let mut response = poise::serenity::builder::CreateAutocompleteResponse::default();
                    response.set_choices(poise::serenity::json::Value::Array(choices_json));
                    Ok(response)
                })) }
            }
            None => quote::quote! { None },
        };

        // We can just cast to f64 here because Discord only uses f64 precision anyways
        let min_value_setter = match &param.args.min {
            Some(x) => quote::quote! { o.min_number_value(#x as f64); },
            None => quote::quote! {},
        };
        let max_value_setter = match &param.args.max {
            Some(x) => quote::quote! { o.max_number_value(#x as f64); },
            None => quote::quote! {},
        };
        let type_setter = match inv.args.slash_command {
            true => quote::quote! { Some(|o| {
                poise::create_slash_argument!(#type_, o);
                #min_value_setter #max_value_setter
            }) },
            false => quote::quote! { None },
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
                    name: stringify!(#param_name),
                    description: #description,
                    required: #required,
                    channel_types: #channel_types,
                    type_setter: #type_setter,
                    autocomplete_callback: #autocomplete_callback,
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

    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv
        .parameters
        .iter()
        .map(|p| match p.args.flag {
            true => syn::parse_quote! { FLAG },
            false => p.type_.clone(),
        })
        .collect::<Vec<_>>();

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            // idk why this can't be put in the macro itself (where the lint is triggered) and
            // why clippy doesn't turn off this lint inside macros in the first place
            #[allow(clippy::needless_question_mark)]

            let ( #( #param_names, )* ) = ::poise::parse_slash_args!(
                ctx.discord, ctx.interaction, ctx.args =>
                #( (#param_names: #param_types), )*
            ).await.map_err(|error| match error {
                poise::SlashArgError::CommandStructureMismatch(description) => {
                    poise::FrameworkError::CommandStructureMismatch { ctx, description }
                },
                poise::SlashArgError::Parse { error, input } => {
                    poise::FrameworkError::ArgumentParse {
                        ctx: ctx.into(),
                        error,
                        input: Some(input),
                    }
                },
            })?;

            inner(ctx.into(), #( #param_names, )*)
                .await
                .map_err(|error| poise::FrameworkError::Command {
                    error,
                    ctx: ctx.into(),
                })
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
                inner(ctx.into(), value)
                    .await
                    .map_err(|error| poise::FrameworkError::Command {
                        error,
                        ctx: ctx.into(),
                    })
            })
        })
    })
}
