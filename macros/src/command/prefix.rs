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
            let literal = proc_macro2::Literal::string(&p.name.to_string());
            quote::quote! { #[flag] (#literal) }
        }
        Modifier::Lazy => quote::quote! { #[lazy] (#type_) },
        Modifier::Rest => quote::quote! { #[rest] (#type_) },
        Modifier::None => quote::quote! { (#type_) },
    })
}

pub fn generate_prefix_action(inv: &Invocation) -> Result<proc_macro2::TokenStream, syn::Error> {
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
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
        |ctx, args| Box::pin(async move {
            let ( #( #param_names, )* .. ) = ::poise::parse_prefix_args!(
                ctx.discord, ctx.msg, args, 0 =>
                #( #param_specs, )*
                #wildcard_arg
            ).await.map_err(|(error, input)| poise::FrameworkError::ArgumentParse {
                error,
                input,
                ctx: ctx.into(),
            })?;

            inner(ctx.into(), #( #param_names, )* )
                .await
                .map_err(|error| poise::FrameworkError::Command {
                    error,
                    ctx: ctx.into(),
                })
        })
    })
}
