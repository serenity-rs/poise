use syn::spanned::Spanned as _;

use super::{wrap_option, Invocation};

pub fn generate_prefix_command_spec(
    inv: &Invocation,
) -> Result<proc_macro2::TokenStream, darling::Error> {
    let description = wrap_option(inv.description);
    let explanation = match &inv.more.explanation_fn {
        Some(explanation_fn) => quote::quote! { Some(#explanation_fn) },
        None => match &inv.explanation {
            Some(extracted_explanation) => quote::quote! { Some(|| #extracted_explanation.into()) },
            None => quote::quote! { None },
        },
    };

    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.more.check {
        Some(check) => {
            quote::quote! { Some(|ctx| Box::pin(#check(ctx.into()))) }
        }
        None => quote::quote! { None },
    };
    let on_error = match &inv.more.on_error {
        Some(on_error) => {
            quote::quote! { Some(|err, ctx| Box::pin(#on_error(err, ctx.into()))) }
        }
        None => quote::quote! { None },
    };

    let wildcard_arg = if inv.more.discard_spare_arguments {
        Some(quote::quote! { #[rest] (String), })
    } else {
        None
    };

    let param_specs =
        inv.parameters
            .iter()
            .map(|p| {
                enum Modifier {
                    None,
                    Lazy,
                    Flag,
                    Rest,
                }

                let modifier =
                    match (p.more.lazy, p.more.rest, p.more.flag) {
                        (false, false, false) => Modifier::None,
                        (true, false, false) => Modifier::Lazy,
                        (false, true, false) => Modifier::Rest,
                        (false, false, true) => Modifier::Flag,
                        _ => return Err(syn::Error::new(
                            p.span,
                            "modifiers like #[lazy] or #[rest] currently cannot be used together",
                        )
                        .into()),
                    };

                let type_ = &p.type_;
                Ok(match modifier {
                    Modifier::Flag => {
                        if p.type_ != syn::parse_quote! { bool } {
                            return Err(
                                syn::Error::new(p.type_.span(), "Must use bool for flags").into()
                            );
                        }
                        let literal = proc_macro2::Literal::string(&p.name.to_string());
                        quote::quote! { #[flag] (#literal) }
                    }
                    Modifier::Lazy => quote::quote! { #[lazy] (#type_) },
                    Modifier::Rest => quote::quote! { #[rest] (#type_) },
                    Modifier::None => quote::quote! { (#type_) },
                })
            })
            .collect::<Result<Vec<_>, darling::Error>>()?;

    // Currently you can either fallback to framework setting or opt-in with zero delay. Rest of the
    // cases are not covered because I don't know how the syntax should look like
    let broadcast_typing = match inv.more.broadcast_typing {
        Some(()) => quote::quote! {
            Some(::poise::BroadcastTypingBehavior::WithDelay(std::time::Duration::from_secs(0)))
        },
        None => quote::quote! {
            None
        },
    };

    let command_name = &inv.command_name;
    let track_edits = inv.more.track_edits;
    let aliases = &inv.more.aliases.0;
    let hide_in_help = &inv.more.hide_in_help;
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let required_permissions = inv.required_permissions;
    let owners_only = inv.more.owners_only;
    Ok(quote::quote! {
        ::poise::PrefixCommand {
            name: #command_name,
            action: |ctx, args| Box::pin(async move {
                let ( #( #param_names, )* .. ) = ::poise::parse_prefix_args!(
                    ctx.discord, ctx.msg, args =>
                    #( #param_specs, )*
                    #wildcard_arg
                ).await?;
                inner(ctx.into(), #( #param_names, )* ).await
            }),
            options: ::poise::PrefixCommandOptions {
                track_edits: #track_edits,
                broadcast_typing: #broadcast_typing,
                aliases: &[ #( #aliases, )* ],
                inline_help: #description,
                multiline_help: #explanation,
                check: #check,
                on_error: #on_error,
                hide_in_help: #hide_in_help,
                required_permissions: #required_permissions,
                owners_only: #owners_only,
            }
        }
    })
}
