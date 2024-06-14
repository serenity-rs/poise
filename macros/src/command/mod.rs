mod prefix;
mod slash;

use crate::util::{
    iter_tuple_2_to_hash_map, wrap_option, wrap_option_and_map, wrap_option_to_string,
};
use proc_macro::TokenStream;
use syn::spanned::Spanned as _;

/// Representation of the command attribute arguments (`#[command(...)]`)
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
pub struct CommandArgs {
    prefix_command: bool,
    slash_command: bool,
    context_menu_command: Option<String>,

    // When changing these, document it in parent file!
    // TODO: decide why darling(multiple) feels wrong here but not in e.g. localizations (because
    //  if it's actually irrational, the inconsistency should be fixed)
    subcommands: crate::util::List<syn::Path>,
    aliases: crate::util::List<String>,
    subcommand_required: bool,
    invoke_on_edit: bool,
    reuse_response: bool,
    track_deletion: bool,
    track_edits: bool,
    broadcast_typing: bool,
    help_text_fn: Option<syn::Path>,
    #[darling(multiple)]
    check: Vec<syn::Path>,
    on_error: Option<syn::Path>,
    rename: Option<String>,
    #[darling(multiple)]
    name_localized: Vec<crate::util::Tuple2<String>>,
    #[darling(multiple)]
    description_localized: Vec<crate::util::Tuple2<String>>,
    discard_spare_arguments: bool,
    hide_in_help: bool,
    ephemeral: bool,
    default_member_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    required_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    required_bot_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    owners_only: bool,
    guild_only: bool,
    dm_only: bool,
    nsfw_only: bool,
    identifying_name: Option<String>,
    category: Option<String>,
    custom_data: Option<syn::Expr>,

    // In seconds
    global_cooldown: Option<u64>,
    user_cooldown: Option<u64>,
    guild_cooldown: Option<u64>,
    channel_cooldown: Option<u64>,
    member_cooldown: Option<u64>,
}

impl CommandArgs {
    // Check if a field has the default value
    fn is_default<T: Default + PartialEq>(value: &T) -> bool {
        value == &T::default()
    }

