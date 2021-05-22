//! Holds prefix-command definition structs.

use crate::{serenity_prelude as serenity, BoxFuture, Framework};

/// Passed to command invocations.
///
/// Contains the trigger message, the Discord connection management stuff, and the user data.
pub struct PrefixContext<'a, U, E> {
    pub discord: &'a serenity::Context,
    pub msg: &'a serenity::Message,
    pub framework: &'a Framework<U, E>,
    pub data: &'a U,
}
// manual Copy+Clone implementations because Rust is getting confused about the type parameter
impl<U, E> Clone for PrefixContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<U, E> Copy for PrefixContext<'_, U, E> {}
impl<U, E> crate::_GetGenerics for PrefixContext<'_, U, E> {
    type U = U;
    type E = E;
}

pub struct PrefixCommandOptions<U, E> {
    /// Short description of the command. Displayed inline in help menus and similar.
    pub inline_help: Option<&'static str>,
    /// Multiline description with detailed usage instructions. Displayed in the command specific
    /// help: `~help command_name`
    // TODO: fix the inconsistency that this is String and everywhere else it's &'static str
    pub multiline_help: Option<fn() -> String>,
    /// Alternative triggers for the command
    pub aliases: &'static [&'static str],
    /// Fall back to the framework-specified value on None.
    pub on_error: Option<fn(E, PrefixCommandErrorContext<'_, U, E>) -> BoxFuture<'_, ()>>,
    /// If this function returns false, this command will not be executed.
    pub check: Option<fn(PrefixContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Whether to enable edit tracking for commands by default. Note that this won't do anything
    /// if `Framework::edit_tracker` isn't set.
    pub track_edits: bool,
    /// Fall back to the framework-specified value on None.
    pub broadcast_typing: Option<BroadcastTypingBehavior>,
    /// Whether to hide this command in help menus.
    pub hide_in_help: bool,
}

impl<U, E> Default for PrefixCommandOptions<U, E> {
    fn default() -> Self {
        Self {
            inline_help: None,
            multiline_help: None,
            check: None,
            on_error: None,
            aliases: &[],
            track_edits: false,
            broadcast_typing: None,
            hide_in_help: false,
        }
    }
}

pub struct PrefixCommand<U, E> {
    pub name: &'static str,
    pub action: for<'a> fn(PrefixContext<'a, U, E>, args: &'a str) -> BoxFuture<'a, Result<(), E>>,
    pub options: PrefixCommandOptions<U, E>,
}

pub struct PrefixCommandErrorContext<'a, U, E> {
    pub while_checking: bool,
    pub command: &'a PrefixCommand<U, E>,
    pub ctx: PrefixContext<'a, U, E>,
}

impl<U, E> Clone for PrefixCommandErrorContext<'_, U, E> {
    fn clone(&self) -> Self {
        Self {
            while_checking: self.while_checking,
            command: self.command,
            ctx: self.ctx,
        }
    }
}

pub struct PrefixFrameworkOptions<U, E> {
    /// List of bot commands.
    pub commands: Vec<PrefixCommand<U, E>>,
    /// List of additional bot prefixes
    pub additional_prefixes: &'static [&'static str],
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// Individual commands may override this callback.
    pub command_check: fn(PrefixContext<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>,
    /// If Some, the framework will react to message edits by editing the corresponding bot response
    /// with the new result.
    pub edit_tracker: Option<parking_lot::RwLock<super::EditTracker>>,
    /// Whether to broadcast a typing indicator while executing this commmand's action.
    pub broadcast_typing: BroadcastTypingBehavior,
    /// Whether commands in messages emitted by the bot itself should be executed as well.
    pub execute_self_messages: bool,
}

impl<U, E> Default for PrefixFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            additional_prefixes: &[],
            command_check: |_| Box::pin(async { Ok(true) }),
            edit_tracker: None,
            broadcast_typing: BroadcastTypingBehavior::None,
            execute_self_messages: false,
        }
    }
}

pub enum BroadcastTypingBehavior {
    /// Don't broadcast typing
    None,
    /// Broadcast typing after the command has been running for a certain time
    ///
    /// Set duration to zero for immediate typing broadcast
    WithDelay(std::time::Duration),
}
