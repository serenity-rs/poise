mod prefix;
mod slash;

use proc_macro::TokenStream;
use syn::spanned::Spanned as _;

// Convert None => None and Some(T) => Some(T)
fn wrap_option<T: quote::ToTokens>(literal: Option<T>) -> syn::Expr {
    match literal {
        Some(literal) => syn::parse_quote! { Some(#literal) },
        None => syn::parse_quote! { None },
    }
}

struct AllLifetimesToStatic;
impl syn::fold::Fold for AllLifetimesToStatic {
    fn fold_lifetime(&mut self, _: syn::Lifetime) -> syn::Lifetime {
        syn::parse_quote! { 'static }
    }
}

#[derive(Debug)]
struct List<T>(Vec<T>);
impl<T: darling::FromMeta> darling::FromMeta for List<T> {
    fn from_list(items: &[::syn::NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| T::from_nested_meta(item))
            .collect::<darling::Result<Vec<T>>>()
            .map(Self)
    }
}
impl<T> Default for List<T> {
    fn default() -> Self {
        Self(Default::default())
    }
}

/// Representation of the command attribute arguments (`#[command(...)]`)
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
pub struct CommandArgs {
    prefix_command: bool,
    slash_command: bool,
    context_menu_command: Option<String>,

    aliases: List<String>,
    track_edits: bool,
    invoke_on_edit: bool,
    reuse_response: bool,
    broadcast_typing: bool,
    explanation_fn: Option<syn::Path>,
    check: Option<syn::Path>,
    on_error: Option<syn::Path>,
    rename: Option<String>,
    discard_spare_arguments: bool,
    hide_in_help: bool,
    ephemeral: bool,
    required_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    required_bot_permissions: Option<syn::punctuated::Punctuated<syn::Ident, syn::Token![|]>>,
    owners_only: bool,
    identifying_name: Option<String>,
    category: Option<String>,

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
    description: Option<String>,
    autocomplete: Option<syn::Path>,
    channel_types: Option<List<syn::Ident>>,
    min: Option<syn::Lit>,
    max: Option<syn::Lit>,
    lazy: bool,
    flag: bool,
    rest: bool,
}

/// Part of the Invocation struct. Represents a single parameter of a Discord command.
struct CommandParameter {
    name: syn::Ident,
    type_: syn::Type,
    args: ParamArgs,
    span: proc_macro2::Span,
}

/// Passed to prefix and slash command spec generators; contains info to be included in command spec
pub struct Invocation {
    command_name: String,
    parameters: Vec<CommandParameter>,
    description: Option<String>,
    explanation: Option<String>,
    function: syn::ItemFn,
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
                    let literal = literal.value();
                    let literal = literal.strip_prefix(' ').unwrap_or(&literal);

                    doc_lines += literal;
                    doc_lines += "\n";
                }
            }
        }
    }

    // Apply newline escapes
    let doc_lines = doc_lines.trim().replace("\\\n", "");

    if doc_lines.is_empty() {
        return (None, None);
    }

    let mut paragraphs = doc_lines.splitn(2, "\n\n");
    let inline_help = paragraphs.next().unwrap().replace("\n", " ");
    let multiline_help = paragraphs.next().map(|x| x.to_owned());

    (Some(inline_help), multiline_help)
}