    // create a new CommandArgs from self, with default fields replaced by value from GroupArgs
    pub fn from_group_args(&self, group_args: &GroupArgs) -> CommandArgs {
        CommandArgs {
            prefix_command: if Self::is_default(&self.prefix_command) {
                group_args.prefix_command
            } else {
                self.prefix_command
            },
            slash_command: if Self::is_default(&self.slash_command) {
                group_args.slash_command
            } else {
                self.slash_command
            },
            context_menu_command: if Self::is_default(&self.context_menu_command) {
                group_args.context_menu_command.clone()
            } else {
                self.context_menu_command.clone()
            },
            subcommands: self.subcommands.clone(), // `GroupArgs` doesn't have `subcommands`
            aliases: self.aliases.clone(),         // `GroupArgs` doesn't have `aliases`
            subcommand_required: self.subcommand_required, // `GroupArgs` doesn't have `subcommand_required`
            invoke_on_edit: if Self::is_default(&self.invoke_on_edit) {
                group_args.invoke_on_edit
            } else {
                self.invoke_on_edit
            },
            reuse_response: if Self::is_default(&self.reuse_response) {
                group_args.reuse_response
            } else {
                self.reuse_response
            },
            track_deletion: if Self::is_default(&self.track_deletion) {
                group_args.track_deletion
            } else {
                self.track_deletion
            },
            track_edits: if Self::is_default(&self.track_edits) {
                group_args.track_edits
            } else {
                self.track_edits
            },
            broadcast_typing: if Self::is_default(&self.broadcast_typing) {
                group_args.broadcast_typing
            } else {
                self.broadcast_typing
            },
            help_text_fn: if Self::is_default(&self.help_text_fn) {
                group_args.help_text_fn.clone()
            } else {
                self.help_text_fn.clone()
            },
            check: if Self::is_default(&self.check) {
                group_args.check.clone()
            } else {
                self.check.clone()
            },
            on_error: if Self::is_default(&self.on_error) {
                group_args.on_error.clone()
            } else {
                self.on_error.clone()
            },
            rename: self.rename.clone(), // `GroupArgs` doesn't have `rename`
            name_localized: if Self::is_default(&self.name_localized) {
                group_args.name_localized.clone()
            } else {
                self.name_localized.clone()
            },
            description_localized: if Self::is_default(&self.description_localized) {
                group_args.description_localized.clone()
            } else {
                self.description_localized.clone()
            },
            discard_spare_arguments: if Self::is_default(&self.discard_spare_arguments) {
                group_args.discard_spare_arguments
            } else {
                self.discard_spare_arguments
            },
            hide_in_help: if Self::is_default(&self.hide_in_help) {
                group_args.hide_in_help
            } else {
                self.hide_in_help
            },
            ephemeral: if Self::is_default(&self.ephemeral) {
                group_args.ephemeral
            } else {
                self.ephemeral
            },
            default_member_permissions: if Self::is_default(&self.default_member_permissions) {
                group_args.default_member_permissions.clone()
            } else {
                self.default_member_permissions.clone()
            },
            required_permissions: if Self::is_default(&self.required_permissions) {
                group_args.required_permissions.clone()
            } else {
                self.required_permissions.clone()
            },
            required_bot_permissions: if Self::is_default(&self.required_bot_permissions) {
                group_args.required_bot_permissions.clone()
            } else {
                self.required_bot_permissions.clone()
            },
            owners_only: if Self::is_default(&self.owners_only) {
                group_args.owners_only
            } else {
                self.owners_only
            },
            guild_only: if Self::is_default(&self.guild_only) {
                group_args.guild_only
            } else {
                self.guild_only
            },
            dm_only: if Self::is_default(&self.dm_only) {
                group_args.dm_only
            } else {
                self.dm_only
            },
            nsfw_only: if Self::is_default(&self.nsfw_only) {
                group_args.nsfw_only
            } else {
                self.nsfw_only
            },
            identifying_name: self.identifying_name.clone(), // `GroupArgs` doesn't have `identifying_name`
            category: if Self::is_default(&self.category) {
                group_args.category.clone()
            } else {
                self.category.clone()
            },
            custom_data: if Self::is_default(&self.custom_data) {
                group_args.custom_data.clone()
            } else {
                self.custom_data.clone()
            },
            global_cooldown: if Self::is_default(&self.global_cooldown) {
                group_args.global_cooldown
            } else {
                self.global_cooldown
            },
            user_cooldown: if Self::is_default(&self.user_cooldown) {
                group_args.user_cooldown
            } else {
                self.user_cooldown
            },
            guild_cooldown: if Self::is_default(&self.guild_cooldown) {
                group_args.guild_cooldown
            } else {
                self.guild_cooldown
            },
            channel_cooldown: if Self::is_default(&self.channel_cooldown) {
                group_args.channel_cooldown
            } else {
                self.channel_cooldown
            },
            member_cooldown: if Self::is_default(&self.member_cooldown) {
                group_args.member_cooldown
            } else {
                self.member_cooldown
            },
        }
    }
}

/// Representation of the group attribute arguments (`#[group(...)]`)
///
/// Same as `CommandArgs`, but with the following removed:
/// - subcommands
/// - aliases
/// - subcommand_required
/// - rename
/// - identifying_name
///
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
pub struct GroupArgs {
    prefix_command: bool,
    slash_command: bool,
    context_menu_command: Option<String>,

    // When changing these, document it in parent file!
    // TODO: decide why darling(multiple) feels wrong here but not in e.g. localizations (because
    //  if it's actually irrational, the inconsistency should be fixed)
    invoke_on_edit: bool,
    reuse_response: bool,
    track_deletion: bool,
    track_edits: bool,
    broadcast_typing: bool,
    help_text_fn: Option<syn::Path>,
    #[darling(multiple)]
    check: Vec<syn::Path>,
    on_error: Option<syn::Path>,
    #[darling(multiple)]
    name_localized: Vec<crate::util::Tuple2<String>>,
    #[darling(multiple)]
    description_localized: Vec<crate::util::Tuple2<String>>,
    discard_spare_arguments: bool,
    hide_in_help: bool,
    ephemeral: bool,
    default_member_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    required_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    required_bot_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    owners_only: bool,
    guild_only: bool,
    dm_only: bool,
    nsfw_only: bool,
    category: Option<String>,
    custom_data: Option<syn::Expr>,

    // In seconds
    global_cooldown: Option<u64>,
    user_cooldown: Option<u64>,
    guild_cooldown: Option<u64>,
    channel_cooldown: Option<u64>,
    member_cooldown: Option<u64>,
}

