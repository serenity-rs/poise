//! Implements the #[derive(Modal)] derive macro

use proc_macro::TokenStream;

/// Representation of the struct attributes
#[derive(Debug, Default, darling::FromMeta)]
#[darling(allow_unknown_fields, default)]
struct StructAttributes {
    name: Option<String>,
}

/// Representation of the struct field attributes
#[derive(Debug, Default, darling::FromMeta)]
#[darling(allow_unknown_fields, default)]
struct FieldAttributes {
    name: Option<String>,
    placeholder: Option<String>,
    min_length: Option<u64>,
    max_length: Option<u64>,
    paragraph: Option<()>,
}

pub fn modal(input: syn::DeriveInput) -> Result<TokenStream, darling::Error> {
    let fields = match input.data {
        syn::Data::Struct(syn::DataStruct {
            fields: syn::Fields::Named(fields),
            ..
        }) => fields.named,
        _ => {
            return Err(syn::Error::new(
                input.ident.span(),
                "Only structs with named fields can be used for choice parameters",
            )
            .into())
        }
    };

    let struct_attrs = input
        .attrs
        .iter()
        .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
        .collect::<Result<Vec<_>, _>>()?;
    let struct_attrs = <StructAttributes as darling::FromMeta>::from_list(&struct_attrs)?;

    let mut builders = Vec::new();
    let mut parsers = Vec::new();
    for field in fields {
        // Extract data from syn::Field
        let field_attrs = field
            .attrs
            .iter()
            .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
            .collect::<Result<Vec<_>, _>>()?;
        let field_attrs = <FieldAttributes as darling::FromMeta>::from_list(&field_attrs)?;
        let field_ident = field.ident.unwrap();

        // Create modal builder code for this field
        let label = field_attrs.name.unwrap_or(field_ident.to_string());
        let placeholder = field_attrs.placeholder.into_iter();
        let required = crate::util::extract_type_parameter("Option", &field.ty).is_none();
        let style = if field_attrs.paragraph.is_some() {
            quote::quote!(serenity::InputTextStyle::Paragraph)
        } else {
            quote::quote!(serenity::InputTextStyle::Short)
        };
        let min_length = field_attrs.min_length.into_iter();
        let max_length = field_attrs.max_length.into_iter();
        builders.push(quote::quote! {
            .create_action_row(|b| b
                .create_input_text(|b| b
                    .label(#label)
                    #( .placeholder(#placeholder) )*
                    .required(#required)
                    .custom_id(stringify!(#field_ident))
                    .style(#style)
                    #( .min_length(#min_length) )*
                    #( .max_length(#max_length) )*
                )
            )
        });

        // Create modal parser code for this field
        let ok_or = if required {
            let error = format!("missing {}", field_ident);
            Some(quote::quote! { .ok_or(#error)? })
        } else {
            None
        };
        parsers.push(quote::quote! {
            #field_ident: poise::find_modal_text(&mut data, stringify!(#field_ident)) #ok_or,
        });
    }

    let modal_title = struct_attrs.name.unwrap_or(input.ident.to_string());
    let struct_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote::quote! { const _: () = {
        use poise::serenity_prelude as serenity;
        impl #impl_generics poise::Modal for #struct_ident #ty_generics #where_clause {
            fn create() -> serenity::CreateInteractionResponse<'static> {
                let mut b = serenity::CreateInteractionResponse::default();
                b.kind(serenity::InteractionResponseType::Modal);
                b.interaction_response_data(|b| {
                    b.custom_id("0").title(#modal_title).components(|b| b #( #builders )* )
                });
                b
            }

            fn parse(mut data: serenity::ModalSubmitInteractionData) -> ::std::result::Result<Self, &'static str> {
                Ok(Self { #( #parsers )* })
            }
        }
    }; }
    .into())
}
