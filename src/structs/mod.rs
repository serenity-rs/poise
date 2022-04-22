//! Plain data structs that define the framework configuration.

mod context;
pub use context::*;

mod framework_options;
pub use framework_options::*;

mod command;
pub use command::*;

use crate::serenity_prelude as serenity;

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
        framework: &'a crate::Framework<U, E>,
    },
    /// Error occured during command execution
    Command {
        /// Error which was thrown in the command code
        error: E,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// A command argument failed to parse from the Discord message or interaction content
    ArgumentParse {
        /// Error which was thrown by the parameter type's parsing routine
        error: Box<dyn std::error::Error + Send + Sync>,
        /// If applicable, the input on which parsing failed
        input: Option<String>,
        /// General context
        ctx: Context<'a, U, E>,
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
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the bot is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingBotPermissions {
        /// Which permissions in particular the bot is lacking for this command
        missing_permissions: serenity::Permissions,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the user is lacking the permissions specified in
    /// [`crate::Command::required_bot_permissions`]
    MissingUserPermissions {
        /// List of permissions that the user is lacking. May be None if retrieving the user's
        /// permissions failed
        missing_permissions: Option<serenity::Permissions>,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// A non-owner tried to invoke an owners-only command
    NotAnOwner {
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a DM channel
    GuildOnly {
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the channel was a non-DM channel
    DmOnly {
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the channel wasn't a NSFW channel
    NsfwOnly {
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Provided pre-command check either errored, or returned false, so command execution aborted
    CommandCheckFailed {
        /// If execution wasn't aborted because of an error but because it successfully returned
        /// false, this field is None
        error: Option<E>,
        /// General context
        ctx: Context<'a, U, E>,
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
