//! Plain data structs that define the framework configuration.

use crate::{serenity_prelude as serenity, BoxFuture};

/// Wrapper around either [`crate::ApplicationContext`] or [`crate::PrefixContext`]
pub enum Context<'a, U, E> {
    /// Application command context
    Application(crate::ApplicationContext<'a, U, E>),
    /// Prefix command context
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
impl<'a, U, E> Context<'a, U, E> {
    /// Defer the response, giving the bot multiple minutes to respond without the user seeing an
    /// "interaction failed error".
    ///
    /// Also sets the [`crate::ApplicationContext::has_sent_initial_response`] flag so subsequent
    /// responses will be sent in the correct manner.
    ///
    /// No-op if this is an autocomplete context
    ///
    /// This will make the response public; to make it ephemeral, use [`Self::defer_ephemeral()`].
    pub async fn defer(self) -> Result<(), serenity::Error> {
        if let Self::Application(ctx) = self {
            ctx.defer_response(false).await?;
        }
        Ok(())
    }

    /// See [`Self::defer()`]
    ///
    /// This will make the response ephemeral; to make it public, use [`Self::defer()`].
    pub async fn defer_ephemeral(self) -> Result<(), serenity::Error> {
        if let Self::Application(ctx) = self {
            ctx.defer_response(true).await?;
        }
        Ok(())
    }

    /// If this is an application command, [`Self::defer()`] is called
    ///
    /// If this is a prefix command, a typing broadcast is started until the return value is
    /// dropped.
    // #[must_use = "The typing broadcast will only persist if you store it"] // currently doesn't work
    pub async fn defer_or_broadcast(self) -> Result<Option<serenity::Typing>, serenity::Error> {
        Ok(match self {
            Self::Application(ctx) => {
                ctx.defer_response(false).await?;
                None
            }
            Self::Prefix(ctx) => Some(ctx.msg.channel_id.start_typing(&ctx.discord.http)?),
        })
    }

    /// Shorthand of [`crate::say_reply`]
    pub async fn say(
        self,
        text: impl Into<String>,
    ) -> Result<Option<crate::ReplyHandle<'a>>, serenity::Error> {
        crate::say_reply(self, text).await
    }

    /// Shorthand of [`crate::send_reply`]
    pub async fn send<'b>(
        self,
        builder: impl for<'c> FnOnce(&'c mut crate::CreateReply<'b>) -> &'c mut crate::CreateReply<'b>,
    ) -> Result<Option<crate::ReplyHandle<'a>>, serenity::Error> {
        crate::send_reply(self, builder).await
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

