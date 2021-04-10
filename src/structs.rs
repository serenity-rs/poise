//! Plain data structs that define the framework configuration.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Wrapper around either [`slash::Context`] or [`prefix::Context`]
pub enum Context<'a, U, E> {
    Slash(crate::SlashContext<'a, U, E>),
    Prefix(crate::PrefixContext<'a, U, E>),
}
impl<U, E> Clone for Context<'_, U, E> {
    fn clone(&self) -> Self {
        match *self {
            Self::Slash(x) => Self::Slash(x),
            Self::Prefix(x) => Self::Prefix(x),
        }
    }
}
impl<U, E> Copy for Context<'_, U, E> {}

impl<U, E> Context<'_, U, E> {
    pub fn author(&self) -> &serenity::User {
        match self {
            Self::Slash(ctx) => &ctx.interaction.member.user,
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }
}

/// Contains the location of the error with location-specific context
pub enum ErrorContext<'a, U, E> {
    Setup,
    Listener(&'a crate::Event<'a>),
    PrefixCommand(crate::PrefixCommandErrorContext<'a, U, E>),
    SlashCommand(crate::SlashCommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for ErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Setup => Self::Setup,
            Self::Listener(x) => Self::Listener(x),
            Self::PrefixCommand(x) => Self::PrefixCommand(x.clone()),
            Self::SlashCommand(x) => Self::SlashCommand(x.clone()),
        }
    }
}

pub struct FrameworkOptions<U, E> {
    /// Provide a callback to be invoked when any user code yields an error.
    // pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<()>,
    pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a crate::Event<'a>,
        &'a crate::Framework<U, E>,
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Slash command specific options.
    pub slash_options: crate::SlashFrameworkOptions<U, E>,
    /// Prefix command specific options.
    pub prefix_options: crate::PrefixFrameworkOptions<U, E>,
}

impl<U: Send + Sync, E: std::fmt::Display + Send> Default for FrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            on_error: |error, ctx| {
                Box::pin(async move {
                    match ctx {
                        ErrorContext::Setup => println!("Error in user data setup: {}", error),
                        ErrorContext::Listener(event) => println!(
                            "User event listener encountered an error on {} event: {}",
                            event.name(),
                            error
                        ),
                        ErrorContext::PrefixCommand(ctx) => {
                            println!(
                                "Error in prefix command \"{}\" from message \"{}\": {}",
                                &ctx.command.name, &ctx.ctx.msg.content, error
                            );
                        }
                        ErrorContext::SlashCommand(ctx) => {
                            println!("Error in slash command \"{}\": {}", ctx.command.name, error);
                        }
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            slash_options: Default::default(),
            prefix_options: Default::default(),
        }
    }
}

pub enum Arguments<'a> {
    Prefix(&'a str),
    Slash(&'a [serenity::ApplicationCommandInteractionDataOption]),
}
