//! Simple module for the `FrameworkError` struct and its impls

use crate::serenity_prelude as serenity;

/// Any error that can occur while the bot runs. Either thrown by user code (those variants will
/// have an `error` field with your error type `E` in it), or originating from within the framework.
///
/// These errors are handled with the [`crate::FrameworkOptions::on_error`] callback
#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub enum FrameworkError<'a, U, E> {
    /// User code threw an error in user data setup
    #[non_exhaustive]
    Setup {
        /// Error which was thrown in the setup code
        error: E,
        /// The Framework passed to the event
        #[derivative(Debug = "ignore")]
        framework: &'a crate::Framework<U, E>,
        /// Discord Ready event data present during setup
        data_about_bot: &'a serenity::Ready,
        /// The serenity Context passed to the event
        #[derivative(Debug = "ignore")]
        ctx: &'a serenity::Context,
    },
    /// User code threw an error in generic event event handler
    #[non_exhaustive]
    EventHandler {
        /// Error which was thrown in the event handler code
        error: E,
        /// The serenity context passed to the event handler
        ctx: &'a serenity::Context,
        /// Which event was being processed when the error occurred
        event: &'a serenity::FullEvent,
        /// The Framework passed to the event
        #[derivative(Debug = "ignore")]
        framework: crate::FrameworkContext<'a, U, E>,
    },
    /// Error occurred during command execution
    #[non_exhaustive]
    Command {
        /// Error which was thrown in the command code
        error: E,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked without specifying a subcommand, but the command has `subcommand_required` set
    SubcommandRequired {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Panic occurred at any phase of command execution after constructing the `crate::Context`.
    ///
    /// This feature is intended as a last-resort safeguard to gracefully print an error message to
    /// the user on a panic. Panics should only be thrown for bugs in the code, don't use this for
    /// normal errors!
    #[non_exhaustive]
    CommandPanic {
        /// Panic payload which was thrown in the command code
        ///
        /// If a panic was thrown via [`std::panic::panic_any()`] and the payload was neither &str,
        /// nor String, the payload is `None`.
        ///
        /// The reason the original [`Box<dyn Any + Send>`] payload isn't provided here is that it
        /// would make [`FrameworkError`] not [`Sync`] anymore.
        payload: Option<String>,
        /// Command context
        ctx: crate::Context<'a, U, E>,
    },
    /// A command argument failed to parse from the Discord message or interaction content
    #[non_exhaustive]
    ArgumentParse {
        /// Error which was thrown by the parameter type's parsing routine
        error: Box<dyn std::error::Error + Send + Sync>,
        /// If applicable, the input on which parsing failed
        input: Option<String>,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Expected a certain argument type at a certain position in the unstructured list of
    /// arguments, but found something else.
    ///
    /// Most often the result of the bot not having registered the command in Discord, so Discord
    /// stores an outdated version of the command and its parameters.
    #[non_exhaustive]
    CommandStructureMismatch {
        /// Developer-readable description of the type mismatch
        description: &'static str,
        /// General context
        ctx: crate::ApplicationContext<'a, U, E>,
    },
    /// Command was invoked before its cooldown expired
    #[non_exhaustive]
    CooldownHit {
        /// Time until the command may be invoked for the next time in the given context
        remaining_cooldown: std::time::Duration,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the bot is lacking the permissions specified in
    /// [`crate::Command::required_permissions`]
    #[non_exhaustive]
    MissingBotPermissions {
        /// Which permissions in particular the bot is lacking for this command
        missing_permissions: serenity::Permissions,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the user is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    #[non_exhaustive]
    MissingUserPermissions {
        /// List of permissions that the user is lacking. May be None if retrieving the user's
        /// permissions failed
        missing_permissions: Option<serenity::Permissions>,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// A non-owner tried to invoke an owners-only command
    #[non_exhaustive]
    NotAnOwner {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a DM channel
    #[non_exhaustive]
    GuildOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a non-DM channel
    #[non_exhaustive]
    DmOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel wasn't a NSFW channel
    #[non_exhaustive]
    NsfwOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Provided pre-command check either errored, or returned false, so command execution aborted
    #[non_exhaustive]
    CommandCheckFailed {
        /// If execution wasn't aborted because of an error but because it successfully returned
        /// false, this field is None
        error: Option<E>,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// [`crate::PrefixFrameworkOptions::dynamic_prefix`] or
    /// [`crate::PrefixFrameworkOptions::stripped_dynamic_prefix`] returned an error
    #[non_exhaustive]
    DynamicPrefix {
        /// Error which was thrown in the dynamic prefix code
        error: E,
        /// General context
        #[derivative(Debug = "ignore")]
        ctx: crate::PartialContext<'a, U, E>,
        /// Message which the dynamic prefix callback was evaluated upon
        msg: &'a serenity::Message,
    },
    /// A message had the correct prefix but the following string was not a recognized command
    #[non_exhaustive]
    UnknownCommand {
        /// Serenity's Context
        #[derivative(Debug = "ignore")]
        ctx: &'a serenity::Context,
        /// The message in question
        msg: &'a serenity::Message,
        /// The prefix that was recognized
        prefix: &'a str,
        /// The rest of the message (after the prefix) which was not recognized as a command
        ///
        /// This is a single field instead of two fields (command name and args) due to subcommands
        msg_content: &'a str,
        /// Framework context
        #[derivative(Debug = "ignore")]
        framework: crate::FrameworkContext<'a, U, E>,
        /// See [`crate::Context::invocation_data`]
        #[derivative(Debug = "ignore")]
        invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
        /// Which event triggered the message parsing routine
        trigger: crate::MessageDispatchTrigger,
    },
    /// The command name from the interaction is unrecognized
    #[non_exhaustive]
    UnknownInteraction {
        #[derivative(Debug = "ignore")]
        /// Serenity's Context
        ctx: &'a serenity::Context,
        /// Framework context
        #[derivative(Debug = "ignore")]
        framework: crate::FrameworkContext<'a, U, E>,
        /// The interaction in question
        interaction: &'a serenity::CommandInteraction,
    },
    /// An error occurred in [`crate::PrefixFrameworkOptions::non_command_message`]
    #[non_exhaustive]
    NonCommandMessage {
        /// The error thrown by user code
        error: E,
        #[derivative(Debug = "ignore")]
        /// Serenity's Context
        ctx: &'a serenity::Context,
        /// Framework context
        #[derivative(Debug = "ignore")]
        framework: crate::FrameworkContext<'a, U, E>,
        /// The interaction in question
        msg: &'a serenity::Message,
    },
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    __NonExhaustive(std::convert::Infallible),
}

impl<'a, U, E> FrameworkError<'a, U, E> {
    /// Returns the [`serenity::Context`] of this error
    pub fn serenity_context(&self) -> &'a serenity::Context {
        match *self {
            Self::Setup { ctx, .. } => ctx,
            Self::EventHandler { ctx, .. } => ctx,
            Self::Command { ctx, .. } => ctx.serenity_context(),
            Self::SubcommandRequired { ctx } => ctx.serenity_context(),
            Self::CommandPanic { ctx, .. } => ctx.serenity_context(),
            Self::ArgumentParse { ctx, .. } => ctx.serenity_context(),
            Self::CommandStructureMismatch { ctx, .. } => ctx.serenity_context,
            Self::CooldownHit { ctx, .. } => ctx.serenity_context(),
            Self::MissingBotPermissions { ctx, .. } => ctx.serenity_context(),
            Self::MissingUserPermissions { ctx, .. } => ctx.serenity_context(),
            Self::NotAnOwner { ctx, .. } => ctx.serenity_context(),
            Self::GuildOnly { ctx, .. } => ctx.serenity_context(),
            Self::DmOnly { ctx, .. } => ctx.serenity_context(),
            Self::NsfwOnly { ctx, .. } => ctx.serenity_context(),
            Self::CommandCheckFailed { ctx, .. } => ctx.serenity_context(),
            Self::DynamicPrefix { ctx, .. } => ctx.serenity_context,
            Self::UnknownCommand { ctx, .. } => ctx,
            Self::UnknownInteraction { ctx, .. } => ctx,
            Self::NonCommandMessage { ctx, .. } => ctx,
            Self::__NonExhaustive(unreachable) => match unreachable {},
        }
    }

    /// Returns the [`crate::Context`] of this error, if it has one
    pub fn ctx(&self) -> Option<crate::Context<'a, U, E>> {
        Some(match *self {
            Self::Command { ctx, .. } => ctx,
            Self::SubcommandRequired { ctx } => ctx,
            Self::CommandPanic { ctx, .. } => ctx,
            Self::ArgumentParse { ctx, .. } => ctx,
            Self::CommandStructureMismatch { ctx, .. } => crate::Context::Application(ctx),
            Self::CooldownHit { ctx, .. } => ctx,
            Self::MissingBotPermissions { ctx, .. } => ctx,
            Self::MissingUserPermissions { ctx, .. } => ctx,
            Self::NotAnOwner { ctx, .. } => ctx,
            Self::GuildOnly { ctx, .. } => ctx,
            Self::DmOnly { ctx, .. } => ctx,
            Self::NsfwOnly { ctx, .. } => ctx,
            Self::CommandCheckFailed { ctx, .. } => ctx,
            Self::Setup { .. }
            | Self::EventHandler { .. }
            | Self::UnknownCommand { .. }
            | Self::UnknownInteraction { .. }
            | Self::NonCommandMessage { .. }
            | Self::DynamicPrefix { .. } => return None,
            Self::__NonExhaustive(unreachable) => match unreachable {},
        })
    }

    /// Calls the appropriate `on_error` function (command-specific or global) with this error
    pub async fn handle(self, framework_options: &crate::FrameworkOptions<U, E>) {
        let on_error = self
            .ctx()
            .and_then(|c| c.command().on_error)
            .unwrap_or(framework_options.on_error);
        on_error(self).await;
    }
}

/// Support functions for the macro, which can't create these #[non_exhaustive] enum variants
#[doc(hidden)]
impl<'a, U, E> FrameworkError<'a, U, E> {
    pub fn new_command(ctx: crate::Context<'a, U, E>, error: E) -> Self {
        Self::Command { error, ctx }
    }

    pub fn new_argument_parse(
        ctx: crate::Context<'a, U, E>,
        input: Option<String>,
        error: Box<dyn std::error::Error + Send + Sync>,
    ) -> Self {
        Self::ArgumentParse { error, input, ctx }
    }

    pub fn new_command_structure_mismatch(
        ctx: crate::ApplicationContext<'a, U, E>,
        description: &'static str,
    ) -> Self {
        Self::CommandStructureMismatch { description, ctx }
    }
}

/// Simple macro to deduplicate code. Can't be a function due to lifetime issues with `format_args`
macro_rules! full_command_name {
    ($ctx:expr) => {
        format_args!("{}{}", $ctx.prefix(), $ctx.command().qualified_name)
    };
}

impl<U, E: std::fmt::Display> std::fmt::Display for FrameworkError<'_, U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup {
                error: _,
                framework: _,
                data_about_bot: _,
                ctx: _,
            } => write!(f, "poise setup error"),
            Self::EventHandler { event, .. } => write!(
                f,
                "error in {} event event handler",
                event.snake_case_name()
            ),
            Self::Command { error: _, ctx } => {
                write!(f, "error in command `{}`", full_command_name!(ctx))
            }
            Self::SubcommandRequired { ctx } => {
                write!(
                    f,
                    "expected subcommand for command `{}`",
                    full_command_name!(ctx)
                )
            }
            Self::CommandPanic { ctx, payload: _ } => {
                write!(f, "panic in command `{}`", full_command_name!(ctx))
            }
            Self::ArgumentParse {
                error: _,
                input,
                ctx,
            } => write!(
                f,
                "failed to parse argument in command `{}` on input {:?}",
                full_command_name!(ctx),
                input
            ),
            Self::CommandStructureMismatch { description, ctx } => write!(
                f,
                "unexpected application command structure in command `{}`: {}",
                full_command_name!(crate::Context::Application(*ctx)),
                description
            ),
            Self::CooldownHit {
                remaining_cooldown,
                ctx,
            } => write!(
                f,
                "cooldown hit in command `{}` ({:?} remaining)",
                full_command_name!(ctx),
                remaining_cooldown
            ),
            Self::MissingBotPermissions {
                missing_permissions,
                ctx,
            } => write!(
                f,
                "bot is missing permisions ({}) to execute command `{}`",
                missing_permissions,
                full_command_name!(ctx),
            ),
            Self::MissingUserPermissions {
                missing_permissions,
                ctx,
            } => write!(
                f,
                "user is or may be missing permisions ({:?}) to execute command `{}`",
                missing_permissions,
                full_command_name!(ctx),
            ),
            Self::NotAnOwner { ctx } => write!(
                f,
                "owner-only command `{}` cannot be run by non-owners",
                full_command_name!(ctx)
            ),
            Self::GuildOnly { ctx } => write!(
                f,
                "guild-only command `{}` cannot run in DMs",
                full_command_name!(ctx)
            ),
            Self::DmOnly { ctx } => write!(
                f,
                "DM-only command `{}` cannot run in guilds",
                full_command_name!(ctx)
            ),
            Self::NsfwOnly { ctx } => write!(
                f,
                "nsfw-only command `{}` cannot run in non-nsfw channels",
                full_command_name!(ctx)
            ),
            Self::CommandCheckFailed { error: _, ctx } => write!(
                f,
                "pre-command check for command `{}` either denied access or errored",
                full_command_name!(ctx)
            ),
            Self::DynamicPrefix {
                error: _,
                ctx: _,
                msg,
            } => {
                write!(
                    f,
                    "dynamic prefix callback errored on message {:?}",
                    msg.content
                )
            }
            Self::UnknownCommand { msg_content, .. } => {
                write!(f, "unknown command `{}`", msg_content)
            }
            Self::UnknownInteraction { interaction, .. } => {
                write!(f, "unknown interaction `{}`", interaction.data.name)
            }
            Self::NonCommandMessage { msg, .. } => {
                write!(
                    f,
                    "error in non-command message handler in <@{}> (message ID {})",
                    msg.channel_id, msg.id
                )
            }
            Self::__NonExhaustive(unreachable) => match *unreachable {},
        }
    }
}

impl<'a, U: std::fmt::Debug, E: std::error::Error + 'static> std::error::Error
    for FrameworkError<'a, U, E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup { error, .. } => Some(error),
            Self::EventHandler { error, .. } => Some(error),
            Self::Command { error, .. } => Some(error),
            Self::SubcommandRequired { .. } => None,
            Self::CommandPanic { .. } => None,
            Self::ArgumentParse { error, .. } => Some(&**error),
            Self::CommandStructureMismatch { .. } => None,
            Self::CooldownHit { .. } => None,
            Self::MissingBotPermissions { .. } => None,
            Self::MissingUserPermissions { .. } => None,
            Self::NotAnOwner { .. } => None,
            Self::GuildOnly { .. } => None,
            Self::DmOnly { .. } => None,
            Self::NsfwOnly { .. } => None,
            Self::CommandCheckFailed { error, .. } => error.as_ref().map(|x| x as _),
            Self::DynamicPrefix { error, .. } => Some(error),
            Self::UnknownCommand { .. } => None,
            Self::UnknownInteraction { .. } => None,
            Self::NonCommandMessage { error, .. } => Some(error),
            Self::__NonExhaustive(unreachable) => match *unreachable {},
        }
    }
}
