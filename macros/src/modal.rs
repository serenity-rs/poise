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
    value: Option<String>,
    text_display: Option<String>,
    file_upload: Option<()>,
    string_select: Option<crate::util::List<String>>,
    user_select: Option<crate::util::List<String>>,
    role_select: Option<crate::util::List<String>>,
    mentionable_select: Option<crate::util::List<String>>,
    channel_select: Option<crate::util::List<String>>,
    channel_types: Option<crate::util::List<syn::Ident>>,
    #[darling(rename = "min_items")]
    min_values: darling::util::SpannedValue<Option<u8>>,
    #[darling(rename = "max_items")]
    max_values: darling::util::SpannedValue<Option<u8>>,
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
        if builders.len() > 5 {
            let err = "Cannot have more than five components in a modal";
            return Err(darling::Error::custom(err));
        }
        builders.push(quote::quote! {
            serenity::CreateModalComponent::TextDisplay(serenity::CreateTextDisplay::new(#content)),
        });
    }

    for field in fields {
        // Extract data from syn::Field
        let attrs: Vec<_> = field
            .attrs
            .into_iter()
            .map(|attr| darling::ast::NestedMeta::Meta(attr.meta))
            .collect();
        let field_attrs = <FieldAttributes as darling::FromMeta>::from_list(&attrs)?;
        let field_ident = field.ident.unwrap();

        // Allow a text display component to be placed above any field
        if let Some(content) = field_attrs.text_display {
            builders.push(quote::quote! {
                serenity::CreateModalComponent::TextDisplay(serenity::CreateTextDisplay::new(#content)),
            });
        }

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
            + field_attrs.mentionable_select.is_some() as usize
            + field_attrs.channel_select.is_some() as usize
            > 1
        {
            let err = "Cannot have multiple input component attributes on a single field";
            return Err(darling::Error::custom(err).with_span(&field_ident));
        }

        if required && field_attrs.min_values.is_some_and(|min| min == 0) {
            let err = "Value of `min_items` must be greater than 0 for required components";
            return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
        }
        if let Some(max) = *field_attrs.max_values {
            if field_attrs.min_values.is_some_and(|min| min > max) {
                let err =
                    "Value of `min_items` should be less than or equal to that of `max_items`";
                return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
            }
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

        // If field is a select menu component, process and continue
        let (select_menu_kind, values, kind) = match field_attrs {
            FieldAttributes {
                string_select: Some(ref string_select),
                ..
            } => {
                let strings = &string_select.0;
                (
                    quote::quote! {
                        serenity::CreateSelectMenuKind::String {
                            options: Cow::Owned(vec![
                                #( serenity::CreateSelectMenuOption::new(#strings, #strings) ),*
                            ]),
                        }
                    },
                    strings.len(),
                    quote::quote! { .strings },
                )
            }
            FieldAttributes {
                user_select: Some(user_select),
                ..
            } => {
                let users: Vec<_> = user_select
                    .0
                    .iter()
                    .flat_map(|s| s.parse::<u64>())
                    .collect();
                (
                    quote::quote! {
                        serenity::CreateSelectMenuKind::User {
                            default_users: Some(Cow::Owned(vec![
                                #( serenity::UserId::new(#users) ),*
                            ])),
                        }
                    },
                    users.len(),
                    quote::quote! { .users },
                )
            }
            FieldAttributes {
                role_select: Some(role_select),
                ..
            } => {
                let roles: Vec<_> = role_select
                    .0
                    .iter()
                    .flat_map(|s| s.parse::<u64>())
                    .collect();
                (
                    quote::quote! {
                        serenity::CreateSelectMenuKind::Role {
                            default_roles: Some(Cow::Owned(vec![
                                #( serenity::RoleId::new(#roles) ),*
                            ])),
                        }
                    },
                    roles.len(),
                    quote::quote! { .roles },
                )
            }
            FieldAttributes {
                mentionable_select: Some(mentionables),
                ..
            } => {
                let mentionables = mentionables.0;
                let mut users = Vec::new();
                let mut roles = Vec::new();
                for string in mentionables {
                    if string.starts_with("<@&") && string.ends_with(">") {
                        if let Some(id) = string
                            .trim_start_matches("<@&")
                            .trim_end_matches(">")
                            .parse::<u64>()
                            .ok()
                        {
                            roles.push(id);
                        }
                    } else if string.starts_with("<@") && string.ends_with(">") {
                        if let Some(id) = string
                            .trim_start_matches("<@")
                            .trim_end_matches(">")
                            .parse::<u64>()
                            .ok()
                        {
                            users.push(id);
                        }
                    }
                }
                let default_users = quote::quote! { #( serenity::UserId::new( #users ) ),* };
                let default_roles = quote::quote! { #( serenity::RoleId::new( #roles ) ),* };
                (
                    quote::quote! {
                        serenity::CreateSelectMenuKind::Mentionable {
                            default_users: Some(Cow::Owned(vec![#default_users])),
                            default_roles: Some(Cow::Owned(vec![#default_roles])),
                        }
                    },
                    users.len() + roles.len(),
                    quote::quote! { .mentionables },
                )
            }
            FieldAttributes {
                channel_select: Some(ref channel_select),
                ..
            } => {
                let channel_types = match &field_attrs.channel_types {
                    Some(crate::util::List(channel_types)) => {
                        quote::quote! {
                            Some(Cow::Borrowed(&[ #( serenity::ChannelType::#channel_types ),* ]))
                        }
                    }
                    None => quote::quote! { None },
                };
                let channels: Vec<_> = channel_select
                    .0
                    .iter()
                    .flat_map(|v| v.parse::<u64>())
                    .collect();
                (
                    quote::quote! {
                        serenity::CreateSelectMenuKind::Channel {
                            channel_types: #channel_types,
                            default_channels: Some(Cow::Owned(vec![
                                #( serenity::GenericChannelId::new(#channels) ),*
                            ])),
                        }
                    },
                    channels.len(),
                    quote::quote! { .channels },
                )
            }
            _ => (quote::quote! {}, 0, quote::quote! {}),
        };

        if field_attrs.string_select.is_some()
            && field_attrs
                .max_values
                .is_some_and(|v| usize::from(v) > values)
        {
            let err = "Value of `max_items` cannot be greater than the number of options provided";
            return Err(darling::Error::custom(err).with_span(&field_attrs.max_values.span()));
        }
        if field_attrs.string_select.is_none()
            && field_attrs
                .max_values
                .is_some_and(|v| usize::from(v) < values)
        {
            let err =
                "Value of `max_items` cannot be less than the number of default values provided";
            return Err(darling::Error::custom(err).with_span(&field_attrs.max_values.span()));
        }
        if field_attrs.string_select.is_none() && field_attrs.max_values.is_none() && values > 0 {
            let err = "`max_items` must be set to equal to or greater than the number of default values provided";
            return Err(darling::Error::custom(err).with_span(&field_ident));
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
                        .required(#required)
                        #( .min_values(#min_values) )*
                        #( .max_values(#max_values) )*
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_selections(&mut data, stringify!(#field_ident)) #kind #ok_or,
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
        let value = field_attrs.value.into_iter();

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
                        #( .value(#value) )*
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

    if builders.len() > 5 {
        let err = "Cannot have more than five components in a modal";
        return Err(darling::Error::custom(err));
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
