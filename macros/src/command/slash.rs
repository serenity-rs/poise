use syn::spanned::Spanned as _;

use super::{extract_option_type, extract_vec_type, wrap_option, Invocation};

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

    let mut parameter_builders = Vec::new();
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
        parameter_builders.push((
            quote::quote! {
                |o| (&&&&&std::marker::PhantomData::<#type_>).create(o)
                    .required(#required)
                    .name(stringify!(#param_name))
                    .description(#description)
            },
            required,
        ));
    }
    // Sort the parameters so that optional parameters come last - Discord requires this order
    parameter_builders.sort_by_key(|(_, required)| !required);
    let parameter_builders = parameter_builders
        .into_iter()
        .map(|(builder, _)| builder)
        .collect::<Vec<_>>();

    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.more.check {
        Some(check) => quote::quote! { Some(|ctx| Box::pin(#check(::poise::Context::Slash(ctx)))) },
        None => quote::quote! { None },
    };
    let on_error = match &inv.more.on_error {
        Some(on_error) => quote::quote! {
            Some(|err, ctx| Box::pin(#on_error(err, ::poise::CommandErrorContext::Slash(ctx))))
        },
        None => quote::quote! { None },
    };

    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv
        .parameters
        .iter()
        .map(|p| match p.more.flag {
            true => syn::parse_quote! { FLAG },
            false => p.type_.clone(),
        })
        .collect::<Vec<_>>();
    let defer_response = wrap_option(inv.more.defer_response);
    let ephemeral = inv.more.ephemeral;
    let required_permissions = inv.required_permissions;
    let owners_only = inv.more.owners_only;
    Ok(quote::quote! {
        ::poise::SlashCommand {
            kind: ::poise::SlashCommandKind::ChatInput {
                name: #command_name,
                description: #description,
                parameters: {
                    use ::poise::SlashArgumentHack;
                    vec![ #( #parameter_builders, )* ]
                },
                action: |ctx, args| Box::pin(async move {
                    // idk why this can't be put in the macro itself (where the lint is triggered) and
                    // why clippy doesn't turn off this lint inside macros in the first place
                    #[allow(clippy::needless_question_mark)]

                    let ( #( #param_names, )* ) = ::poise::parse_slash_args!(
                        ctx.discord, ctx.interaction.guild_id, ctx.interaction.channel_id, args =>
                        #( (#param_names: #param_types), )*
                    ).await?;

                    inner(::poise::Context::Slash(ctx), #( #param_names, )*).await
                }),
            },
            options: ::poise::SlashCommandOptions {
                defer_response: #defer_response,
                check: #check,
                on_error: #on_error,
                ephemeral: #ephemeral,
                required_permissions: #required_permissions,
                owners_only: #owners_only,
            }
        }
    })
}