pub fn command(
    args: CommandArgs,
    mut function: syn::ItemFn,
) -> Result<TokenStream, darling::Error> {
    // Verify that the function is marked async. Not strictly needed, but avoids confusion
    if function.sig.asyncness.is_none() {
        return Err(syn::Error::new(function.sig.span(), "command function must be async").into());
    }

    // Verify that at least one command type was enabled
    if !args.prefix_command && !args.slash_command && args.context_menu_command.is_none() {
        let err_msg = "you must enable at least one of `prefix_command`, `slash_command` or \
            `context_menu_command`";
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
            syn::Pat::Ident(pat_ident) => &pat_ident.ident,
            x => {
                return Err(syn::Error::new(x.span(), "must use an identifier pattern here").into())
            }
        };

        let attrs = pattern
            .attrs
            .drain(..)
            .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
            .collect::<Result<Vec<_>, _>>()?;
        let attrs = <ParamArgs as darling::FromMeta>::from_list(&attrs)?;

        parameters.push(CommandParameter {
            name: name.clone(),
            type_: (*pattern.ty).clone(),
            args: attrs,
            span: command_param.span(),
        });
    }

    // Extract the command descriptions from the function doc comments
    let (description, explanation) = extract_help_from_doc_comments(&function.attrs);

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
    let required_permissions = permissions_to_tokens(&args.required_permissions);
    let required_bot_permissions = permissions_to_tokens(&args.required_bot_permissions);

    let inv = Invocation {
        command_name: args
            .rename
            .clone()
            .unwrap_or_else(|| function.sig.ident.to_string()),
        parameters,
        description,
        explanation,
        args,
        function,
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
    let ctx_type_with_static = syn::fold::fold_type(&mut AllLifetimesToStatic, ctx_type.clone());

    let prefix_action = wrap_option(match inv.args.prefix_command {
        true => Some(prefix::generate_prefix_action(&inv)?),
        false => None,
    });
    let slash_action = wrap_option(match inv.args.slash_command {
        true => Some(slash::generate_slash_action(&inv)),
        false => None,
    });
    let context_menu_action = wrap_option(match &inv.args.context_menu_command {
        Some(_) => Some(slash::generate_context_menu_action(&inv)?),
        None => None,
    });

    let identifying_name = inv
        .args
        .identifying_name
        .as_ref()
        .unwrap_or(&inv.command_name);
    let command_name = &inv.command_name;
    let context_menu_name = wrap_option(inv.args.context_menu_command.as_ref());

    let description = wrap_option(inv.description.as_ref());
    let hide_in_help = &inv.args.hide_in_help;
    let category = wrap_option(inv.args.category.as_ref());

    let global_cooldown = wrap_option(inv.args.global_cooldown);
    let user_cooldown = wrap_option(inv.args.user_cooldown);
    let guild_cooldown = wrap_option(inv.args.guild_cooldown);
    let channel_cooldown = wrap_option(inv.args.channel_cooldown);
    let member_cooldown = wrap_option(inv.args.member_cooldown);

    let required_permissions = &inv.required_permissions;
    let required_bot_permissions = &inv.required_bot_permissions;
    let owners_only = inv.args.owners_only;

    let explanation = match &inv.args.explanation_fn {
        Some(explanation_fn) => quote::quote! { Some(#explanation_fn) },
        None => match &inv.explanation {
            Some(extracted_explanation) => quote::quote! { Some(|| #extracted_explanation.into()) },
            None => quote::quote! { None },
        },
    };

    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.args.check {
        Some(check) => quote::quote! { Some(|ctx| Box::pin(#check(ctx))) },
        None => quote::quote! { None },
    };
    let on_error = match &inv.args.on_error {
        Some(on_error) => quote::quote! { Some(|err| Box::pin(#on_error(err))) },
        None => quote::quote! { None },
    };

    let invoke_on_edit = inv.args.invoke_on_edit || inv.args.track_edits;
    let reuse_response = inv.args.reuse_response || inv.args.track_edits;
    let broadcast_typing = inv.args.broadcast_typing;
    let aliases = &inv.args.aliases.0;

    let parameters = slash::generate_parameters(&inv)?;
    let ephemeral = inv.args.ephemeral;

    let function_name = std::mem::replace(&mut inv.function.sig.ident, syn::parse_quote! { inner });
    let function_visibility = &inv.function.vis;
    let function = &inv.function;
    Ok(quote::quote! {
        #function_visibility fn #function_name() -> ::poise::Command<
            <#ctx_type_with_static as poise::_GetGenerics>::U,
            <#ctx_type_with_static as poise::_GetGenerics>::E,
        > {
            #function

            ::poise::Command {
                prefix_action: #prefix_action,
                slash_action: #slash_action,
                context_menu_action: #context_menu_action,

                subcommands: Vec::new(),
                name: #command_name,
                qualified_name: String::from(#command_name), // properly filled in later by Framework
                identifying_name: String::from(#identifying_name),
                category: #category,
                inline_help: #description,
                multiline_help: #explanation,
                hide_in_help: #hide_in_help,
                cooldowns: std::sync::Mutex::new(::poise::Cooldowns::new(::poise::CooldownConfig {
                    global: #global_cooldown.map(std::time::Duration::from_secs),
                    user: #user_cooldown.map(std::time::Duration::from_secs),
                    guild: #guild_cooldown.map(std::time::Duration::from_secs),
                    channel: #channel_cooldown.map(std::time::Duration::from_secs),
                    member: #member_cooldown.map(std::time::Duration::from_secs),
                })),
                reuse_response: #reuse_response,
                required_permissions: #required_permissions,
                required_bot_permissions: #required_bot_permissions,
                owners_only: #owners_only,
                check: #check,
                on_error: #on_error,
                parameters: vec![ #( #parameters ),* ],

                aliases: &[ #( #aliases, )* ],
                invoke_on_edit: #invoke_on_edit,
                broadcast_typing: #broadcast_typing,

                context_menu_name: #context_menu_name,
                ephemeral: #ephemeral,
            }
        }
    })
}
