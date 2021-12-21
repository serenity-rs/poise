use syn::spanned::Spanned as _;

use super::Invocation;

pub fn generate_prefix_command_spec(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let wildcard_arg = if inv.more.discard_spare_arguments {
        Some(quote::quote! { #[rest] (String), })
    } else {
        None
    };

    let param_specs = inv
        .parameters
        .iter()
        .map(quote_parameter)
        .collect::<Result<Vec<_>, darling::Error>>()?;

    let command_name = &inv.command_name;
    let track_edits = inv.more.track_edits;
    let broadcast_typing = inv.more.broadcast_typing;
    let aliases = &inv.more.aliases.0;
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    Ok(quote::quote! {
        ::poise::PrefixCommand {
            name: #command_name,
            action: |ctx, args| Box::pin(async move {
                let ( #( #param_names, )* .. ) = ::poise::parse_prefix_args!(
                    ctx.discord, ctx.msg, args =>
                    #( #param_specs, )*
                    #wildcard_arg
                ).await.map_err(|error| poise::FrameworkError::ArgumentParse {
                    ctx: ctx.into(),
                    error: error.0
                })?;

                inner(ctx.into(), #( #param_names, )* )
                    .await
                    .map_err(|error| poise::FrameworkError::Command {
                        error,
                        location: poise::CommandErrorLocation::Body,
                        ctx: ctx.into(),
                    })
            }),
            id: std::sync::Arc::clone(&command_id),
            track_edits: #track_edits,
            broadcast_typing: #broadcast_typing,
            aliases: &[ #( #aliases, )* ],
        }
    })
}

fn quote_parameter(
    p: &super::CommandParameter,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    enum Modifier {
        None,
        Lazy,
        Flag,
        Rest,
    }
    let modifier = match (p.more.lazy, p.more.rest, p.more.flag) {
        (false, false, false) => Modifier::None,
        (true, false, false) => Modifier::Lazy,
        (false, true, false) => Modifier::Rest,
        (false, false, true) => Modifier::Flag,
        _ => {
            return Err(syn::Error::new(
                p.span,
                "modifiers like #[lazy] or #[rest] currently cannot be used together",
            )
            .into())
        }
    };
    let type_ = &p.type_;
    Ok(match modifier {
        Modifier::Flag => {
            if p.type_ != syn::parse_quote! { bool } {
                return Err(syn::Error::new(p.type_.span(), "Must use bool for flags").into());
            }
            let literal = proc_macro2::Literal::string(&p.name.to_string());
            quote::quote! { #[flag] (#literal) }
        }
        Modifier::Lazy => quote::quote! { #[lazy] (#type_) },
        Modifier::Rest => quote::quote! { #[rest] (#type_) },
        Modifier::None => quote::quote! { (#type_) },
    })
}
