//! Implements the #[derive(Modal)] derive macro

use proc_macro::TokenStream;

/// Representation of the struct attributes
#[derive(Debug, Default, darling::FromMeta)]
#[darling(allow_unknown_fields, default)]
struct StructAttributes {
    name: Option<String>,
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
    text_display: Option<String>,
    file_upload: Option<()>,
    file_types: Option<crate::util::List<String>>,
    string_select: Option<crate::util::List<String>>,
    string_select_emojis: Option<crate::util::List<String>>,
    string_select_descriptions: Option<crate::util::List<String>>,
    user_select: Option<()>,
    role_select: Option<()>,
    mentionable_select: Option<()>,
    channel_select: Option<()>,
    channel_types: Option<crate::util::List<syn::Ident>>,
    radio_group: Option<crate::util::List<String>>,
    radio_group_descriptions: Option<crate::util::List<String>>,
    checkbox_group: Option<crate::util::List<String>>,
    checkbox_group_descriptions: Option<crate::util::List<String>>,
    checkbox: Option<()>,
    min_values: darling::util::SpannedValue<Option<u8>>,
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
                "only structs with named fields can be used for derived modals",
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
        // Builder count check appears twice (here and after the final parser is pushed),
        // but this is intentional. Count does not trigger properly otherwise.
        if builders.len() > 5 {
            let err = "cannot have more than five components in a modal";
            return Err(darling::Error::custom(err));
        }
        builders.push(quote::quote! {
            serenity::CreateModalComponent::TextDisplay(
                serenity::CreateTextDisplay::new(#content),
            ),
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

        // Allow a text display component to be placed above any field.
        if let Some(content) = field_attrs.text_display {
            builders.push(quote::quote! {
                serenity::CreateModalComponent::TextDisplay(
                    serenity::CreateTextDisplay::new(#content),
                ),
            });
        }

        // Initialize variables common to most components and do some form validation.
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
            + field_attrs.radio_group.is_some() as usize
            + field_attrs.checkbox_group.is_some() as usize
            + field_attrs.checkbox.is_some() as usize
            > 1
        {
            let err = "cannot have multiple interactive component attributes on a single field";
            return Err(darling::Error::custom(err).with_span(&field_ident));
        }

        if let Some(min) = *field_attrs.min_values {
            if required && min == 0 {
                let err = "value of `min_values` must be greater than 0 for required components";
                return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
            }
            if min > 10
                && (field_attrs.file_upload.is_some() || field_attrs.checkbox_group.is_some())
            {
                let err = "`min_values` must not be greater than 10 for this component";
                return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
            }
            if min > 25
                && (field_attrs.string_select.is_some()
                    || field_attrs.user_select.is_some()
                    || field_attrs.role_select.is_some()
                    || field_attrs.mentionable_select.is_some()
                    || field_attrs.channel_select.is_some())
            {
                let err = "`min_values` must not be greater than 25 for this component";
                return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
            }
        }
        if let Some(max) = *field_attrs.max_values {
            if field_attrs.min_values.is_some_and(|min| min > max) {
                let err =
                    "value of `min_values` must be less than or equal to that of `max_values`";
                return Err(darling::Error::custom(err).with_span(&field_attrs.min_values.span()));
            }
            if max > 10
                && (field_attrs.file_upload.is_some() || field_attrs.checkbox_group.is_some())
            {
                let err = "`max_values` must not be greater than 10 for this component";
                return Err(darling::Error::custom(err).with_span(&field_attrs.max_values.span()));
            }
            if max > 25
                && (field_attrs.string_select.is_some()
                    || field_attrs.user_select.is_some()
                    || field_attrs.role_select.is_some()
                    || field_attrs.mentionable_select.is_some()
                    || field_attrs.channel_select.is_some())
            {
                let err = "`max_values` must not be greater than 25 for this component";
                return Err(darling::Error::custom(err).with_span(&field_attrs.max_values.span()));
            }
        }

        // If field is a file upload component, process and continue.
        if field_attrs.file_upload.is_some() {
            let file_types = match field_attrs.file_types {
                Some(crate::util::List(file_types)) => {
                    if file_types.len() > 10 {
                        let err = "maximum of 10 file types allowed";
                        return Err(err_on_attr(&attrs, err, "file_types"));
                    }
                    quote::quote! { .file_types( &[ #( Cow::Borrowed(#file_types) ),* ] ) }
                }
                None => quote::quote! {},
            };
            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::file_upload(
                        #label,
                        serenity::CreateFileUpload::new(stringify!(#field_ident))
                            .required(#required)
                            #( .min_values(#min_values) )*
                            #( .max_values(#max_values) )*
                            #file_types
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident))
                    .attachments
                    #ok_or,
            });

            continue;
        }

        // If field is a checkbox component, process and continue.
        if field_attrs.checkbox.is_some() {
            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::checkbox(
                        #label,
                        {
                            let mut b = serenity::CreateCheckbox::new(stringify!(#field_ident));
                            if let Some(defaults) = &defaults {
                                b = b.default_selected(defaults.#field_ident);
                            }
                            b
                        }
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident)).checked,
            });

            continue;
        }

        // If field is a radio group component, process and continue.
        if let Some(radio_group) = field_attrs.radio_group {
            let options = radio_group.0;
            if options.len() < 2 {
                let err = "minimum of 2 radio group options required";
                return Err(err_on_attr(&attrs, err, "radio_group"));
            } else if options.len() > 10 {
                let err = "maximum of 10 radio group options allowed";
                return Err(err_on_attr(&attrs, err, "radio_group"));
            }
            let descriptions = field_attrs.radio_group_descriptions.unwrap_or_default().0;
            let create_option = if descriptions.is_empty() {
                quote::quote! {
                    #({
                        let mut b = serenity::CreateRadioGroupOption::new(#options, #options);
                        if !default.is_empty() && default == #options {
                            b = b.default_selection(true);
                        }
                        b
                    }),*
                }
            } else {
                if descriptions.len() < options.len() {
                    let err =
                        "number of descriptions must not be less than the number of radio group options";
                    return Err(err_on_attr(&attrs, err, "radio_group_descriptions"));
                }
                quote::quote! {
                    #({
                        let mut b = serenity::CreateRadioGroupOption::new(#options, #options);
                        if !default.is_empty() && default == #options {
                            b = b.default_selection(true);
                        }
                        b = b.description(#descriptions);
                        b
                    }),*
                }
            };

            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::radio_group(
                        #label,
                        serenity::CreateRadioGroup::new(stringify!(#field_ident), {
                            let default = if let Some(defaults) = &mut defaults {
                                let default = std::mem::take(&mut defaults.#field_ident);
                                Option::from(default).unwrap_or_else(FixedString::new)
                            } else {
                                FixedString::new()
                            };
                            Cow::Owned(vec![#create_option])
                        })
                        .required(#required),
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident))
                    .radio_option
                    #ok_or,
            });

            continue;
        }

        // If field is a checkbox group component, process and continue.
        if let Some(checkbox_group) = field_attrs.checkbox_group {
            let options = checkbox_group.0;
            if options.len() < 1 {
                let err = "minimum of 1 checkbox group option required";
                return Err(err_on_attr(&attrs, err, "checkbox_group"));
            } else if options.len() > 10 {
                let err = "maximum of 10 checkbox group options allowed";
                return Err(err_on_attr(&attrs, err, "checkbox_group"));
            }
            if field_attrs
                .max_values
                .is_some_and(|v| usize::from(v) > options.len())
            {
                let err =
                    "value of `max_values` cannot be greater than the number of options provided";
                return Err(darling::Error::custom(err).with_span(&field_attrs.max_values.span()));
            }
            let descriptions = field_attrs
                .checkbox_group_descriptions
                .unwrap_or_default()
                .0;
            let create_option = if descriptions.is_empty() {
                quote::quote! {
                    #({
                        let mut b = serenity::CreateCheckboxGroupOption::new(#options, #options);
                        if default.as_ref().is_some_and(|default| {
                            default.contains(&#options.to_string())
                        }) {
                            b = b.default_selection(true);
                        }
                        b
                    }),*
                }
            } else {
                if descriptions.len() < options.len() {
                    let err =
                        "number of descriptions must not be less than the number of checkbox group options";
                    return Err(err_on_attr(&attrs, err, "checkbox_group_descriptions"));
                }
                quote::quote! {
                    #({
                        let mut b = serenity::CreateCheckboxGroupOption::new(#options, #options);
                        if default.as_ref().is_some_and(|default| {
                            default.contains(&#options.to_string())
                        }) {
                            b = b.default_selection(true);
                        }
                        b = b.description(#descriptions);
                        b
                    }),*
                }
            };

            builders.push(quote::quote! {
                serenity::CreateModalComponent::Label(
                    serenity::CreateLabel::checkbox_group(
                        #label,
                        serenity::CreateCheckboxGroup::new(stringify!(#field_ident), {
                            let default = if let Some(defaults) = &mut defaults {
                                let default = std::mem::take(&mut defaults.#field_ident);
                                Option::<FixedArray<String>>::from(default)
                            } else {
                                Option::<FixedArray<String>>::None
                            };
                            Cow::Owned(vec![#create_option])
                        })
                        .required(#required)
                        #( .min_values(#min_values) )*
                        #( .max_values(#max_values) )*
                    )
                    #( .description(#description) )*
                ),
            });

            parsers.push(quote::quote! {
                #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident))
                    .checkbox_options
                    #ok_or,
            });

            continue;
        }

        let placeholder = field_attrs.placeholder.into_iter();

        // If field is a select menu component, process and continue.
        let (select_menu_kind, kind) = match field_attrs {
            FieldAttributes {
                string_select: Some(string_select),
                ..
            } => {
                let strings = string_select.0;
                if strings.len() < 1 {
                    let err = "minimum of 1 string select option required";
                    return Err(err_on_attr(&attrs, err, "string_select"));
                } else if strings.len() > 25 {
                    let err = "maximum of 25 string select options allowed";
                    return Err(err_on_attr(&attrs, err, "string_select"));
                }
                if field_attrs
                    .max_values
                    .is_some_and(|v| usize::from(v) > strings.len())
                {
                    let err =
                    "value of `max_values` cannot be greater than the number of options provided";
                    return Err(
                        darling::Error::custom(err).with_span(&field_attrs.max_values.span())
                    );
                }
                let emojis = field_attrs.string_select_emojis.unwrap_or_default().0;
                if !emojis.is_empty() && emojis.len() < strings.len() {
                    let err =
                        "number of emojis must not be less than the number of string select options";
                    return Err(err_on_attr(&attrs, err, "string_select_emojis"));
                }
                let descriptions = field_attrs.string_select_descriptions.unwrap_or_default().0;
                if !descriptions.is_empty() && descriptions.len() < strings.len() {
                    let err =
                        "number of descriptions must not be less than the number of string select options";
                    return Err(err_on_attr(&attrs, err, "string_select_descriptions"));
                }
                let create_option = match (emojis.is_empty(), descriptions.is_empty()) {
                    (true, true) => quote::quote! {
                        #({
                            let mut b = serenity::CreateSelectMenuOption::new(#strings, #strings);
                            if default.as_ref().is_some_and(|default| {
                                default.contains(&#strings.to_string())
                            }) {
                                b = b.default_selection(true);
                            }
                            b
                        }),*
                    },
                    (true, false) => quote::quote! {
                        #({
                            let mut b = serenity::CreateSelectMenuOption::new(#strings, #strings);
                            if default.as_ref().is_some_and(|default| {
                                default.contains(&#strings.to_string())
                            }) {
                                b = b.default_selection(true);
                            }
                            b = b.description(#descriptions);
                            b
                        }),*
                    },
                    (false, true) => quote::quote! {
                        #({
                            let mut b = serenity::CreateSelectMenuOption::new(#strings, #strings);
                            if default.as_ref().is_some_and(|default| {
                                default.contains(&#strings.to_string())
                            }) {
                                b = b.default_selection(true);
                            }
                            if let Ok(emoji) = serenity::ReactionType::try_from(#emojis) {
                                b = b.emoji(emoji);
                            }
                            b
                        }),*
                    },
                    (false, false) => quote::quote! {
                        #({
                            let mut b = serenity::CreateSelectMenuOption::new(#strings, #strings);
                            if default.as_ref().is_some_and(|default| {
                                default.contains(&#strings.to_string())
                            }) {
                                b = b.default_selection(true);
                            }
                            if let Ok(emoji) = serenity::ReactionType::try_from(#emojis) {
                                b = b.emoji(emoji);
                            }
                            b = b.description(#descriptions);
                            b
                        }),*
                    },
                };
                (
                    quote::quote! {
                        {
                            let default = if let Some(defaults) = &mut defaults {
                                let default = std::mem::take(&mut defaults.#field_ident);
                                Option::<FixedArray<String>>::from(default)
                            } else {
                                Option::<FixedArray<String>>::None
                            };
                            serenity::CreateSelectMenuKind::String {
                                options: Cow::Owned(vec![#create_option]),
                            }
                        }
                    },
                    quote::quote! { .strings },
                )
            }
            FieldAttributes {
                user_select: Some(()),
                ..
            } => (
                quote::quote! {
                    {
                        let default_users = if let Some(defaults) = &mut defaults {
                            let default = std::mem::take(&mut defaults.#field_ident);
                            Option::<FixedArray<serenity::User>>::from(default).map(|default| {
                                let default = default.iter().map(|u| u.id).collect::<Vec<_>>();
                                Cow::Owned(default)
                            })
                        } else {
                            None
                        };
                        serenity::CreateSelectMenuKind::User { default_users }
                    }
                },
                quote::quote! { .users },
            ),
            FieldAttributes {
                role_select: Some(()),
                ..
            } => (
                quote::quote! {
                    {
                        let default_roles = if let Some(defaults) = &mut defaults {
                            let default = std::mem::take(&mut defaults.#field_ident);
                            Option::<FixedArray<serenity::Role>>::from(default).map(|default| {
                                let default = default.iter().map(|r| r.id).collect::<Vec<_>>();
                                Cow::Owned(default)
                            })
                        } else {
                            None
                        };
                        serenity::CreateSelectMenuKind::Role { default_roles }
                    }
                },
                quote::quote! { .roles },
            ),
            FieldAttributes {
                mentionable_select: Some(()),
                ..
            } => (
                quote::quote! {
                    {
                        let (default_users, default_roles) =
                            if let Some(defaults) = &mut defaults {
                                let default = std::mem::take(&mut defaults.#field_ident);
                                Option::<Mentionables>::from(default).map_or(
                                    (None, None),
                                    |default| {
                                        let default_users =
                                            default.users.iter().map(|u| u.id).collect::<Vec<_>>();
                                        let default_roles =
                                            default.roles.iter().map(|r| r.id).collect::<Vec<_>>();
                                        (
                                            Some(Cow::Owned(default_users)),
                                            Some(Cow::Owned(default_roles))
                                        )
                                    },
                                )
                            } else {
                                (None, None)
                            };
                        serenity::CreateSelectMenuKind::Mentionable {
                            default_users,
                            default_roles
                        }
                    }
                },
                quote::quote! { .mentionables },
            ),
            FieldAttributes {
                channel_select: Some(()),
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
                (
                    quote::quote! {
                        {
                            let default_channels = if let Some(defaults) = &mut defaults {
                                let default = std::mem::take(&mut defaults.#field_ident);
                                Option::<FixedArray<serenity::GenericChannelId>>::from(default)
                                    .map(|default| Cow::Owned(default.into_vec()))
                            } else {
                                None
                            };
                            serenity::CreateSelectMenuKind::Channel {
                                channel_types: #channel_types,
                                default_channels
                            }
                        }
                    },
                    quote::quote! { .channels },
                )
            }
            _ => (quote::quote! {}, quote::quote! {}),
        };

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
                #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident))
                    #kind
                    #ok_or,
            });

            continue;
        }

        // Field was none of the above, so process as text input (=default).
        let style = if field_attrs.paragraph.is_some() {
            quote::quote!(serenity::InputTextStyle::Paragraph)
        } else {
            quote::quote!(serenity::InputTextStyle::Short)
        };
        let min_length = field_attrs.min_length.into_iter();
        let max_length = field_attrs.max_length.into_iter();
        if field_attrs.min_length.is_some_and(|min| min > 4000) {
            let err = "value of `min_length` must be less than or equal to 4000";
            return Err(err_on_attr(&attrs, err, "min_length"));
        }
        if let Some(max) = field_attrs.max_length {
            if max < 1 {
                let err = "value of `max_length` must be greater than or equal to 1";
                return Err(err_on_attr(&attrs, err, "max_length"));
            }
            if max > 4000 {
                let err = "value of `max_length` must be less than or equal to 4000";
                return Err(err_on_attr(&attrs, err, "max_length"));
            }
            if field_attrs.min_length.is_some_and(|min| min > max) {
                let err =
                    "value of `min_length` must be less than or equal to that of `max_length`";
                return Err(err_on_attr(&attrs, err, "min_length"));
            }
        }

        builders.push(quote::quote! {
            serenity::CreateModalComponent::Label(
                serenity::CreateLabel::input_text(#label, {
                    let mut b = serenity::CreateInputText::new(#style, stringify!(#field_ident));
                    if let Some(defaults) = &mut defaults {
                        // Can use `defaults.#field_ident` directly in Edition 2021 due to more
                        // specific closure capture rules
                        let default = std::mem::take(&mut defaults.#field_ident);
                        // Option::from() dance to handle both T and Option<T>
                        if let Some(default) = Option::<FixedString<u16>>::from(default) {
                            b = b.value(default);
                        }
                    }
                    b
                        #( .placeholder(#placeholder) )*
                        .required(#required)
                        #( .min_length(#min_length) )*
                        #( .max_length(#max_length) )*
                })
                #( .description(#description) )*
            ),
        });

        parsers.push(quote::quote! {
            #field_ident: poise::find_modal_data(&mut data, stringify!(#field_ident)).text #ok_or,
        });
    }

    // Second builder count check. Required to trigger the error consistently.
    if builders.len() > 5 {
        let err = "cannot have more than five components in a modal";
        return Err(darling::Error::custom(err));
    }

    let modal_title = struct_attrs.name.unwrap_or(input.ident.to_string());
    let struct_ident = input.ident;
    let (impl_generics, ty_generics, where_clause) = input.generics.split_for_impl();
    Ok(quote::quote! { const _: () = {
        use std::borrow::Cow;
        use poise::serenity_prelude as serenity;
        use poise::serenity_prelude::small_fixed_array::{FixedArray, FixedString};
        impl #impl_generics poise::Modal for #struct_ident #ty_generics #where_clause {
            fn create(
                mut defaults: Option<Self>,
                custom_id: String,
            ) -> serenity::CreateInteractionResponse<'static> {
                serenity::CreateInteractionResponse::Modal(
                    serenity::CreateModal::new(custom_id, #modal_title)
                        .components(vec!{#( #builders )*})
                )
            }

            fn parse(mut data: serenity::ModalInteractionData) -> Self {
                Self { #( #parsers )* }
            }
        }
    }; }
    .into())
}

/// Used to generate a [`darling::Error`] on a specific `target` attribute.
#[doc(hidden)]
fn err_on_attr(attrs: &[darling::ast::NestedMeta], err: &str, target: &str) -> darling::Error {
    for attr in attrs.iter() {
        if let darling::ast::NestedMeta::Meta(meta) = attr {
            if meta.path().is_ident(target) {
                return darling::Error::custom(err).with_span(&meta.path());
            }
        }
    }
    darling::Error::custom(err)
}
