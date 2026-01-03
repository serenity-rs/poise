//! Implements the #[derive(Modal)] derive macro

use proc_macro::TokenStream;

/// Representation of the struct attributes
#[derive(Debug, Default, darling::FromMeta)]
#[darling(allow_unknown_fields, default)]
struct StructAttributes {
    name: Option<String>,
    #[darling(rename = "text")]
    text_display: Option<String>,
}

/// Representation of the struct field attributes
#[derive(Debug, Default, darling::FromMeta)]
#[darling(allow_unknown_fields, default)]
struct FieldAttributes {
    name: Option<String>,
    description: Option<String>,
    placeholder: Option<String>,
    min_length: Option<u16>,
    max_length: Option<u16>,
    paragraph: Option<()>,
    file_upload: Option<()>,
    string_select: Option<crate::util::List<String>>,
    user_select: Option<crate::util::List<String>>,
    role_select: Option<crate::util::List<String>>,
    channel_select: Option<crate::util::List<String>>,
    channel_types: Option<crate::util::List<syn::Ident>>,
    #[darling(rename = "min_items")]
    min_values: Option<u8>,
    #[darling(rename = "max_items")]
    max_values: Option<u8>,
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
                "Only structs with named fields can be used for derived modals",
            )
            .into())
        }
    };

    let struct_attrs: Vec<_> = input
        .attrs
        .into_iter()
        .map(|attr| darling::ast::NestedMeta::Meta(attr.meta))
        .collect();
    let struct_attrs = <StructAttributes as darling::FromMeta>::from_list(&struct_attrs)?;

    let mut builders = Vec::new();
    let mut parsers = Vec::new();

    if let Some(content) = struct_attrs.text_display {
        builders.push(quote::quote! {
            serenity::CreateModalComponent::TextDisplay(serenity::CreateTextDisplay::new(#content)),
        });
    }

    for field in fields {
        // Extract data from syn::Field
        let field_attrs: Vec<_> = field
            .attrs
            .into_iter()
            .map(|attr| darling::ast::NestedMeta::Meta(attr.meta))
            .collect();
        let field_attrs = <FieldAttributes as darling::FromMeta>::from_list(&field_attrs)?;
        let field_ident = field.ident.unwrap();

        // Prepare to create modal builder and parser code for this field
        let label = field_attrs.name.unwrap_or(field_ident.to_string());
        let description = field_attrs.description.into_iter();
        let required = crate::util::extract_type_parameter("Option", &field.ty).is_none();
        let ok_or = if required {
            let error = format!("missing {}", field_ident);
            Some(quote::quote! { .expect(#error) })
        } else {
            None
        };
        let min_values = field_attrs.min_values.into_iter();
        let max_values = field_attrs.max_values.into_iter();

        if field_attrs.file_upload.is_some() as usize
            + field_attrs.string_select.is_some() as usize
            + field_attrs.user_select.is_some() as usize
            + field_attrs.role_select.is_some() as usize
            > 1 {
             return Err(darling::Error::custom("Cannot have multiple component type attributes on a single field"));
            }

        // If field is a file upload component, process and continue
        if field_attrs.file_upload.is_some() {
            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::file_upload(
                        #label,
                        serenity::CreateFileUpload::new(stringify!(#field_ident))
                        .required(#required)
                        #( .min_values(#min_values) )*
                        #( .max_values(#max_values) )*
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_attachments(&mut data, stringify!(#field_ident)) #ok_or,
            });

            continue;
        }

        let placeholder = field_attrs.placeholder.into_iter();

        // Can be removed once the required field is added to Serenity
        if field_attrs.min_values.is_some_and(|min| min == 0) {
            return Err(darling::Error::custom("Min items for select menus must be greater than 0"));
        }

        // If field is a select menu component, process and continue
        let (select_menu_kind, values) = match field_attrs {
            FieldAttributes { string_select: Some(ref string_select), .. } => {
                let strings = &string_select.0;
                (quote::quote! {
                    serenity::CreateSelectMenuKind::String {
                        options: Cow::Owned(vec![
                            #( serenity::CreateSelectMenuOption::new(#strings, #strings) ),*
                        ]),
                    }
                }, strings.len())
            },
            FieldAttributes { user_select: Some(user_select), .. } => {
                let users: Vec<_> = user_select.0.iter().flat_map(|v| v.parse::<u64>()).collect();
                (quote::quote! {
                    serenity::CreateSelectMenuKind::User {
                        default_users: Some(Cow::Owned(vec![
                            #( serenity::UserId::new(#users) ),*
                        ])),
                    }
                }, users.len())
            },
            FieldAttributes { role_select: Some(role_select), .. } => {
                let roles: Vec<_> = role_select.0.iter().flat_map(|v| v.parse::<u64>()).collect();
                (quote::quote! {
                    serenity::CreateSelectMenuKind::Role {
                        default_roles: Some(Cow::Owned(vec![
                            #( serenity::RoleId::new(#roles) ),*
                        ])),
                    }
                }, roles.len())
            },
            FieldAttributes { channel_select: Some(ref channel_select), .. } => {
                let channel_types = match &field_attrs.channel_types {
                    Some(crate::util::List(channel_types)) => {
                        quote::quote! {
                            Some(Cow::Borrowed(&[ #( serenity::ChannelType::#channel_types ),* ]))
                        }
                    },
                    None => quote::quote! { None },
                };
                let channels: Vec<_> = channel_select.0.iter().flat_map(|v| v.parse::<u64>()).collect();
                (quote::quote! {
                    serenity::CreateSelectMenuKind::Channel {
                        channel_types: #channel_types,
                        default_channels: Some(Cow::Owned(vec![
                            #( serenity::GenericChannelId::new(#channels) ),*
                        ])),
                    }
                }, channels.len())
            },
            _ => (quote::quote! {}, 0)
        };

        match field_attrs.max_values {
            Some(max_values) if field_attrs.string_select.is_some() => { if usize::from(max_values) > values {
                return Err(darling::Error::custom("Max items for string select menus must not be greater than the number of options provided"));
            } },
            Some(max_values) => { if values > usize::from(max_values) {
                return Err(darling::Error::custom("Max items for non-string select menus must not be less than the number of default values provided"));
            } },
            None => {},
        }

        if !select_menu_kind.is_empty() {
            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::select_menu(
                        #label,
                        serenity::CreateSelectMenu::new(
                            stringify!(#field_ident),
                            #select_menu_kind
                        )
                        #( .placeholder(#placeholder) )*
                        #( .min_values(#min_values) )*
                        #( .max_values(#max_values) )*
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_selections(&mut data, stringify!(#field_ident)) #ok_or,
            });

            continue;
        }

        // Field was not upload or select menu, so process as text input
        let style = if field_attrs.paragraph.is_some() {
            quote::quote!(serenity::InputTextStyle::Paragraph)
        } else {
            quote::quote!(serenity::InputTextStyle::Short)
        };
        let min_length = field_attrs.min_length.into_iter();
        let max_length = field_attrs.max_length.into_iter();
        builders.push(quote::quote! {
            serenity::CreateModalComponent::Label(
                serenity::CreateLabel::input_text(
                    #label, 
                    {
                        let mut b = serenity::CreateInputText::new(#style, stringify!(#field_ident));
                        if let Some(defaults) = &mut defaults {
                            // Can use `defaults.#field_ident` directly in Edition 2021 due to more
                            // specific closure capture rules
                            let default = std::mem::take(&mut defaults.#field_ident);
                            // Option::from().unwrap_or_default() dance to handle both T and Option<T>
                            b = b.value(Option::from(default).unwrap_or_else(String::new));
                        }
                        b
                        #( .placeholder(#placeholder) )*
                        .required(#required)
                        #( .min_length(#min_length) )*
                        #( .max_length(#max_length) )*
                    }
                )
                #( .description(#description) )*
            ),
        });

        // Create modal parser code for this field
        parsers.push(quote::quote! {
            #field_ident: poise::find_modal_text(&mut data, stringify!(#field_ident)) #ok_or,
        });
    }

    let modal_title = struct_attrs.name.unwrap_or(input.ident.to_string());
    let struct_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote::quote! { const _: () = {
        use std::borrow::Cow;
        use poise::serenity_prelude as serenity;
        impl #impl_generics poise::Modal for #struct_ident #ty_generics #where_clause {
            fn create(mut defaults: Option<Self>, custom_id: String) -> serenity::CreateInteractionResponse<'static> {
                serenity::CreateInteractionResponse::Modal(
                    serenity::CreateModal::new(custom_id, #modal_title).components(vec!{#( #builders )*})
                )
            }

            fn parse(mut data: serenity::ModalInteractionData) -> Self {
                Self { #( #parsers )* }
            }
        }
    }; }
    .into())
}