/// Representation of the function parameter attribute arguments
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
struct ParamArgs {
    // When changing these, document it in parent file!
    description: Option<String>,
    rename: Option<String>,
    #[darling(multiple)]
    name_localized: Vec<crate::util::Tuple2<String>>,
    #[darling(multiple)]
    description_localized: Vec<crate::util::Tuple2<String>>,
    autocomplete: Option<syn::Path>,
    channel_types: Option<crate::util::List<syn::Ident>>,
    choices: Option<crate::util::List<syn::Lit>>,
    min: Option<syn::Lit>,
    max: Option<syn::Lit>,
    min_length: Option<syn::Lit>,
    max_length: Option<syn::Lit>,
    lazy: bool,
    flag: bool,
    rest: bool,
}

/// Part of the Invocation struct. Represents a single parameter of a Discord command.
struct CommandParameter {
    name: String,
    type_: syn::Type,
    args: ParamArgs,
    span: proc_macro2::Span,
}

/// Passed to prefix and slash command spec generators; contains info to be included in command spec
pub struct Invocation {
    parameters: Vec<CommandParameter>,
    description: Option<String>,
    help_text: Option<String>,
    function: syn::ItemFn,
    default_member_permissions: syn::Expr,
    required_permissions: syn::Expr,
    required_bot_permissions: syn::Expr,
    args: CommandArgs,
}

fn extract_help_from_doc_comments(attrs: &[syn::Attribute]) -> (Option<String>, Option<String>) {
    let mut doc_lines = String::new();
    for attr in attrs {
        if let syn::Meta::NameValue(doc_attr) = &attr.meta {
            if doc_attr.path == quote::format_ident!("doc").into() {
                if let syn::Expr::Lit(lit_expr) = &doc_attr.value {
                    if let syn::Lit::Str(literal) = &lit_expr.lit {
                        doc_lines += literal.value().trim(); // Trim lines like rustdoc does
                        doc_lines += "\n";
                    }
                }
            }
        }
    }

    // Trim trailing newline and apply newline escapes
    let doc_lines = doc_lines.trim().replace("\\\n", "");

    let mut paragraphs = doc_lines.splitn(2, "\n\n").filter(|x| !x.is_empty()); // "".split => [""]

    // Pop first paragraph as description if needed (but no newlines bc description is single line)
    let description = paragraphs.next().map(|x| x.replace("\n", " "));
    // Use rest of doc comments as help text
    let help_text = paragraphs.next().map(|x| x.to_owned());

    (description, help_text)
}

pub fn command(
    args: CommandArgs,
    mut function: syn::ItemFn,
) -> Result<TokenStream, darling::Error> {
    // Verify some things about the function. Not strictly needed, but avoids confusion
    if function.sig.asyncness.is_none() {
        return Err(syn::Error::new(function.sig.span(), "command function must be async").into());
    }
    if function.sig.output == syn::ReturnType::Default {
        return Err(syn::Error::new(
            function.sig.span(),
            "command function must return Result<(), ...>",
        )
        .into());
    }

    // Verify that at least one command type was enabled
    if !args.prefix_command && !args.slash_command && args.context_menu_command.is_none() {
        let err_msg = "you must enable at least one of `prefix_command`, `slash_command` or \
            `context_menu_command`";
        return Err(syn::Error::new(proc_macro2::Span::call_site(), err_msg).into());
    }

    // If subcommand_required is set to true, then the command cannot have any arguments
    if args.subcommand_required && function.sig.inputs.len() > 1 {
        let err_msg = "subcommand_required is set to true, but the command has arguments";
        return Err(syn::Error::new(proc_macro2::Span::call_site(), err_msg).into());
    }

    // If subcommand_required is set to true, then the command must have at least one subcommand
    if args.subcommand_required && args.subcommands.0.is_empty() {
        let err_msg = "subcommand_required is set to true, but the command has no subcommands";
        return Err(syn::Error::new(proc_macro2::Span::call_site(), err_msg).into());
    }

    // Collect argument names/types/attributes to insert into generated function
    let mut parameters = Vec::new();
    for command_param in function.sig.inputs.iter_mut().skip(1) {
        let span = command_param.span();

        let pattern = match command_param {
            syn::FnArg::Typed(x) => x,
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "self argument is invalid here").into());
            }
        };

        let attrs: Vec<_> = pattern
            .attrs
            .drain(..)
            .map(|attr| darling::ast::NestedMeta::Meta(attr.meta))
            .collect();
        let attrs = <ParamArgs as darling::FromMeta>::from_list(&attrs)?;

        let name = if let Some(rename) = &attrs.rename {
            rename.clone()
        } else if let syn::Pat::Ident(ident) = &*pattern.pat {
            ident.ident.to_string().trim_start_matches("r#").into()
        } else {
            let message = "#[rename = \"...\"] must be specified for pattern parameters";
            return Err(syn::Error::new(pattern.pat.span(), message).into());
        };
        parameters.push(CommandParameter {
            name,
            type_: (*pattern.ty).clone(),
            args: attrs,
            span,
        });
    }

    // Extract the command descriptions from the function doc comments
    let (description, help_text) = extract_help_from_doc_comments(&function.attrs);

    fn permissions_to_tokens(
        perms: &Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    ) -> syn::Expr {
        match perms {
            Some(perms) => {
                let perms = perms.iter();
                syn::parse_quote! { #(poise::serenity_prelude::Permissions::#perms)|* }
            }
            None => syn::parse_quote! { poise::serenity_prelude::Permissions::empty() },
        }
    }
    let default_member_permissions = permissions_to_tokens(&args.default_member_permissions);
    let required_permissions = permissions_to_tokens(&args.required_permissions);
    let required_bot_permissions = permissions_to_tokens(&args.required_bot_permissions);

    let inv = Invocation {
        parameters,
        description,
        help_text,
        args,
        function,
        default_member_permissions,
        required_permissions,
        required_bot_permissions,
    };

    Ok(TokenStream::from(generate_command(inv)?))
}

