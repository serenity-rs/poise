//! Plain data structs that define the framework configuration.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Wrapper around either [`SlashContext`] or [`PrefixContext`]
pub enum Context<'a, U, E> {
    Slash(crate::SlashContext<'a, U, E>),
    Prefix(crate::PrefixContext<'a, U, E>),
}
impl<U, E> Clone for Context<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U, E> Copy for Context<'_, U, E> {}

// needed for proc macro
#[doc(hidden)]
pub trait _GetGenerics {
    type U;
    type E;
}
impl<U, E> _GetGenerics for Context<'_, U, E> {
    type U = U;
    type E = E;
}

impl<U, E> Context<'_, U, E> {
    pub fn discord(&self) -> &serenity::Context {
        match self {
            Self::Slash(ctx) => ctx.discord,
            Self::Prefix(ctx) => ctx.discord,
        }
    }

    pub fn framework(&self) -> &crate::Framework<U, E> {
        match self {
            Self::Slash(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    pub fn data(&self) -> &U {
        match self {
            Self::Slash(ctx) => ctx.data,
            Self::Prefix(ctx) => ctx.data,
        }
    }

    pub fn channel_id(&self) -> serenity::ChannelId {
        match self {
            Self::Slash(ctx) => match ctx.interaction.channel_id {
                Some(x) => x,
                None => panic!("Slash command was sent outside specific channel"),
            },
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    pub fn guild_id(&self) -> Option<serenity::GuildId> {
        match self {
            Self::Slash(ctx) => ctx.interaction.guild_id,
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::Slash(ctx) => ctx.interaction.id.created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    pub fn author(&self) -> Option<&serenity::User> {
        match self {
            Self::Slash(ctx) => {
                if let Some(member) = &ctx.interaction.member {
                    Some(&member.user)
                } else if let Some(user) = &ctx.interaction.user {
                    Some(user)
                } else {
                    // Neither a Member nor a User was sent with interaction
                    None
                }
            }
            Self::Prefix(ctx) => Some(&ctx.msg.author),
        }
    }
}

pub enum CommandRef<'a, U, E> {
    Prefix(&'a crate::PrefixCommand<U, E>),
    Slash(&'a crate::SlashCommand<U, E>),
}

impl<U, E> Clone for CommandRef<'_, U, E> {
    fn clone(&self) -> Self {
        match *self {
            Self::Prefix(x) => Self::Prefix(x),
            Self::Slash(x) => Self::Slash(x),
        }
    }
}

impl<U, E> Copy for CommandRef<'_, U, E> {}

impl<U, E> CommandRef<'_, U, E> {
    pub fn name(self) -> &'static str {
        match self {
            Self::Prefix(x) => x.name,
            Self::Slash(x) => x.name,
        }
    }
}

pub enum CommandErrorContext<'a, U, E> {
    Prefix(crate::PrefixCommandErrorContext<'a, U, E>),
    Slash(crate::SlashCommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for CommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Prefix(x) => Self::Prefix(x.clone()),
            Self::Slash(x) => Self::Slash(x.clone()),
        }
    }
}

impl<'a, U, E> CommandErrorContext<'a, U, E> {
    pub fn command(&self) -> CommandRef<'_, U, E> {
        match self {
            Self::Prefix(x) => CommandRef::Prefix(x.command),
            Self::Slash(x) => CommandRef::Slash(x.command),
        }
    }

    pub fn while_checking(&self) -> bool {
        match self {
            Self::Prefix(x) => x.while_checking,
            Self::Slash(x) => x.while_checking,
        }
    }
    pub fn ctx(&self) -> Context<'a, U, E> {
        match self {
            Self::Prefix(x) => Context::Prefix(x.ctx),
            Self::Slash(x) => Context::Slash(x.ctx),
        }
    }
}

/// Contains the location of the error with location-specific context
pub enum ErrorContext<'a, U, E> {
    Setup,
    Listener(&'a crate::Event<'a>),
    Command(CommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for ErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Setup => Self::Setup,
            Self::Listener(x) => Self::Listener(x),
            Self::Command(x) => Self::Command(x.clone()),
        }
    }
}

pub struct FrameworkOptions<U, E> {
    /// Provide a callback to be invoked when any user code yields an error.
    pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    pub pre_command: fn(Context<'_, U, E>) -> BoxFuture<'_, ()>,
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

impl<U, E> FrameworkOptions<U, E> {
    pub fn command(
        &mut self,
        spec: fn() -> (
            crate::PrefixCommand<U, E>,
            Option<crate::SlashCommand<U, E>>,
        ),
    ) {
        let (prefix_command, slash_command) = spec();

        self.prefix_options.commands.push(prefix_command);

        if let Some(slash_command) = slash_command {
            self.slash_options.commands.push(slash_command);
        }
    }

    pub fn command_with_category(
        &mut self,
        spec: fn() -> (
            crate::PrefixCommand<U, E>,
            Option<crate::SlashCommand<U, E>>,
        ),
        category: &'static str,
    ) {
        let (mut prefix_command, slash_command) = spec();

        prefix_command.options.category = Some(category);
        self.prefix_options.commands.push(prefix_command);

        if let Some(slash_command) = slash_command {
            self.slash_options.commands.push(slash_command);
        }
    }
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
                        ErrorContext::Command(CommandErrorContext::Prefix(ctx)) => {
                            println!(
                                "Error in prefix command \"{}\" from message \"{}\": {}",
                                &ctx.command.name, &ctx.ctx.msg.content, error
                            );
                        }
                        ErrorContext::Command(CommandErrorContext::Slash(ctx)) => {
                            println!("Error in slash command \"{}\": {}", ctx.command.name, error);
                        }
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            slash_options: Default::default(),
            prefix_options: Default::default(),
        }
    }
}

pub enum Arguments<'a> {
    Prefix(&'a str),
    Slash(&'a [serenity::ApplicationCommandInteractionDataOption]),
}
