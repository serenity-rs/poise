use super::Invocation;
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
            let literal = match &p.name {
                Some(name) => proc_macro2::Literal::string(&name.to_string()),
                _ => {
                    return Err(syn::Error::new(p.span, "#[flag] requires a parameter name").into())
                }
            };
            quote::quote! { #[flag] (#literal) }
        }
        Modifier::Lazy => quote::quote! { #[lazy] (#type_) },
        Modifier::Rest => quote::quote! { #[rest] (#type_) },
        Modifier::None => quote::quote! { (#type_) },
    })
}

pub fn generate_prefix_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_names: Vec<syn::Ident> = inv
        .parameters
        .iter()
        .enumerate()
        .map(|(i, p)| match &p.name {
            Some(x) => x.clone(),
            // Generate a synthetic variable name for command parameters without a name
            None => syn::Ident::new(
                &format!("non_ident_param_{}", i),
                proc_macro2::Span::mixed_site(),
            ),
        })
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
            let ( #( #param_names, )* .. ) = ::poise::parse_prefix_args!(
                ctx.serenity_context, ctx.msg, ctx.args, 0 =>
                #( #param_specs, )*
                #wildcard_arg
            ).await.map_err(|(error, input)| poise::FrameworkError::new_argument_parse(
                ctx.into(),
                input,
                error,
            ))?;

            if !ctx.framework.options.manual_cooldowns {
                ctx.command.cooldowns.lock().unwrap().start_cooldown(ctx.into());
            }

            inner(ctx.into(), #( #param_names, )* )
                .await
                .map_err(|error| poise::FrameworkError::new_command(
                    ctx.into(),
                    error,
                ))
        })
    })
}
