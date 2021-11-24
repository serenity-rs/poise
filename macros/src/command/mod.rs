mod prefix;
mod slash;

use proc_macro::TokenStream;
use syn::spanned::Spanned as _;

// ngl this is ugly
// transforms a type of form `OuterType<T>` into `T`
fn extract_type_parameter<'a>(outer_type: &str, t: &'a syn::Type) -> Option<&'a syn::Type> {
    if let syn::Type::Path(path) = t {
        if path.path.segments.len() == 1 {
            let path = &path.path.segments[0];
            if path.ident == outer_type {
                if let syn::PathArguments::AngleBracketed(generics) = &path.arguments {
                    if generics.args.len() == 1 {
                        if let syn::GenericArgument::Type(t) = &generics.args[0] {
                            return Some(t);
                        }
                    }
                }
            }
        }
    }
    None
}

// Convert None => None and Some(T) => Some(T)
fn wrap_option<T: quote::ToTokens>(literal: Option<T>) -> syn::Expr {
    match literal {
        Some(literal) => syn::parse_quote! { Some(#literal) },
        None => syn::parse_quote! { None },
    }
}

fn extract_option_type(t: &syn::Type) -> Option<&syn::Type> {
    extract_type_parameter("Option", t)
}

fn extract_vec_type(t: &syn::Type) -> Option<&syn::Type> {
    extract_type_parameter("Vec", t)
}

struct AllLifetimesToStatic;
impl syn::fold::Fold for AllLifetimesToStatic {
    fn fold_lifetime(&mut self, _: syn::Lifetime) -> syn::Lifetime {
        syn::parse_quote! { 'static }
    }
}

#[derive(Debug, Default)]
struct Aliases(Vec<String>);

impl darling::FromMeta for Aliases {
    fn from_list(items: &[::syn::NestedMeta]) -> darling::Result<Self> {
        items
            .iter()
            .map(|item| String::from_nested_meta(item))
            .collect::<darling::Result<Vec<String>>>()
            .map(Self)
    }
}

/// Representation of the command attribute arguments (`#[command(...)]`)
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
pub struct CommandOptions {
    prefix_command: bool,
    slash_command: bool,
    context_menu_command: Option<String>,

    aliases: Aliases,
    track_edits: bool,
    broadcast_typing: bool,
    explanation_fn: Option<syn::Path>,
    check: Option<syn::Path>,
    on_error: Option<syn::Path>,
    rename: Option<String>,
    discard_spare_arguments: bool,
    hide_in_help: bool,
    ephemeral: bool,
    required_permissions: Option<syn::Ident>,
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
struct ParamOptions {
    description: Option<String>,
    autocomplete: Option<syn::Path>,
    lazy: bool,
    flag: bool,
    rest: bool,
}

/// Part of the Invocation struct. Represents a single parameter of a Discord command.
struct CommandParameter {
    name: syn::Ident,
    type_: syn::Type,
    more: ParamOptions,
    span: proc_macro2::Span,
}

/// Passed to prefix and slash command spec generators; contains info to be included in command spec
pub struct Invocation<'a> {
    command_name: String,
    parameters: &'a [CommandParameter],
    description: Option<&'a str>,
    explanation: Option<&'a str>,
    function: &'a syn::ItemFn,
    required_permissions: &'a syn::Expr,
    more: &'a CommandOptions,
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

fn make_command_id(inv: &Invocation) -> proc_macro2::TokenStream {
    let identifying_name = &inv.more.identifying_name;
    let identifying_name = identifying_name.as_ref().unwrap_or(&inv.command_name);

    let description = wrap_option(inv.description);
    let hide_in_help = &inv.more.hide_in_help;
    let category = wrap_option(inv.more.category.as_ref());

    let global_cooldown = wrap_option(inv.more.global_cooldown);
    let user_cooldown = wrap_option(inv.more.user_cooldown);
    let guild_cooldown = wrap_option(inv.more.guild_cooldown);
    let channel_cooldown = wrap_option(inv.more.channel_cooldown);
    let member_cooldown = wrap_option(inv.more.member_cooldown);

    quote::quote! {
        ::poise::CommandId {
            identifying_name: String::from(#identifying_name),
            category: #category,
            inline_help: #description,
            hide_in_help: #hide_in_help,
            cooldowns: std::sync::Mutex::new(::poise::Cooldowns::new(::poise::CooldownConfig {
                global: #global_cooldown.map(std::time::Duration::from_secs),
                user: #user_cooldown.map(std::time::Duration::from_secs),
                guild: #guild_cooldown.map(std::time::Duration::from_secs),
                channel: #channel_cooldown.map(std::time::Duration::from_secs),
                member: #member_cooldown.map(std::time::Duration::from_secs),
            }))
        }
    }
}

pub fn command(
    args: CommandOptions,
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
        let attrs = <ParamOptions as darling::FromMeta>::from_list(&attrs)?;

        parameters.push(CommandParameter {
            name: name.clone(),
            type_: (*pattern.ty).clone(),
            more: attrs,
            span: command_param.span(),
        });
    }

    let ctx_type = match function.sig.inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => &**ty,
        _ => {
            return Err(syn::Error::new(function.sig.span(), "expected a Context parameter").into())
        }
    };

    // Extract the command descriptionss from the function doc comments
    let (description, explanation) = extract_help_from_doc_comments(&function.attrs);

    let required_permissions = match &args.required_permissions {
        Some(perms) => syn::parse_quote! { poise::serenity_prelude::Permissions::#perms },
        None => syn::parse_quote! { poise::serenity_prelude::Permissions::empty() },
    };

    let invocation = Invocation {
        command_name: args
            .rename
            .clone()
            .unwrap_or_else(|| function.sig.ident.to_string()),
        parameters: &parameters,
        description: description.as_deref(),
        explanation: explanation.as_deref(),
        more: &args,
        function: &function,
        required_permissions: &required_permissions,
    };

    let prefix_command_spec = wrap_option(if args.prefix_command {
        Some(prefix::generate_prefix_command_spec(&invocation)?)
    } else {
        None
    });
    let slash_command_spec = wrap_option(if args.slash_command {
        Some(slash::generate_slash_command_spec(&invocation)?)
    } else {
        None
    });
    let context_menu_command_spec = wrap_option(if let Some(name) = &args.context_menu_command {
        Some(slash::generate_context_menu_command_spec(
            &invocation,
            name,
        )?)
    } else {
        None
    });
    let command_id = make_command_id(&invocation);

    // Needed because we're not allowed to have lifetimes in the hacky use case below
    let ctx_type_with_static = syn::fold::fold_type(&mut AllLifetimesToStatic, ctx_type.clone());

    let function_name = std::mem::replace(&mut function.sig.ident, syn::parse_quote! { inner });
    let function_visibility = &function.vis;
    Ok(TokenStream::from(quote::quote! {
        #function_visibility fn #function_name() -> ::poise::CommandDefinition<
            <#ctx_type_with_static as poise::_GetGenerics>::U,
            <#ctx_type_with_static as poise::_GetGenerics>::E,
        > {
            #function

            let command_id = std::sync::Arc::new(#command_id);

            ::poise::CommandDefinition {
                prefix: #prefix_command_spec,
                slash: #slash_command_spec,
                context_menu: #context_menu_command_spec,
            }
        }
    }))
}
