//! Holds prefix-command definition structs.

use std::borrow::Cow;

use crate::{serenity_prelude as serenity, BoxFuture};

/// The event that triggered a prefix command execution
#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum MessageDispatchTrigger {
    /// The invocation message was posted directly (common case)
    MessageCreate,
    /// The message was edited, and was already a valid invocation pre-edit
    MessageEdit,
    /// The message was edited, and was not a valid invocation pre-edit (i.e. user typoed the
    /// command, then fixed it)
    MessageEditFromInvalid,
    #[doc(hidden)]
    __NonExhaustive,
}

/// Prefix-specific context passed to command invocations.
///
/// Contains the trigger message, the Discord connection management stuff, and the user data.
#[derive_where::derive_where(Debug)]
pub struct PrefixContext<'a, U, E> {
    /// The invoking user message
    pub msg: &'a serenity::Message,
    /// Position in the string that the prefix used by the user to invoke this command ends.
    pub content_start: u16,
    /// Command name used by the user to invoke this command
    pub invoked_command_name: &'a str,
    /// Entire argument string
    pub args: &'a str,
    /// Read-only reference to the framework
    ///
    /// Useful if you need the list of commands, for example for a custom help command
    #[derive_where(skip)]
    pub framework: crate::FrameworkContext<'a, U, E>,
    /// The command invoked by this message.
    ///
    /// This includes the full command tree, ordered top-down from parent commands to invoked
    /// command. For example, if `?x y z` is invoked, this contains `&[&x, &y, &z]`.
    pub command_tree: &'a [&'a crate::Command<U, E>],
    /// Custom user data carried across a single command invocation
    pub invocation_data: &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>>,
    /// How this command invocation was triggered
    pub trigger: MessageDispatchTrigger,

    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    pub __non_exhaustive: (),
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

/// Possible ways to define a command prefix
#[derive(Clone, Debug)]
pub enum Prefix {
    /// A case-sensitive string literal prefix (passed to [`str::strip_prefix`])
    Literal(&'static str),
    /// Regular expression which matches the prefix
    Regex(regex::Regex),
    #[doc(hidden)]
    __NonExhaustive,
}

/// Prefix-specific framework configuration
#[derive_where::derive_where(Debug)]
pub struct PrefixFrameworkOptions<U, E> {
    /// The main bot prefix. Can be set to None if the bot supports only
    /// [dynamic prefixes](Self::dynamic_prefix).
    pub prefix: Option<Cow<'static, str>>,
    /// List of additional bot prefixes
    // TODO: maybe it would be nicer to have separate fields for literal and regex prefixes
    // That way, you don't need to wrap every single literal prefix in a long path which looks ugly
    pub additional_prefixes: Vec<Prefix>,
    /// Callback invoked on every message to return a prefix.
    ///
    /// Override this field for a simple dynamic prefix which changes depending on the guild or user.
    ///
    /// For more advanced dynamic prefixes, see [`Self::stripped_dynamic_prefix`]
    #[derive_where(skip)]
    pub dynamic_prefix: Option<
        fn(crate::PartialContext<'_, U, E>) -> BoxFuture<'_, Result<Option<Cow<'static, str>>, E>>,
    >,
    /// Callback invoked on every message to strip the prefix off an incoming message.
    ///
    /// Override this field for advanced dynamic prefixes which change depending on guild or user.
    ///
    /// Return value is the prefix found:
    /// ```rust,no_run
    /// # poise::PrefixFrameworkOptions::<(), ()> { stripped_dynamic_prefix: Some(|_, msg, _| Box::pin(async move {
    /// let my_cool_prefix = "$";
    /// if msg.content.starts_with(my_cool_prefix) {
    ///     return Ok(Some(&msg.content[..my_cool_prefix.len()]));
    /// }
    /// Ok(None)
    /// # })), ..Default::default() };
    /// ```
    #[derive_where(skip)]
    pub stripped_dynamic_prefix: Option<
        for<'a> fn(
            &'a serenity::Context,
            &'a serenity::Message,
            &'a U,
        ) -> BoxFuture<'a, Result<Option<&'a str>, E>>,
    >,
    /// Treat a bot mention (a ping) like a prefix
    pub mention_as_prefix: bool,
    /// If Some, the framework will react to message edits by editing the corresponding bot response
    /// with the new result.
    pub edit_tracker: Option<std::sync::Arc<std::sync::RwLock<crate::EditTracker>>>,
    /// If the user makes a typo in their message and a subsequent edit creates a valid invocation,
    /// the bot will execute the command if this attribute is set.
    ///
    /// That does not mean that any subsequent edits will also trigger execution. For that,
    /// see [`crate::Command::invoke_on_edit`].
    ///
    /// Note: Only has an effect if [`Self::edit_tracker`] is set.
    pub execute_untracked_edits: bool,
    /// Whether to ignore message edits on messages that have not yet been responded to.
    ///
    /// This is the case if the message edit happens before a command has sent a response, or if the
    /// command does not send a response at all.
    pub ignore_edits_if_not_yet_responded: bool,
    /// Whether to ignore message edits when the message was present in the message cache and the
    /// content of the updated message is unchanged. Default is `true`.
    ///
    /// It is recommended to keep this on to prevent unintended command invocation.
    ///
    /// Note: Only has an effect if [`Self::edit_tracker`] is set and message caching is used.
    #[cfg(feature = "cache")]
    pub check_edits_against_cache: bool,
    /// Optional window of time during which edit tracking may be initiated, beginning at message
    /// creation. Default is 15 minutes.
    ///
    /// Setting a sensible duration is recommended to prevent unintended command invocations for
    /// old messages, which can be triggered by both Discord and user edits.
    ///
    /// Note: Only has an effect if [`Self::edit_tracker`] is set.
    pub tracking_initiation_window: Option<std::time::Duration>,

