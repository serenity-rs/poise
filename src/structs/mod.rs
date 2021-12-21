//! Plain data structs that define the framework configuration.

mod context;
pub use context::*;

mod framework_options;
pub use framework_options::*;

use crate::{serenity_prelude as serenity, BoxFuture};

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

impl<'a, U, E> CommandRef<'a, U, E> {
    /// Yield `name` of this command, or, if context menu command, the context menu entry label
    pub fn name(self) -> &'static str {
        match self {
            Self::Prefix(x) => x.name,
            Self::Application(x) => x.slash_or_context_menu_name(),
        }
    }

    /// Yield `id` of this command.
    pub fn id(self) -> &'a std::sync::Arc<CommandId<U, E>> {
        match self {
            Self::Prefix(x) => &x.id,
            Self::Application(x) => x.id(),
        }
    }
}

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
#[derive(Default, Clone, Debug)]
pub struct CommandDefinition<U, E> {
    /// Generated prefix command, if it was enabled
    pub prefix: Option<crate::PrefixCommand<U, E>>,
    /// Generated slash command, if it was enabled
    pub slash: Option<crate::SlashCommand<U, E>>,
    /// Generated context menu command, if it was enabled
    pub context_menu: Option<crate::ContextMenuCommand<U, E>>,
    /// Implementation type agnostic data that is always present
    pub id: std::sync::Arc<CommandId<U, E>>,
}

/// A view into a command definition with its different implementations
#[derive(Default, Debug)]
pub struct CommandDefinitionRef<'a, U, E> {
    /// Prefix implementation of the command
    pub prefix: Option<&'a crate::PrefixCommandMeta<U, E>>,
    /// Slash implementation of the command
    pub slash: Option<&'a crate::SlashCommandMeta<U, E>>,
    /// Context menu implementation of the command
    pub context_menu: Option<&'a crate::ContextMenuCommand<U, E>>,
    /// Implementation type agnostic data that is always present
    pub id: std::sync::Arc<CommandId<U, E>>,
}

/// This struct holds all data shared across different command types of the same implementation.
///
/// For example with a `#[command(prefix_command, slash_command)]`, the generated
/// [`crate::PrefixCommand`] and [`crate::SlashCommand`] will both contain an `Arc<CommandId<U, E>>`
/// pointing to the same [`CommandId`] instance.
#[derive(Default)]
pub struct CommandId<U, E> {
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
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub multiline_help: Option<fn() -> String>,
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
    /// Command-specific override for [`crate::FrameworkOptions::on_error`]
    pub on_error: Option<fn(FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
}

impl<U, E> std::fmt::Debug for CommandId<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            identifying_name,
            category,
            hide_in_help,
            inline_help,
            multiline_help,
            cooldowns,
            required_permissions,
            required_bot_permissions,
            owners_only,
            on_error,
            check,
        } = self;

        f.debug_struct("CommandId")
            .field("identifying_name", identifying_name)
            .field("category", category)
            .field("hide_in_help", hide_in_help)
            .field("inline_help", inline_help)
            .field("multiline_help", multiline_help)
            .field("cooldowns", cooldowns)
            .field("required_permissions", required_permissions)
            .field("required_bot_permissions", required_bot_permissions)
            .field("owners_only", owners_only)
            .field("on_error", &on_error.map(|f| f as *const ()))
            .field("check", &check.map(|f| f as *const ()))
            .finish()
    }
}

/// Used for command errors to store the specific operation in a command's execution where an
/// error occured
#[derive(Copy, Clone, Debug, Hash, PartialEq, Eq)]
pub enum CommandErrorLocation {
    /// Error occured in the main command code
    Body,
    /// Error occured in one of the pre-command checks
    Check,
    /// Error occured in a parameter autocomplete callback
    Autocomplete,
}

/// Any error that can occur while the bot runs. Either thrown by user code (those variants will
/// have an `error` field with your error type `E` in it), or originating from within the framework.
///
/// These errors are handled with the [`crate::FrameworkOptions::on_error`] callback
#[derive(Debug)]
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
        /// Which event was being processed when the error occurred
        event: &'a crate::Event<'a>,
    },
    /// User code threw an error in bot command
    Command {
        /// Error which was thrown in the command code
        error: E,
        /// In which part of the command execution the error occured
        location: crate::CommandErrorLocation,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// A command argument failed to parse from the Discord message or interaction content
    ArgumentParse {
        /// Error which was thrown by the parameter type's parsing routine
        error: Box<dyn std::error::Error + Send + Sync>,
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
        error: &'static str,
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
    /// `crate::CommandId::required_bot_permissions`
    MissingBotPermissions {
        /// Which permissions in particular the bot is lacking for this command
        missing_permissions: serenity::Permissions,
        /// General context
        ctx: Context<'a, U, E>,
    },
    /// Command was invoked but the user is lacking the permissions specified in
    /// `crate::CommandId::required_bot_permissions`
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
    /// Provided pre-command check didn't succeed, so command execution aborted
    CommandCheckFailed {
        /// General context
        ctx: Context<'a, U, E>,
    },
}
