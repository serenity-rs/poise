use super::Invocation;
use quote::format_ident;
use syn::spanned::Spanned as _;

fn quote_parameter(p: &super::CommandParameter) -> Result<proc_macro2::TokenStream, syn::Error> {
    enum Modifier {
        None,
        Lazy,
        Flag,
        Rest,
    }
    let modifier = match (p.args.lazy, p.args.rest, p.args.flag) {
        (false, false, false) => Modifier::None,
        (true, false, false) => Modifier::Lazy,
        (false, true, false) => Modifier::Rest,
        (false, false, true) => Modifier::Flag,
        _ => {
            let message = "modifiers like #[lazy] or #[rest] currently cannot be used together";
            return Err(syn::Error::new(p.span, message));
        }
    };
    let type_ = &p.type_;
    Ok(match modifier {
        Modifier::Flag => {
            if p.type_ != syn::parse_quote! { bool } {
                return Err(syn::Error::new(p.type_.span(), "Must use bool for flags"));
            }
            // TODO: doesn't work for r#keywords :( I cant be arsed to fix this rn because basically
            // nobody uses this feature anyways and I'd have to go change the macro_rules macro to
            // not accept non-quoted idents anymore
            let literal = proc_macro2::Literal::string(&p.name);
            quote::quote! { #[flag] (#literal) }
        }
        Modifier::Lazy => quote::quote! { #[lazy] (#type_) },
        Modifier::Rest => quote::quote! { #[rest] (#type_) },
        Modifier::None => quote::quote! { (#type_) },
    })
}

pub fn generate_prefix_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_idents = (0..inv.parameters.len())
        .map(|i| format_ident!("poise_param_{i}"))
        .collect::<Vec<_>>();
    let param_specs = inv
        .parameters
        .iter()
        .map(quote_parameter)
        .collect::<Result<Vec<_>, syn::Error>>()?;
    let wildcard_arg = match inv.args.discard_spare_arguments {
        true => Some(quote::quote! { #[rest] (Option<String>), }),
        false => None,
    };

    Ok(quote::quote! {
        |ctx| Box::pin(async move {
            let ( #( #param_idents, )* .. ) = ::poise::parse_prefix_args!(
                ctx.serenity_context(), ctx.msg, ctx.args, 0 =>
                #( #param_specs, )*
                #wildcard_arg
            ).await.map_err(|(error, input)| poise::FrameworkError::new_argument_parse(
                ctx.into(),
                input,
                error,
            ))?;

            if !ctx.framework.options.manual_cooldowns {
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
