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
    let indices = 0..variant_idents.len();
    Ok(quote::quote! {
        impl poise::ChoiceParameter for #enum_ident {
            fn list() -> Vec<poise::CommandParameterChoice> {
                vec![ #( poise::CommandParameterChoice {
                    name: #names.to_string(),
                    localizations: std::collections::HashMap::from([
                        #( (#locales.to_string(), #localized_names.to_string()) ),*
                    ]),
                    __non_exhaustive: (),
                }, )* ]
            }

            fn from_index(index: usize) -> Option<Self> {
                match index {
                    #( #indices => Some(Self::#variant_idents), )*
                    _ => None,
                }
            }

            fn from_name(name: &str) -> Option<Self> {
                #( if name.eq_ignore_ascii_case(#names)
                    #( || name.eq_ignore_ascii_case(#alternative_names) )*
                {
                    return Some(Self::#variant_idents);
                } )*
                None
            }

            fn name(&self) -> &'static str {
                match self {
                    #( Self::#variant_idents => #names, )*
                }
            }

            fn localized_name(&self, locale: &str) -> Option<&'static str> {
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
