use crate::serenity_prelude as serenity;

/// Any error that can occur while the bot runs. Either thrown by user code (those variants will
/// have an `error` field with your error type `E` in it), or originating from within the framework.
///
/// These errors are handled with the [`crate::FrameworkOptions::on_error`] callback
#[derive(derivative::Derivative)]
#[derivative(Debug)]
pub enum FrameworkError<'a, U, E> {
    /// User code threw an error in user data setup
    Setup {
        /// Error which was thrown in the setup code
        error: E,
    },
    /// User code threw an error in generic event listener
    Listener {
        /// Error which was thrown in the listener code
        error: E,
        /// The serenity Context passed to the event
        #[derivative(Debug = "ignore")]
        ctx: serenity::Context,
        /// Which event was being processed when the error occurred
        event: &'a crate::Event<'a>,
        /// The Framework passed to the event
        #[derivative(Debug = "ignore")]
        framework: crate::FrameworkContext<'a, U, E>,
    },
    /// Error occured during command execution
    Command {
        /// Error which was thrown in the command code
        error: E,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// A command argument failed to parse from the Discord message or interaction content
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
    CommandStructureMismatch {
        /// Developer-readable description of the type mismatch
        description: &'static str,
        /// General context
        ctx: crate::ApplicationContext<'a, U, E>,
    },
    /// Command was invoked before its cooldown expired
    CooldownHit {
        /// Time until the command may be invoked for the next time in the given context
        remaining_cooldown: std::time::Duration,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the bot is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingBotPermissions {
        /// Which permissions in particular the bot is lacking for this command
        missing_permissions: serenity::Permissions,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the user is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingUserPermissions {
        /// List of permissions that the user is lacking. May be None if retrieving the user's
        /// permissions failed
        missing_permissions: Option<serenity::Permissions>,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// A non-owner tried to invoke an owners-only command
    NotAnOwner {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a DM channel
    GuildOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a non-DM channel
    DmOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Command was invoked but the channel wasn't a NSFW channel
    NsfwOnly {
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// Provided pre-command check either errored, or returned false, so command execution aborted
    CommandCheckFailed {
        /// If execution wasn't aborted because of an error but because it successfully returned
        /// false, this field is None
        error: Option<E>,
        /// General context
        ctx: crate::Context<'a, U, E>,
    },
    /// [`crate::PrefixFrameworkOptions::dynamic_prefix`] or
    /// [`crate::PrefixFrameworkOptions::stripped_dynamic_prefix`] returned an error
    DynamicPrefix {
        /// Error which was thrown in the dynamic prefix code
        error: E,
    },
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    __NonExhaustive,
}

macro_rules! full_command_name {
    ($ctx:expr) => {
        format_args!("{}{}", $ctx.prefix(), $ctx.command().qualified_name)
    };
}

impl<U, E: std::fmt::Display> std::fmt::Display for FrameworkError<'_, U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Setup { error: _ } => write!(f, "poise setup error"),
            Self::Listener {
                error: _,
                ctx: _,
                event,
                framework: _,
            } => write!(f, "error in {} event listener", event.name()),
            Self::Command { error: _, ctx } => {
                write!(f, "error in command `{}`", full_command_name!(ctx))
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
            Self::DynamicPrefix { error: _ } => write!(f, "dynamic prefix callback errored"),
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

impl<'a, U: std::fmt::Debug, E: std::error::Error + 'static> std::error::Error
    for FrameworkError<'a, U, E>
{
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Setup { error, .. } => Some(error),
            Self::Listener { error, .. } => Some(error),
            Self::Command { error, .. } => Some(error),
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
            Self::__NonExhaustive => unreachable!(),
        }
    }
}