    /// Whether commands in messages emitted by this bot itself should be executed as well.
    pub execute_self_messages: bool,
    /// Whether to ignore messages from bots for command invoking. Default `true`
    pub ignore_bots: bool,
    /// Whether to ignore commands contained within thread creation messages. Default `true`
    pub ignore_thread_creation: bool,
    /// Whether command names should be compared case-insensitively.
    pub case_insensitive_commands: bool,
    /// Callback for all non-command messages. Useful if you want to run code on any message that
    /// is not a command
    pub non_command_message: Option<
        for<'a> fn(
            &'a crate::FrameworkContext<'a, U, E>,
            &'a serenity::Message,
        ) -> crate::BoxFuture<'a, Result<(), E>>,
    >,
    /* // TODO: implement
    /// Whether to invoke help command when someone sends a message with just a bot mention
    pub help_when_mentioned: bool,
    /// The bot's general help command. Currently used for [`Self::help_when_mentioned`].
    pub help_commmand: Option<Command<U, E>>,
    // /// The bot's help command for individial commands. Currently used when a command group without
    // /// any specific subcommand is invoked. This command is expected to take the command name as a
    // /// single parameter
    // pub command_specific_help_commmand: Option<Command<U, E>>, */
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<U, E> Default for PrefixFrameworkOptions<U, E> {
    fn default() -> Self {
        Self {
            prefix: None,
            additional_prefixes: Vec::new(),
            dynamic_prefix: None,
            stripped_dynamic_prefix: None,
            mention_as_prefix: true,
            edit_tracker: None,
            execute_untracked_edits: true,
            ignore_edits_if_not_yet_responded: false,
            #[cfg(feature = "cache")]
            check_edits_against_cache: true,
            tracking_initiation_window: Some(std::time::Duration::from_secs(60 * 15)),
            execute_self_messages: false,
            ignore_bots: true,
            ignore_thread_creation: true,
            case_insensitive_commands: true,
            non_command_message: None,
            // help_when_mentioned: true,
            // help_commmand: None,
            // command_specific_help_commmand: None,
            __non_exhaustive: (),
        }
    }
}
