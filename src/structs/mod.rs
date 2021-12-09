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
    // TODO: remove and just use Self::Command instead?
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

/// Type returned from `#[poise::command]` annotated functions, which contains all of the generated
/// prefix and application commands
#[derive(Default, Clone)]
pub struct CommandDefinition<U, E> {
    /// Generated prefix command, if it was enabled
    pub prefix: Option<crate::PrefixCommand<U, E>>,
    /// Generated slash command, if it was enabled
    pub slash: Option<crate::SlashCommand<U, E>>,
    /// Generated context menu command, if it was enabled
    pub context_menu: Option<crate::ContextMenuCommand<U, E>>,
}

/// A view into a command definition with its different implementations
#[derive(Default)]
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
    pub on_error: Option<fn(E, CommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
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
            .field("on_error", &on_error.map(|_| "<function pointer>"))
            .field("check", &check.map(|_| "<function pointer>"))
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
    /// Error occured in [`crate::FrameworkOptions::cooldown_hit`]
    CooldownCallback,
    /// Error occured in [`crate::FrameworkOptions::missing_bot_permissions_handler`]
    MissingBotPermissionsCallback,
}
