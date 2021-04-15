#![allow(unused)] // temporary
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

enum Error {
    Syn(syn::Error),
    Darling(darling::Error),
}
impl From<syn::Error> for Error {
    fn from(x: syn::Error) -> Self {
        Self::Syn(x)
    }
}
impl From<darling::Error> for Error {
    fn from(x: darling::Error) -> Self {
        Self::Darling(x)
    }
}
impl From<(proc_macro2::Span, &'static str)> for Error {
    fn from(x: (proc_macro2::Span, &'static str)) -> Self {
        Self::Syn(syn::Error::new(x.0, x.1))
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
struct CommandAttrArgs {
    aliases: Aliases,
    track_edits: bool,
    broadcast_typing: Option<bool>,
    explanation_fn: Option<syn::Path>,
    check: Option<syn::Path>,
    on_error: Option<syn::Path>,
    rename: Option<String>,
    discard_spare_arguments: bool, // TODO: integrate
    slash_command: bool,
}

/// Representation of the function parameter attribute arguments
#[derive(Default, Debug, darling::FromMeta)]
#[darling(default)]
struct ParamAttrArgs {
    description: Option<String>,
}

/// Part of the Invocation struct. Represents a single parameter of a Discord command.
struct CommandParameter {
    name: syn::Ident,
    type_: syn::Type,
    description: Option<String>,
    span: proc_macro2::Span,
}

/// Passed to prefix and slash command spec generators; contains info to be included in command spec
struct Invocation<'a> {
    command_name: String,
    ctx_type: &'a syn::Type,
    return_type: &'a syn::Type,
    parameters: &'a [CommandParameter],
    description: Option<&'a str>,
    explanation: Option<&'a str>,
    function: &'a syn::ItemFn,
    rest: &'a CommandAttrArgs,
}

fn extract_help_from_doc_comments(attrs: &[syn::Attribute]) -> (Option<String>, Option<String>) {
    let mut doc_lines = String::new();
    for attr in attrs {
        if attr.path == quote::format_ident!("doc").into() {
            for token in attr.tokens.clone() {
                if let Ok(literal) = syn::parse2::<syn::LitStr>(token.into()) {
                    doc_lines += literal.value().trim();
                    doc_lines += "\n";
                }
            }
        }
    }

    let mut paragraphs = doc_lines.split("\n\n");

    let inline_help = paragraphs
        .next()
        .filter(|p| !p.is_empty())
        .map(|s| s.replace("\n", " "));

    let mut multiline_help = String::new();
    for paragraph in paragraphs {
        for line in paragraph.split('\n') {
            multiline_help += line;
            multiline_help += " ";
        }
        multiline_help += "\n\n";
    }
    let multiline_help = if multiline_help.is_empty() {
        None
    } else {
        Some(multiline_help)
    };

    (inline_help, multiline_help)
}

// ngl this is ugly
// transforms a type of form `Option<T>` into `T`
fn unwrap_option_type(t: &syn::Type) -> Option<&syn::Type> {
    if let syn::Type::Path(path) = t {
        if path.path.segments.len() == 1 {
            let path = &path.path.segments[0];
            if path.ident == "Option" {
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

fn generate_command_spec(inv: &Invocation) -> proc_macro2::TokenStream {
    let description = wrap_option(inv.description);
    let explanation = match &inv.rest.explanation_fn {
        Some(explanation_fn) => quote::quote! { Some(#explanation_fn) },
        None => match &inv.explanation {
            Some(extracted_explanation) => quote::quote! { Some(|| #extracted_explanation.into()) },
            None => quote::quote! { None },
        },
    };

    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.rest.check {
        Some(check) => quote::quote! { Some(|a| Box::pin(#check(a))) },
        None => quote::quote! { None },
    };
    let on_error = match &inv.rest.on_error {
        Some(on_error) => {
            if inv.rest.slash_command {
                quote::quote! {
                    Some(|err, ctx| Box::pin(#on_error(err, ::poise::CommandErrorContext::Prefix(ctx))))
                }
            } else {
                quote::quote! { Some(|err, ctx| Box::pin(#on_error(err, ctx))) }
            }
        }
        None => quote::quote! { None },
    };

    let maybe_wrapped_ctx = if inv.rest.slash_command {
        quote::quote! { ::poise::Context::Prefix(ctx) }
    } else {
        quote::quote! { ctx }
    };

    let command_name = &inv.command_name;
    let track_edits = inv.rest.track_edits;
    let broadcast_typing = wrap_option(inv.rest.broadcast_typing);
    let aliases = &inv.rest.aliases.0;
    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv.parameters.iter().map(|p| &p.type_).collect::<Vec<_>>();
    quote::quote! {
        ::poise::PrefixCommand {
            name: #command_name,
            action: |ctx, args| Box::pin(async move {
                let ( #( #param_names ),* ) = ::poise::parse_prefix_args!(
                    ctx.discord, ctx.msg, args =>
                    #( (#param_types) ),*
                ).await?;
                inner(#maybe_wrapped_ctx, #( #param_names ),* ).await
            }),
            options: ::poise::PrefixCommandOptions {
                track_edits: #track_edits,
                broadcast_typing: #broadcast_typing,
                aliases: &[ #( #aliases ),* ],
                inline_help: #description,
                multiline_help: #explanation,
                check: #check,
                on_error: #on_error,
            }
        }
    }
}

fn generate_slash_command_spec(inv: &Invocation) -> Result<proc_macro2::TokenStream, Error> {
    let command_name = &inv.command_name;
    let description = inv.description.as_ref().ok_or((
        inv.function.sig.span(),
        "slash commands must have a description",
    ))?;

    let mut parameter_builders = Vec::new();
    for param in inv.parameters {
        let param_name = &param.name;
        let param_description = param.description.as_ref().ok_or((
            param.span,
            "slash command parameters must have a description",
        ))?;

        let (required, type_) = match unwrap_option_type(&param.type_) {
            Some(t) => (false, t),
            None => (true, &param.type_),
        };

        parameter_builders.push(quote::quote! {
            |o| (&&std::marker::PhantomData::<#type_>).create(o)
                .required(#required)
                .name(stringify!(#param_name))
                .description(#param_description)
        });
    }

    // Box::pin the check and on_error callbacks in order to store them in a struct
    let check = match &inv.rest.check {
        Some(check) => quote::quote! { Some(|a| Box::pin(#check(a))) },
        None => quote::quote! { None },
    };
    let on_error = match &inv.rest.on_error {
        Some(on_error) => quote::quote! {
            Some(|err, ctx| Box::pin(#on_error(err, ::poise::CommandErrorContext::Slash(ctx))))
        },
        None => quote::quote! { None },
    };

    let param_names = inv.parameters.iter().map(|p| &p.name).collect::<Vec<_>>();
    let param_types = inv.parameters.iter().map(|p| &p.type_).collect::<Vec<_>>();
    Ok(quote::quote! {
        ::poise::SlashCommand {
            name: #command_name,
            description: #description,
            action: |ctx, args| Box::pin(async move {
                let ( #( #param_names ),* ) = ::poise::parse_slash_args!(
                    ctx.discord, ctx.interaction.guild_id, ctx.interaction.channel_id, args =>
                    #( (#param_names: #param_types) ),*
                ).await?;
                inner(::poise::Context::Slash(ctx), #( #param_names ),*).await
            }),
            parameters: {
                use ::poise::SlashArgument;
                vec![ #( #parameter_builders ),* ]
            },
            options: ::poise::SlashCommandOptions {
                check: #check,
                on_error: #on_error,
            }
        }
    })
}

fn command_inner(args: CommandAttrArgs, mut function: syn::ItemFn) -> Result<TokenStream, Error> {
    // Verify that the function is marked async. Not strictly needed, but avoids confusion
    if function.sig.asyncness.is_none() {
        return Err((function.sig.span(), "command function must be async").into());
    }

    // Collect argument names/types/attributes to insert into generated function
    let mut parameters = Vec::new();
    for command_param in function.sig.inputs.iter_mut().skip(1) {
        let pattern = match command_param {
            syn::FnArg::Typed(x) => &mut *x,
            syn::FnArg::Receiver(r) => {
                return Err((r.span(), "self argument is invalid here").into());
            }
        };
        let name = match &*pattern.pat {
            syn::Pat::Ident(pat_ident) => &pat_ident.ident,
            x => return Err((x.span(), "must use an identifier pattern here").into()),
        };

        let attrs = pattern
            .attrs
            .drain(..)
            .map(|attr| attr.parse_meta().map(syn::NestedMeta::Meta))
            .collect::<Result<Vec<_>, _>>()?;
        let attrs = <ParamAttrArgs as darling::FromMeta>::from_list(&attrs)?;

        parameters.push(CommandParameter {
            name: name.clone(),
            type_: (*pattern.ty).clone(),
            description: attrs.description,
            span: command_param.span(),
        });
    }

    let ctx_type = match function.sig.inputs.first() {
        Some(syn::FnArg::Typed(syn::PatType { ty, .. })) => &**ty,
        _ => return Err((function.sig.span(), "expected a Context parameter").into()),
    };
    let unit_type = syn::parse_quote! { () };
    let return_type = match &function.sig.output {
        syn::ReturnType::Default => &unit_type,
        syn::ReturnType::Type(_, ty) => &*ty,
    };

    // Extract the command descriptionss from the function doc comments
    let (description, explanation) = extract_help_from_doc_comments(&function.attrs);

    let invocation = Invocation {
        command_name: args
            .rename
            .clone()
            .unwrap_or_else(|| function.sig.ident.to_string()),
        ctx_type,
        return_type,
        parameters: &parameters,
        description: description.as_deref(),
        explanation: explanation.as_deref(),
        rest: &args,
        function: &function,
    };
    let command_spec = generate_command_spec(&invocation);
    let slash_command_spec = wrap_option(if args.slash_command {
        let spec = generate_slash_command_spec(&invocation)?;
        Some(spec)
    } else {
        None
    });

    // Needed because we're not allowed to have lifetimes in the hacky use case below
    let ctx_type_with_static = syn::fold::fold_type(&mut AllLifetimesToStatic, ctx_type.clone());

    let function_name = std::mem::replace(&mut function.sig.ident, syn::parse_quote! { inner });
    let function_visibility = &function.vis;
    Ok(TokenStream::from(quote::quote! {
        #function_visibility fn #function_name() -> (
            ::poise::PrefixCommand<
                <#ctx_type_with_static as poise::_GetGenerics>::U,
                <#ctx_type_with_static as poise::_GetGenerics>::E,
            >,
            Option<::poise::SlashCommand<
                <#ctx_type_with_static as poise::_GetGenerics>::U,
                <#ctx_type_with_static as poise::_GetGenerics>::E,
            >>,
        ) {
            #function

            use ::poise::serenity_prelude as serenity;
            (#command_spec, #slash_command_spec)
        }
    }))
}

#[proc_macro_attribute]
pub fn command(args: TokenStream, function: TokenStream) -> TokenStream {
    let args = syn::parse_macro_input!(args as Vec<syn::NestedMeta>);
    let args = match <CommandAttrArgs as darling::FromMeta>::from_list(&args) {
        Ok(x) => x,
        Err(e) => return TokenStream::from(e.write_errors()),
    };

    let function = syn::parse_macro_input!(function as syn::ItemFn);

    match command_inner(args, function) {
        // Ok(x) => panic!("{}", x),
        Ok(x) => x,
        Err(Error::Syn(e)) => e.into_compile_error().into(),
        Err(Error::Darling(e)) => e.write_errors().into(),
    }
}
