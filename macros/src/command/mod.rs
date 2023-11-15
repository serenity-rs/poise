mod prefix;
mod slash;

use crate::util::{wrap_option, wrap_option_to_string};
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
    name: Option<syn::Ident>,
    type_: syn::Type,
    args: ParamArgs,
    span: proc_macro2::Span,
}

/// Passed to prefix and slash command spec generators; contains info to be included in command spec
pub struct Invocation {
    command_name: String,
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
        if attr.path == quote::format_ident!("doc").into() {
            for token in attr.tokens.clone() {
                if let Ok(literal) = syn::parse2::<syn::LitStr>(token.into()) {
                    doc_lines += literal.value().trim(); // Trim lines like rustdoc does
                    doc_lines += "\n";
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
        let pattern = match command_param {
            syn::FnArg::Typed(x) => &mut *x,
            syn::FnArg::Receiver(r) => {
                return Err(syn::Error::new(r.span(), "self argument is invalid here").into());
            }
        };
        let name = match &*pattern.pat {
            syn::Pat::Ident(pat_ident) => Some(pat_ident.ident.clone()),
            _ => None,
        };

        let attrs = pattern
            .attrs
            .drain(..)
            .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
            .collect::<Result<Vec<_>, _>>()?;
        let attrs = <ParamArgs as darling::FromMeta>::from_list(&attrs)?;

        parameters.push(CommandParameter {
            name,
            type_: (*pattern.ty).clone(),
            args: attrs,
            span: command_param.span(),
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
        command_name: args
            .rename
            .clone()
            .unwrap_or_else(|| function.sig.ident.to_string()),
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

    let identifying_name = inv
        .args
        .identifying_name
        .clone()
        .unwrap_or_else(|| inv.function.sig.ident.to_string());
    let command_name = &inv.command_name;
    let context_menu_name = wrap_option_to_string(inv.args.context_menu_command.as_ref());

    let description = match &inv.description {
        Some(x) => quote::quote! { Some(#x.to_string()) },
        None => quote::quote! { None },
    };
    let hide_in_help = &inv.args.hide_in_help;
    let category = wrap_option_to_string(inv.args.category.as_ref());

    let global_cooldown = wrap_option(inv.args.global_cooldown);
    let user_cooldown = wrap_option(inv.args.user_cooldown);
    let guild_cooldown = wrap_option(inv.args.guild_cooldown);
    let channel_cooldown = wrap_option(inv.args.channel_cooldown);
    let member_cooldown = wrap_option(inv.args.member_cooldown);

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

    let name_localizations = crate::util::vec_tuple_2_to_hash_map(inv.args.name_localized);
    let description_localizations =
        crate::util::vec_tuple_2_to_hash_map(inv.args.description_localized);

    let function_name = std::mem::replace(&mut inv.function.sig.ident, syn::parse_quote! { inner });
    let function_generics = &inv.function.sig.generics;
    let function_visibility = &inv.function.vis;
    let function = &inv.function;
    Ok(quote::quote! {
        #[allow(clippy::str_to_string)]
        #function_visibility fn #function_name #function_generics() -> ::poise::Command<
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
                category: #category,
                description: #description,
                description_localizations: #description_localizations,
                help_text: #help_text,
                hide_in_help: #hide_in_help,
                cooldowns: std::sync::Mutex::new(::poise::Cooldowns::new()),
                cooldown_config: std::sync::Mutex::new(::poise::CooldownConfig {
                    global: #global_cooldown.map(std::time::Duration::from_secs),
                    user: #user_cooldown.map(std::time::Duration::from_secs),
                    guild: #guild_cooldown.map(std::time::Duration::from_secs),
                    channel: #channel_cooldown.map(std::time::Duration::from_secs),
                    member: #member_cooldown.map(std::time::Duration::from_secs),
                    __non_exhaustive: (),
                }),
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
