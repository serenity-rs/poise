//! Plain data structs that define the framework configuration.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Wrapper around either [`SlashContext`] or [`PrefixContext`]
pub enum Context<'a, U, E> {
    Application(crate::ApplicationContext<'a, U, E>),
    Prefix(crate::PrefixContext<'a, U, E>),
}
impl<U, E> Clone for Context<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U, E> Copy for Context<'_, U, E> {}
impl<'a, U, E> From<crate::ApplicationContext<'a, U, E>> for Context<'a, U, E> {
    fn from(x: crate::ApplicationContext<'a, U, E>) -> Self {
        Self::Application(x)
    }
}
impl<'a, U, E> From<crate::PrefixContext<'a, U, E>> for Context<'a, U, E> {
    fn from(x: crate::PrefixContext<'a, U, E>) -> Self {
        Self::Prefix(x)
    }
}

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
            Self::Application(ctx) => ctx.discord,
            Self::Prefix(ctx) => ctx.discord,
        }
    }

    pub fn framework(&self) -> &crate::Framework<U, E> {
        match self {
            Self::Application(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    pub fn data(&self) -> &U {
        match self {
            Self::Application(ctx) => ctx.data,
            Self::Prefix(ctx) => ctx.data,
        }
    }

    pub fn channel_id(&self) -> serenity::ChannelId {
        match self {
            Self::Application(ctx) => ctx.interaction.channel_id,
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    pub fn guild_id(&self) -> Option<serenity::GuildId> {
        match self {
            Self::Application(ctx) => ctx.interaction.guild_id,
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Warnings: clones the entire Guild instance out of the cache
    pub fn guild(&self) -> Option<serenity::Guild> {
        self.guild_id()?.to_guild_cached(self.discord())
    }

    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::Application(ctx) => ctx.interaction.id.created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    /// Get the author of the command message or application command.
    pub fn author(&self) -> &serenity::User {
        match self {
            Self::Application(ctx) => &ctx.interaction.user,
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }

    /// Return a ID that uniquely identifies this command invocation.
    pub fn id(&self) -> u64 {
        match self {
            Self::Application(ctx) => ctx.interaction.id.0,
            Self::Prefix(ctx) => {
                let mut id = ctx.msg.id.0;
                if let Some(edited_timestamp) = ctx.msg.edited_timestamp {
                    // We replace the 42 datetime bits with msg.timestamp_edited so that the ID is
                    // unique even after edits

                    // Set existing datetime bits to zero
                    id &= !0 >> 42;

                    // Calculate Discord's datetime representation (millis since Discord epoch) and
                    // insert those bits into the ID
                    id |= ((edited_timestamp.timestamp_millis() - 1420070400000) as u64) << 22;
                }
                id
            }
        }
    }
}

pub enum CommandRef<'a, U, E> {
    Prefix(&'a crate::PrefixCommand<U, E>),
    Application(&'a crate::ApplicationCommand<U, E>),
}

impl<U, E> Clone for CommandRef<'_, U, E> {
    fn clone(&self) -> Self {
        match *self {
            Self::Prefix(x) => Self::Prefix(x),
            Self::Application(x) => Self::Application(x),
        }
    }
}

impl<U, E> Copy for CommandRef<'_, U, E> {}

impl<U, E> CommandRef<'_, U, E> {
    /// Yield name of this command, or, if context menu command, the context menu entry label
    pub fn name(self) -> &'static str {
        match self {
            Self::Prefix(x) => x.name,
            Self::Application(x) => x.slash_or_context_menu_name(),
        }
    }
}

pub enum CommandErrorContext<'a, U, E> {
    Prefix(crate::PrefixCommandErrorContext<'a, U, E>),
    Application(crate::ApplicationCommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for CommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Prefix(x) => Self::Prefix(x.clone()),
            Self::Application(x) => Self::Application(x.clone()),
        }
    }
}

impl<'a, U, E> From<crate::PrefixCommandErrorContext<'a, U, E>> for CommandErrorContext<'a, U, E> {
    fn from(x: crate::PrefixCommandErrorContext<'a, U, E>) -> Self {
        Self::Prefix(x)
    }
}

impl<'a, U, E> From<crate::ApplicationCommandErrorContext<'a, U, E>>
    for CommandErrorContext<'a, U, E>
{
    fn from(x: crate::ApplicationCommandErrorContext<'a, U, E>) -> Self {
        Self::Application(x)
    }
}

impl<'a, U, E> CommandErrorContext<'a, U, E> {
    pub fn command(&self) -> CommandRef<'_, U, E> {
        match self {
            Self::Prefix(x) => CommandRef::Prefix(x.command),
            Self::Application(x) => CommandRef::Application(x.command),
        }
    }

    pub fn while_checking(&self) -> bool {
        match self {
            Self::Prefix(x) => x.while_checking,
            Self::Application(x) => x.while_checking,
        }
    }
    pub fn ctx(&self) -> Context<'a, U, E> {
        match self {
            Self::Prefix(x) => Context::Prefix(x.ctx),
            Self::Application(x) => Context::Application(x.ctx),
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

pub struct CommandBuilder<U, E> {
    prefix_command: Option<crate::PrefixCommandMeta<U, E>>,
    slash_command: Option<crate::SlashCommand<U, E>>,
    context_menu_command: Option<crate::ContextMenuCommand<U, E>>,
}

impl<U, E> CommandBuilder<U, E> {
    pub fn category(&mut self, category: &'static str) -> &mut Self {
        if let Some(prefix_command) = &mut self.prefix_command {
            prefix_command.category = Some(category);
        }
        self
    }

    pub fn subcommand(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut Self) -> &mut Self,
    ) -> &mut Self {
        // STUB: do slash support

        let crate::CommandDefinition {
            prefix: prefix_command,
            slash: slash_command,
            context_menu: context_menu_command,
        } = definition;

        let prefix_command = prefix_command.map(|prefix_command| crate::PrefixCommandMeta {
            command: prefix_command,
            category: None,
            subcommands: Vec::new(),
        });

        let mut builder = CommandBuilder {
            prefix_command,
            slash_command,
            context_menu_command,
        };
        meta_builder(&mut builder);

        if let (Some(parent), Some(subcommand)) = (&mut self.prefix_command, builder.prefix_command)
        {
            parent.subcommands.push(subcommand);
        }

        self
    }
}

pub struct FrameworkOptions<U, E> {
    /// Provide a callback to be invoked when any user code yields an error.
    pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    pub pre_command: fn(Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Default set of allowed mentions to use for all responses
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a crate::Event<'a>,
        &'a crate::Framework<U, E>,
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Application command specific options.
    pub application_options: crate::ApplicationFrameworkOptions<U, E>,
    /// Prefix command specific options.
    pub prefix_options: crate::PrefixFrameworkOptions<U, E>,
    /// User IDs which are allowed to use owners_only commands
    pub owners: std::collections::HashSet<serenity::UserId>,
}

impl<U, E> FrameworkOptions<U, E> {
    /// Add a command definition, which can include a prefix implementation, slash implementation,
    /// and context menu implementation, to the framework.
    ///
    /// To define subcommands or other meta information, pass a closure that calls the command
    /// builder
    ///
    /// ```rust
    /// # mod misc {
    /// #     type Error = Box<dyn std::error::Error + Send + Sync>;
    /// #     #[poise::command]
    /// #     pub async fn ping(ctx: poise::PrefixContext<'_, (), Error>) -> Result<(), Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// # use poise::FrameworkOptions;
    /// let mut options = FrameworkOptions::default();
    /// options.command(misc::ping(), |f| f.category("Miscellaneous"));
    /// ```
    pub fn command(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut CommandBuilder<U, E>) -> &mut CommandBuilder<U, E>,
    ) {
        let crate::CommandDefinition {
            prefix: prefix_command,
            slash: slash_command,
            context_menu: context_menu_command,
        } = definition;

        let prefix_command = prefix_command.map(|prefix_command| crate::PrefixCommandMeta {
            command: prefix_command,
            category: None,
            subcommands: Vec::new(),
        });

        let mut builder = CommandBuilder {
            prefix_command,
            slash_command,
            context_menu_command,
        };
        meta_builder(&mut builder);

        if let Some(prefix_command) = builder.prefix_command {
            self.prefix_options.commands.push(prefix_command);
        }
        if let Some(slash_command) = builder.slash_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommand::Slash(slash_command));
        }
        if let Some(context_menu_command) = builder.context_menu_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommand::ContextMenu(context_menu_command));
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
                        ErrorContext::Command(CommandErrorContext::Application(ctx)) => {
                            match &ctx.command {
                                crate::ApplicationCommand::Slash(cmd) => {
                                    println!("Error in slash command \"{}\": {}", cmd.name, error)
                                }
                                crate::ApplicationCommand::ContextMenu(cmd) => println!(
                                    "Error in context menu command \"{}\": {}",
                                    cmd.name, error
                                ),
                            }
                        }
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            allowed_mentions: Some({
                let mut f = serenity::CreateAllowedMentions::default();
                // Only support direct user pings by default
                f.empty_parse().parse(serenity::ParseValue::Users);
                f
            }),
            application_options: Default::default(),
            prefix_options: Default::default(),
            owners: Default::default(),
        }
    }
}

pub enum Arguments<'a> {
    Prefix(&'a str),
    Slash(&'a [serenity::ApplicationCommandInteractionDataOption]),
}

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
pub struct CommandDefinition<U, E> {
    pub prefix: Option<crate::PrefixCommand<U, E>>,
    pub slash: Option<crate::SlashCommand<U, E>>,
    pub context_menu: Option<crate::ContextMenuCommand<U, E>>,
}
