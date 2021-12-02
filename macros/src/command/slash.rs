use syn::spanned::Spanned as _;

use super::{extract_option_type, extract_vec_type, Invocation};

fn generate_options(inv: &Invocation) -> proc_macro2::TokenStream {
    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.more.check {
        Some(check) => {
            quote::quote! { Some(|ctx| Box::pin(#check(ctx.into()))) }
        }
        None => quote::quote! { None },
    };
    let on_error = match &inv.more.on_error {
        Some(on_error) => quote::quote! {
            Some(|err, ctx| Box::pin(#on_error(err, ::poise::CommandErrorContext::Application(ctx))))
        },
        None => quote::quote! { None },
    };

    let ephemeral = inv.more.ephemeral;
    quote::quote! {
        ::poise::ApplicationCommandOptions {
            check: #check,
            on_error: #on_error,
            ephemeral: #ephemeral,
        }
    }
}

pub fn generate_slash_command_spec(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let command_name = &inv.command_name;
    let description = inv.description.as_ref().ok_or_else(|| {
        syn::Error::new(
            inv.function.sig.span(),
            "slash commands must have a description (doc comment)",
        )
    })?;

    let mut parameter_structs = Vec::new();
    for param in inv.parameters {
        let description = param.more.description.as_ref().ok_or_else(|| {
            syn::Error::new(
                param.span,
                "slash command parameters must have a description",
            )
        })?;

        let (mut required, type_) =
            match extract_option_type(&param.type_).or_else(|| extract_vec_type(&param.type_)) {
                Some(t) => (false, t),
                None => (true, &param.type_),
            };

        // Don't require user to input a value for flags - use false as default value (see below)
        if param.more.flag {
            required = false;
        }

        let param_name = &param.name;
        let autocomplete_callback = match &param.more.autocomplete {
            Some(autocomplete_fn) => {
                quote::quote! { Some(|
                    ctx: poise::ApplicationContext<'_, _, _>,
                    interaction: &poise::serenity_prelude::AutocompleteInteraction,
                    options: &[poise::serenity_prelude::ApplicationCommandInteractionDataOption],
                | Box::pin(async move {
                    use ::poise::futures::{Stream, StreamExt};

                    let choice = match options
                        .iter()
                        .find(|option| option.focused && option.name == stringify!(#param_name))
                    {
                        Some(x) => x,
                        None => return Ok(()),
                    };

                    let json_value = choice.value
                        .as_ref()
                        .ok_or(::poise::SlashArgError::CommandStructureMismatch("expected argument value"))?;
                    let partial_input = (&&&&&std::marker::PhantomData::<#type_>).extract_partial(json_value)?;

                    let choices_stream = ::poise::into_stream!(
                        #autocomplete_fn(ctx.into(), partial_input).await
                    );
                    let choices_json = choices_stream
                        .take(25)
                        .map(|value| poise::AutocompleteChoice::from(value))
                        .map(|choice| serde_json::json!({
                            "name": choice.name,
                            "value": (&&&&&std::marker::PhantomData::<#type_>).into_json(choice.value),
                        }))
                        .collect()
                        .await;
                    let choices_json = poise::serde_json::Value::Array(choices_json);

                    if let Err(e) = interaction
                        .create_autocomplete_response(
                            &ctx.discord.http,
                            |b| b.set_choices(choices_json),
                        )
                        .await
                    {
                        println!("Warning: couldn't send autocomplete response: {}", e);
                    }

                    Ok(())
                })) }
            }
            None => quote::quote! { None },
        };

        let is_autocomplete = param.more.autocomplete.is_some();
        parameter_structs.push((
            quote::quote! {
                ::poise::SlashCommandParameter {
                    builder: |o| (&&&&&std::marker::PhantomData::<#type_>).create(o)
                        .required(#required)
                        .name(stringify!(#param_name))
                        .description(#description)
                        .set_autocomplete(#is_autocomplete),
                    autocomplete_callback: #autocomplete_callback,
                }
            },
            required,
        ));
    }
    // Sort the parameters so that optional parameters come last - Discord requires this order
    parameter_structs.sort_by_key(|(_, required)| !required);
    let parameter_structs = parameter_structs
        .into_iter()
        .map(|(builder, _)| builder)
        .collect::<Vec<_>>();

    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv
        .parameters
        .iter()
        .map(|p| match p.more.flag {
            true => syn::parse_quote! { FLAG },
            false => p.type_.clone(),
        })
        .collect::<Vec<_>>();
    let options = generate_options(inv);
    Ok(quote::quote! {
        ::poise::SlashCommand {
            name: #command_name,
            description: #description,
            parameters: {
                use ::poise::{SlashArgumentHack, AutocompletableHack};
                vec![ #( #parameter_structs, )* ]
            },
            action: |ctx, args| Box::pin(async move {
                // idk why this can't be put in the macro itself (where the lint is triggered) and
                // why clippy doesn't turn off this lint inside macros in the first place
                #[allow(clippy::needless_question_mark)]

                let ( #( #param_names, )* ) = ::poise::parse_slash_args!(
                    ctx.discord, ctx.interaction.guild_id(), ctx.interaction.channel_id(), args =>
                    #( (#param_names: #param_types), )*
                ).await?;

                inner(ctx.into(), #( #param_names, )*).await
            }),
            id: std::sync::Arc::clone(&command_id),
            options: #options,
        }
    })
}

pub fn generate_context_menu_command_spec(
    inv: &Invocation,
    name: &str,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    if inv.parameters.len() != 1 {
        return Err(syn::Error::new(
            inv.function.sig.inputs.span(),
            "Context menu commands require exactly one parameter",
        )
        .into());
    }

    let param_type = &inv.parameters[0].type_;

    let options = generate_options(inv);
    Ok(quote::quote! {
        ::poise::ContextMenuCommand {
            name: #name,
            action: <#param_type as ::poise::ContextMenuParameter<_, _>>::to_action(|ctx, value| {
                Box::pin(async move { inner(ctx.into(), value).await })
            }),
            id: std::sync::Arc::clone(&command_id),
            options: #options,
        }
    })
}