impl<'a, U, E> Context<'a, U, E> {
    /// Return the stored [`serenity::Context`] within the underlying context type.
    pub fn discord(&self) -> &'a serenity::Context {
        match self {
            Self::Application(ctx) => ctx.discord,
            Self::Prefix(ctx) => ctx.discord,
        }
    }

    /// Return a read-only reference to [`crate::Framework`].
    pub fn framework(&self) -> &'a crate::Framework<U, E> {
        match self {
            Self::Application(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    /// Return a reference to your custom user data
    pub fn data(&self) -> &'a U {
        match self {
            Self::Application(ctx) => ctx.data,
            Self::Prefix(ctx) => ctx.data,
        }
    }

    /// Return the channel ID of this context
    pub fn channel_id(&self) -> serenity::ChannelId {
        match self {
            Self::Application(ctx) => ctx.interaction.channel_id(),
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    /// Returns the guild ID of this context, if we are inside a guild
    pub fn guild_id(&self) -> Option<serenity::GuildId> {
        match self {
            Self::Application(ctx) => ctx.interaction.guild_id(),
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the guild of this context, if we are inside a guild.
    ///
    /// Warning: clones the entire Guild instance out of the cache
    pub fn guild(&self) -> Option<serenity::Guild> {
        self.guild_id()?.to_guild_cached(self.discord())
    }

    /// Return the datetime of the invoking message or interaction
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::Application(ctx) => ctx.interaction.id().created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    /// Get the author of the command message or application command.
    pub fn author(&self) -> &'a serenity::User {
        match self {
            Self::Application(ctx) => ctx.interaction.user(),
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }

    /// Return a ID that uniquely identifies this command invocation.
    pub fn id(&self) -> u64 {
        match self {
            Self::Application(ctx) => ctx.interaction.id().0,
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

    /// Returns a reference to the command.
    pub fn command(&self) -> Option<crate::CommandRef<'a, U, E>> {
        Some(match self {
            Self::Prefix(x) => crate::CommandRef::Prefix(x.command?),
            Self::Application(x) => crate::CommandRef::Application(x.command),
        })
    }
}

/// A reference to either a prefix or application command.
pub enum CommandRef<'a, U, E> {
    /// Prefix command
    Prefix(&'a crate::PrefixCommand<U, E>),
    /// Application command
    Application(crate::ApplicationCommand<'a, U, E>),
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

/// Context of an error in user code
///
/// Contains slightly different data depending on where the error was raised
pub enum CommandErrorContext<'a, U, E> {
    /// Prefix command specific error context
    Prefix(crate::PrefixCommandErrorContext<'a, U, E>),
    /// Application command specific error context
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
    /// Returns a reference to the command during whose execution the error occured.
    pub fn command(&self) -> CommandRef<'_, U, E> {
        match self {
            Self::Prefix(x) => CommandRef::Prefix(x.command),
            Self::Application(x) => CommandRef::Application(x.ctx.command),
        }
    }

    /// Whether the error occured in a pre-command check or during execution
    pub fn location(&self) -> crate::CommandErrorLocation {
        match self {
            Self::Prefix(x) => x.location,
            Self::Application(x) => x.location,
        }
    }

    /// Further command context
    pub fn ctx(&self) -> Context<'a, U, E> {
        match self {
            Self::Prefix(x) => Context::Prefix(x.ctx),
            Self::Application(x) => Context::Application(x.ctx),
        }
    }
}

/// Contains the location of the error with location-specific context
pub enum ErrorContext<'a, U, E> {
    /// Error in user data setup
    Setup,
    /// Error in generic event listener
    Listener(&'a crate::Event<'a>),
    /// Error in bot command
    Command(CommandErrorContext<'a, U, E>),
    /// Error in autocomplete callback
    Autocomplete(crate::ApplicationCommandErrorContext<'a, U, E>),
}

impl<U, E> Clone for ErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        match self {
            Self::Setup => Self::Setup,
            Self::Listener(x) => Self::Listener(x),
            Self::Command(x) => Self::Command(x.clone()),
            Self::Autocomplete(x) => Self::Autocomplete(x.clone()),
        }
    }
}

/// Builder struct to add a command to the framework
pub struct CommandBuilder<U, E> {
    prefix_command: Option<crate::PrefixCommandMeta<U, E>>,
    slash_command: Option<crate::SlashCommandMeta<U, E>>,
    context_menu_command: Option<crate::ContextMenuCommand<U, E>>,
}

impl<U, E> CommandBuilder<U, E> {
    /// **Deprecated**
    #[deprecated = "Please use `category = \"...\"` on the command attribute instead"]
    pub fn category(&mut self, _category: &'static str) -> &mut Self {
        panic!("Please use `category = \"...\"` on the command attribute instead")
    }

    /// Insert a subcommand
    pub fn subcommand(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut Self) -> &mut Self,
    ) -> &mut Self {
        let crate::CommandDefinition {
            prefix: mut prefix_command,
            slash: mut slash_command,
            context_menu: mut context_menu_command,
        } = definition;

        // Make sure every implementation points to the same CommandId (they may have different
        // IDs if each implemented comes from a different function, like rustbot's rustify)
        let id = if let Some(prefix_command) = &prefix_command {
            prefix_command.id.clone()
        } else if let Some(slash_command) = &slash_command {
            slash_command.id.clone()
        } else if let Some(context_menu_command) = &context_menu_command {
            context_menu_command.id.clone()
        } else {
            panic!("Empty command definition (no implementations)");
        };
        if let Some(prefix_command) = &mut prefix_command {
            prefix_command.id = id.clone();
        }
        if let Some(slash_command) = &mut slash_command {
            slash_command.id = id.clone();
        }
        if let Some(context_menu_command) = &mut context_menu_command {
            context_menu_command.id = id.clone();
        }

        let prefix_command = prefix_command.map(|prefix_command| crate::PrefixCommandMeta {
            command: prefix_command,
            subcommands: Vec::new(),
        });

        let slash_command = slash_command.map(crate::SlashCommandMeta::Command);

        let mut builder = CommandBuilder {
            prefix_command,
            slash_command,
            context_menu_command,
        };
        meta_builder(&mut builder);

        // Nested if's to compile on Rust 1.48
        if let Some(parent) = &mut self.prefix_command {
            if let Some(subcommand) = builder.prefix_command {
                parent.subcommands.push(subcommand);
            }
        }

        if let Some(parent) = &mut self.slash_command {
            if let Some(subcommand) = builder.slash_command {
                match parent {
                    crate::SlashCommandMeta::CommandGroup { subcommands, .. } => {
                        subcommands.push(subcommand);
                    }
                    crate::SlashCommandMeta::Command(cmd) => {
                        *parent = crate::SlashCommandMeta::CommandGroup {
                            name: cmd.name,
                            description: cmd.description,
                            subcommands: vec![subcommand],
                            id,
                        };
                    }
                }
            }
        }

        self
    }
}

/// Framework configuration
pub struct FrameworkOptions<U, E> {
    /// Provide a callback to be invoked when any user code yields an error.
    pub on_error: fn(E, ErrorContext<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    pub pre_command: fn(Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called after every command
    pub post_command: fn(Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// If individual commands add their own check, both callbacks are run and must return true.
    pub command_check: Option<fn(Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Called when a command is invoked before its cooldown has expired
    pub cooldown_hit:
        Option<fn(Context<'_, U, E>, std::time::Duration) -> BoxFuture<'_, Result<(), E>>>,
    /// Called if the bot is lacking any of the permissions specified in
    /// [`crate::CommandId::required_bot_permissions`]. The list of missing permissions is given as
    /// an argument.
    pub missing_bot_permissions_handler:
        fn(Context<'_, U, E>, serenity::Permissions) -> BoxFuture<'_, Result<(), E>>,
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
    /// #     #[poise::command(prefix_command)]
    /// #     pub async fn ping(ctx: poise::Context<'_, (), Error>) -> Result<(), Error> {
    /// #         Ok(())
    /// #     }
    /// # }
    /// # use poise::FrameworkOptions;
    /// let mut options = FrameworkOptions::default();
    /// options.command(misc::ping(), |f| f);
    /// ```
    pub fn command(
        &mut self,
        definition: crate::CommandDefinition<U, E>,
        meta_builder: impl FnOnce(&mut CommandBuilder<U, E>) -> &mut CommandBuilder<U, E>,
    ) {
        // TODO: remove duplication with CommandBuilder::subcommand

        // Unpack command implementations
        let crate::CommandDefinition {
            prefix: mut prefix_command,
            slash: mut slash_command,
            context_menu: mut context_menu_command,
        } = definition;

        // Make sure every implementation points to the same CommandId (they may have different
        // IDs if each implemented comes from a different function, like rustbot's rustify)
        let id = if let Some(prefix_command) = &prefix_command {
            prefix_command.id.clone()
        } else if let Some(slash_command) = &slash_command {
            slash_command.id.clone()
        } else if let Some(context_menu_command) = &context_menu_command {
            context_menu_command.id.clone()
        } else {
            panic!("Empty command definition (no implementations)");
        };
        if let Some(prefix_command) = &mut prefix_command {
            prefix_command.id = id.clone();
        }
        if let Some(slash_command) = &mut slash_command {
            slash_command.id = id.clone();
        }
        if let Some(context_menu_command) = &mut context_menu_command {
            context_menu_command.id = id;
        }

        // Wrap the commands in their meta structs
        let prefix_command = prefix_command.map(|prefix_command| crate::PrefixCommandMeta {
            command: prefix_command,
            subcommands: Vec::new(),
        });
        let slash_command = slash_command.map(crate::SlashCommandMeta::Command);

        // Run the command builder on the meta structs to fill in metadata
        let mut builder = CommandBuilder {
            prefix_command,
            slash_command,
            context_menu_command,
        };
        meta_builder(&mut builder);

        // Insert command implementations
        if let Some(prefix_command) = builder.prefix_command {
            self.prefix_options.commands.push(prefix_command);
        }
        if let Some(slash_command) = builder.slash_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommandTree::Slash(slash_command));
        }
        if let Some(context_menu_command) = builder.context_menu_command {
            self.application_options
                .commands
                .push(crate::ApplicationCommandTree::ContextMenu(
                    context_menu_command,
                ));
        }
    }
}

async fn default_error_handler<U, E>(error: E, ctx: ErrorContext<'_, U, E>)
where
    U: Send + Sync,
    E: std::fmt::Display + Send,
{
    match ctx {
        ErrorContext::Setup => println!("Error in user data setup: {}", error),
        ErrorContext::Listener(event) => println!(
            "User event listener encountered an error on {} event: {}",
            event.name(),
            error
        ),
        ErrorContext::Command(CommandErrorContext::Prefix(err_ctx)) => {
            println!(
                "Error in prefix command \"{}\" from message \"{}\": {}",
                &err_ctx.command.name, &err_ctx.ctx.msg.content, error
            );
        }
        ErrorContext::Command(CommandErrorContext::Application(err_ctx)) => {
            match &err_ctx.ctx.command {
                crate::ApplicationCommand::Slash(cmd) => {
                    println!("Error in slash command \"{}\": {}", cmd.name, error)
                }
                crate::ApplicationCommand::ContextMenu(cmd) => {
                    println!("Error in context menu command \"{}\": {}", cmd.name, error)
                }
            }
        }
        ErrorContext::Autocomplete(err_ctx) => match &err_ctx.ctx.command {
            crate::ApplicationCommand::Slash(cmd) => {
                println!("Error in slash command \"{}\": {}", cmd.name, error)
            }
            crate::ApplicationCommand::ContextMenu(cmd) => {
                println!("Error in context menu command \"{}\": {}", cmd.name, error)
            }
        },
    }
}

impl<U: Send + Sync, E: std::fmt::Display + Send> Default for FrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            on_error: |error, ctx| Box::pin(default_error_handler(error, ctx)),
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            post_command: |_| Box::pin(async {}),
            command_check: None,
            cooldown_hit: Some(|ctx, cooldown_left| {
                Box::pin(async move {
                    let msg = format!(
                        "You're too fast. Please wait {} seconds before retrying",
                        cooldown_left.as_secs()
                    );
                    let _: Result<_, _> = ctx.send(|b| b.content(msg).ephemeral(true)).await;

                    Ok(())
                })
            }),
            missing_bot_permissions_handler: |ctx, missing_permissions| {
                Box::pin(async move {
                    let msg = format!(
                        "Command cannot be executed because the bot is lacking permissions: {}",
                        missing_permissions,
                    );
                    let _: Result<_, _> = ctx.send(|b| b.content(msg).ephemeral(true)).await;

                    Ok(())
                })
            },
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

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
pub struct CommandDefinition<U, E> {
    /// Generated prefix command, if it was enabled
    pub prefix: Option<crate::PrefixCommand<U, E>>,
    /// Generated slash command, if it was enabled
    pub slash: Option<crate::SlashCommand<U, E>>,
    /// Generated context menu command, if it was enabled
    pub context_menu: Option<crate::ContextMenuCommand<U, E>>,
}

/// A view into a command definition with its different implementations
pub struct CommandDefinitionRef<'a, U, E> {
    /// Prefix implementation of the command
    pub prefix: Option<&'a crate::PrefixCommandMeta<U, E>>,
    /// Slash implementation of the command
    pub slash: Option<&'a crate::SlashCommandMeta<U, E>>,
    /// Context menu implementation of the command
    pub context_menu: Option<&'a crate::ContextMenuCommand<U, E>>,
    /// Implementation type agnostic data that is always present
    pub id: std::sync::Arc<CommandId>,
}

/// This struct holds all data shared across different command types of the same implementation.
///
/// For example with a `#[command(prefix_command, slash_command)]`, the generated
/// [`crate::PrefixCommand`] and [`crate::SlashCommand`] will both contain an `Arc<CommandId>`
/// pointing to the same [`CommandId`] instance.
pub struct CommandId {
    /// A string to identify this particular command within a list of commands.
    ///
    /// Can be configured via the [`crate::command`] macro (though it's probably not needed for most
    /// bots). If not explicitly configured, it falls back to prefix command name, slash command
    /// name, or context menu command name (in that order).
    pub identifying_name: String,
    /// Identifier for the category that this command will be displayed in for help commands.
    pub category: Option<&'static str>,
    /// Whether to hide this command in help menus.
    pub hide_in_help: bool,
    /// Short description of the command. Displayed inline in help menus and similar.
    pub inline_help: Option<&'static str>,
    /// Handles command cooldowns. Mainly for framework internal use
    pub cooldowns: std::sync::Mutex<crate::Cooldowns>,
    /// Permissions which users must have to invoke this command.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_permissions: serenity::Permissions,
    /// Permissions without which command execution will fail. You can set this to fail early and
    /// give a descriptive error message in case the
    /// bot hasn't been assigned the minimum permissions by the guild admin.
    ///
    /// Set to [`serenity::Permissions::empty()`] by default
    pub required_bot_permissions: serenity::Permissions,
    /// If true, only users from the [owners list](crate::FrameworkOptions::owners) may use this
    /// command.
    pub owners_only: bool,
}

/// Used for command errors to store the specific operation in a command's execution where an
/// error occured
#[derive(Copy, Clone)]
pub enum CommandErrorLocation {
    /// Error occured in the main command code
    Body,
    /// Error occured in one of the pre-command checks
    Check,
    /// Error occured in a parameter autocomplete callback
    Autocomplete,
    /// Error occured in [`crate::FrameworkOptions::cooldown_hit`]
    CooldownCallback,
    /// Error occured in [`crate::FrameworkOptions::missing_bot_permissions_handler`]
    MissingBotPermissionsCallback,
}
