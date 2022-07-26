//! Implements the #[derive(ChoiceParameter)] derive macro

use proc_macro::TokenStream;
use syn::spanned::Spanned as _;

/// Representation of the function parameter attribute arguments
#[derive(Debug, darling::FromMeta)]
#[darling(allow_unknown_fields)]
struct VariantAttribute {
    // Note to self: when adding an attribute here, add it to #[proc_macro_derive]!
    #[darling(multiple)]
    name: Vec<String>,
    #[darling(multiple)]
    name_localized: Vec<crate::util::Tuple2<String>>,
}

pub fn choice_parameter(input: syn::DeriveInput) -> Result<TokenStream, darling::Error> {
    let enum_ = match input.data {
        syn::Data::Enum(x) => x,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "Only enums can be used for choice parameters",
            )
            .into())
        }
    };

    let mut variant_idents: Vec<proc_macro2::Ident> = Vec::new();
    let mut names: Vec<String> = Vec::new();
    let mut alternative_names = Vec::new();
    let mut locales: Vec<Vec<String>> = Vec::new();
    let mut localized_names: Vec<Vec<String>> = Vec::new();

    for variant in enum_.variants {
        if !matches!(&variant.fields, syn::Fields::Unit) {
            return Err(syn::Error::new(
                variant.fields.span(),
                "Choice parameters cannot have fields",
            )
            .into());
        }

        let attrs = variant
            .attrs
            .into_iter()
            .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
            .collect::<Result<Vec<_>, _>>()?;
        let mut attrs = <VariantAttribute as darling::FromMeta>::from_list(&attrs)?;

        let main_name = if attrs.name.is_empty() {
            variant.ident.to_string()
        } else {
            attrs.name.remove(0)
        };

        variant_idents.push(variant.ident);
        names.push(main_name);
        alternative_names.push(attrs.name);

        let (a, b) = attrs.name_localized.into_iter().map(|x| (x.0, x.1)).unzip();
        locales.push(a);
        localized_names.push(b);
    }

    let enum_ident = &input.ident;
    let indices = 0_u64..(variant_idents.len() as _);
    Ok(quote::quote! {
        #[poise::async_trait]
        impl poise::SlashArgument for #enum_ident {
            async fn extract(
                _: &poise::serenity_prelude::Context,
                _: poise::ApplicationCommandOrAutocompleteInteraction<'_>,
                value: &poise::serenity::json::Value,
            ) -> ::std::result::Result<Self, poise::SlashArgError> {
                use poise::serenity_prelude::json::prelude::*;
                let choice_key = value
                    .as_u64()
                    .ok_or(poise::SlashArgError::CommandStructureMismatch(
                        "expected u64",
                    ))?;

                match choice_key {
                    #( #indices => Ok(Self::#variant_idents), )*
                    _ => Err(poise::SlashArgError::CommandStructureMismatch("out of bounds choice key")),
                }
            }

            fn create(builder: &mut poise::serenity_prelude::CreateApplicationCommandOption) {
                builder.kind(poise::serenity_prelude::CommandOptionType::Integer); 
            }

            fn choices() -> Vec<poise::CommandParameterChoice> {
                vec![ #( poise::CommandParameterChoice {
                    name: #names.to_string(),
                    localizations: std::collections::HashMap::from([
                        #( (#locales.to_string(), #localized_names.to_string()) )*
                    ]),
                }, )* ]
            }
        }

        impl std::str::FromStr for #enum_ident {
            type Err = poise::InvalidChoice;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                #(
                    if s.eq_ignore_ascii_case(#names)
                        #( || s.eq_ignore_ascii_case(#alternative_names) )*
                    {
                        Ok(Self::#variant_idents)
                    } else
                )* {
                    Err(poise::InvalidChoice)
                }
            }
        }

        impl std::fmt::Display for #enum_ident {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(self.name())
            }
        }

        impl #enum_ident {
            /// Returns the non-localized name of this choice
            pub fn name(&self) -> &'static str {
                match self {
                    #( Self::#variant_idents => #names, )*
                }
            }

            /// Returns the localized name for the given locale, if one is set
            pub fn localized_name(&self, locale: &str) -> Option<&'static str> {
                match self {
                    #( Self::#variant_idents => match locale {
                        #( #locales => Some(#localized_names), )*
                        _ => None,
                    }, )*
                }
            }
        }
    }
    .into())
}
