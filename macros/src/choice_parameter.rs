//! Implements the #[derive(ChoiceParameter)] derive macro

use proc_macro::TokenStream;
use syn::spanned::Spanned as _;

/// Representation of the function parameter attribute arguments
#[derive(Debug, darling::FromMeta)]
#[darling(allow_unknown_fields)]
struct VariantAttribute {
    #[darling(multiple)]
    name: Vec<String>,
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
    let mut display_strings: Vec<String> = Vec::new();
    let mut more_display_strings = Vec::new();

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
        let mut names = <VariantAttribute as darling::FromMeta>::from_list(&attrs)?.name;

        let main_name = if names.is_empty() {
            variant.ident.to_string()
        } else {
            names.remove(0)
        };

        variant_idents.push(variant.ident);
        display_strings.push(main_name);
        more_display_strings.push(names);
    }

    let enum_ident = &input.ident;
    let indices1 = 0_u64..(variant_idents.len() as _);
    let indices2 = 0_i32..(variant_idents.len() as _);
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
                    #( #indices1 => Ok(Self::#variant_idents), )*
                    _ => Err(poise::SlashArgError::CommandStructureMismatch("out of bounds choice key")),
                }
            }

            fn create(builder: &mut poise::serenity_prelude::CreateApplicationCommandOption) {
                builder
                    .kind(poise::serenity_prelude::CommandOptionType::Integer)
                    #( .add_int_choice(#display_strings, #indices2 as i32) )* ;
            }
        }

        impl std::str::FromStr for #enum_ident {
            type Err = poise::InvalidChoice;

            fn from_str(s: &str) -> ::std::result::Result<Self, Self::Err> {
                #(
                    if s.eq_ignore_ascii_case(#display_strings)
                        #( || s.eq_ignore_ascii_case(#more_display_strings) )*
                    {
                        Ok(Self::#variant_idents)
                    } else
                )* {
                    Err(poise::InvalidChoice)
                }
            }
        }
    }
    .into())
}