fn generate_command(mut inv: Invocation) -> Result<proc_macro2::TokenStream, darling::Error> {
    let ctx_type = match inv.function.sig.inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => &**ty,
        _ => {
            return Err(
                syn::Error::new(inv.function.sig.span(), "expected a Context parameter").into(),
            )
        }
    };
    // Needed because we're not allowed to have lifetimes in the hacky use case below
    let ctx_type_with_static =
        syn::fold::fold_type(&mut crate::util::AllLifetimesToStatic, ctx_type.clone());

    let prefix_action = wrap_option(match inv.args.prefix_command {
        true => Some(prefix::generate_prefix_action(&inv)?),
        false => None,
    });
    let slash_action = wrap_option(match inv.args.slash_command {
        true => Some(slash::generate_slash_action(&inv)?),
        false => None,
    });
    let context_menu_action = wrap_option(match &inv.args.context_menu_command {
        Some(_) => Some(slash::generate_context_menu_action(&inv)?),
        None => None,
    });

    let function_name = inv
        .function
        .sig
        .ident
        .to_string()
        .trim_start_matches("r#")
        .to_string();
    let identifying_name = inv
        .args
        .identifying_name
        .clone()
        .unwrap_or_else(|| function_name.clone());
    let command_name = &inv
        .args
        .rename
        .clone()
        .unwrap_or_else(|| function_name.clone());

    let context_menu_name = wrap_option_to_string(inv.args.context_menu_command.as_ref());

    let hide_in_help = &inv.args.hide_in_help;
    let description = wrap_option_to_string(inv.description.as_ref());
    let category = wrap_option_to_string(inv.args.category.as_ref());

    let cooldown_config = generate_cooldown_config(&inv.args);

    let default_member_permissions = &inv.default_member_permissions;
    let required_permissions = &inv.required_permissions;
    let required_bot_permissions = &inv.required_bot_permissions;
    let subcommand_required = inv.args.subcommand_required;
    let owners_only = inv.args.owners_only;
    let guild_only = inv.args.guild_only;
    let dm_only = inv.args.dm_only;
    let nsfw_only = inv.args.nsfw_only;

    let help_text = match &inv.args.help_text_fn {
        Some(help_text_fn) => quote::quote! { Some(#help_text_fn()) },
        None => match &inv.help_text {
            Some(extracted_explanation) => quote::quote! { Some(#extracted_explanation.into()) },
            None => quote::quote! { None },
        },
    };

    let checks = &inv.args.check;
    // Box::pin the callback in order to store it in a struct
    let on_error = match &inv.args.on_error {
        Some(on_error) => quote::quote! { Some(|err| Box::pin(#on_error(err))) },
        None => quote::quote! { None },
    };

    let invoke_on_edit = inv.args.invoke_on_edit || inv.args.track_edits;
    let reuse_response = inv.args.reuse_response || inv.args.track_edits;
    let track_deletion = inv.args.track_deletion || inv.args.track_edits;
    let broadcast_typing = inv.args.broadcast_typing;
    let aliases = &inv.args.aliases.0;
    let subcommands = &inv.args.subcommands.0;

    let parameters = slash::generate_parameters(&inv)?;
    let ephemeral = inv.args.ephemeral;
    let custom_data = match &inv.args.custom_data {
        Some(custom_data) => quote::quote! { Box::new(#custom_data) },
        None => quote::quote! { Box::new(()) },
    };

    let name_localizations = iter_tuple_2_to_hash_map(inv.args.name_localized.into_iter());
    let description_localizations =
        iter_tuple_2_to_hash_map(inv.args.description_localized.into_iter());

    let function_ident =
        std::mem::replace(&mut inv.function.sig.ident, syn::parse_quote! { inner });
    let function_generics = &inv.function.sig.generics;
    let function_visibility = &inv.function.vis;
    let function = &inv.function;
    Ok(quote::quote! {
        #[allow(clippy::str_to_string)]
        #function_visibility fn #function_ident #function_generics() -> ::poise::Command<
            <#ctx_type_with_static as poise::_GetGenerics>::U,
            <#ctx_type_with_static as poise::_GetGenerics>::E,
        > {
            #function

            ::poise::Command {
                prefix_action: #prefix_action,
                slash_action: #slash_action,
                context_menu_action: #context_menu_action,

                subcommands: vec![ #( #subcommands() ),* ],
                subcommand_required: #subcommand_required,
                name: #command_name.to_string(),
                name_localizations: #name_localizations,
                qualified_name: String::from(#command_name), // properly filled in later by Framework
                identifying_name: String::from(#identifying_name),
                source_code_name: String::from(#function_name),
                category: #category,
                description: #description,
                description_localizations: #description_localizations,
                help_text: #help_text,
                hide_in_help: #hide_in_help,
                cooldowns: std::sync::Mutex::new(::poise::Cooldowns::new()),
                cooldown_config: #cooldown_config,
                reuse_response: #reuse_response,
                default_member_permissions: #default_member_permissions,
                required_permissions: #required_permissions,
                required_bot_permissions: #required_bot_permissions,
                owners_only: #owners_only,
                guild_only: #guild_only,
                dm_only: #dm_only,
                nsfw_only: #nsfw_only,
                checks: vec![ #( |ctx| Box::pin(#checks(ctx)) ),* ],
                on_error: #on_error,
                parameters: vec![ #( #parameters ),* ],
                custom_data: #custom_data,

                aliases: vec![ #( #aliases.to_string(), )* ],
                invoke_on_edit: #invoke_on_edit,
                track_deletion: #track_deletion,
                broadcast_typing: #broadcast_typing,

                context_menu_name: #context_menu_name,
                ephemeral: #ephemeral,

                __non_exhaustive: (),
            }
        }
    })
}

fn generate_cooldown_config(args: &CommandArgs) -> proc_macro2::TokenStream {
    let all_cooldowns = [
        args.global_cooldown,
        args.user_cooldown,
        args.guild_cooldown,
        args.channel_cooldown,
        args.member_cooldown,
    ];

    if all_cooldowns.iter().all(Option::is_none) {
        return quote::quote!(std::sync::RwLock::default());
    }

    let to_seconds_path = quote::quote!(std::time::Duration::from_secs);

    let global_cooldown = wrap_option_and_map(args.global_cooldown, &to_seconds_path);
    let user_cooldown = wrap_option_and_map(args.user_cooldown, &to_seconds_path);
    let guild_cooldown = wrap_option_and_map(args.guild_cooldown, &to_seconds_path);
    let channel_cooldown = wrap_option_and_map(args.channel_cooldown, &to_seconds_path);
    let member_cooldown = wrap_option_and_map(args.member_cooldown, &to_seconds_path);

    quote::quote!(
        std::sync::RwLock::new(::poise::CooldownConfig {
            global: #global_cooldown,
            user: #user_cooldown,
            guild: #guild_cooldown,
            channel: #channel_cooldown,
            member: #member_cooldown,
            __non_exhaustive: ()
        })
    )
}
