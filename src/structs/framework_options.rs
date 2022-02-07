//! Just contains FrameworkOptions

use crate::{serenity_prelude as serenity, BoxFuture};

/// Framework configuration
pub struct FrameworkOptions<U, E> {
    /// List of commands in the framework
    pub commands: Vec<crate::Command<U, E>>,
    /// Provide a callback to be invoked when any user code yields an error.
    pub on_error: fn(crate::FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    pub pre_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called after every command, no matter if it succeeded or failed
    pub post_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// If individual commands add their own check, both callbacks are run and must return true.
    pub command_check: Option<fn(crate::Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Default set of allowed mentions to use for all responses
    ///
    /// By default, user pings are allowed and role pings and everyone pings are filtered
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// Invoked before every message sent using [`crate::Context::say`] or [`crate::Context::send`]
    ///
    /// Allows you to modify every outgoing message in a central place
    pub reply_callback: Option<fn(crate::Context<'_, U, E>, &mut crate::CreateReply<'_>)>,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a crate::Event<'a>,
        &'a crate::Framework<U, E>,
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Prefix command specific options.
    pub prefix_options: crate::PrefixFrameworkOptions<U, E>,
    /// User IDs which are allowed to use owners_only commands
    ///
    /// If using [`crate::FrameworkBuilder`], automatically initialized with the bot application
    /// owner and team members
    pub owners: std::collections::HashSet<serenity::UserId>,
}

impl<U, E> FrameworkOptions<U, E> {
    /// Add a new command to the framework
    #[deprecated = "supply commands in FrameworkOptions directly with `commands: vec![...]`"]
    pub fn command(
        &mut self,
        mut command: crate::Command<U, E>,
        meta_builder: impl FnOnce(&mut crate::Command<U, E>) -> &mut crate::Command<U, E> + 'static,
    ) {
        meta_builder(&mut command);
        self.commands.push(command);
    }
}

impl<U: std::fmt::Debug, E: std::fmt::Debug> std::fmt::Debug for FrameworkOptions<U, E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            commands: _,
            on_error,
            pre_command,
            post_command,
            command_check,
            allowed_mentions,
            reply_callback,
            listener,
            prefix_options,
            owners,
        } = self;

        f.debug_struct("FrameworkOptions")
            .field("commands", &"< Vec<poise Command> >")
            .field("on_error", &(*on_error as *const ()))
            .field("pre_command", &(*pre_command as *const ()))
            .field("post_command", &(*post_command as *const ()))
            .field("command_check", &command_check.map(|f| f as *const ()))
            .field("allowed_mentions", allowed_mentions)
            .field("command_check", &reply_callback.map(|f| f as *const ()))
            .field("listener", &(*listener as *const ()))
            .field("prefix_options", prefix_options)
            .field("owners", owners)
            .finish()
    }
}

impl<U, E> Default for FrameworkOptions<U, E>
where
    U: Send + Sync,
    E: std::fmt::Display + std::fmt::Debug + Send,
{
    fn default() -> Self {
        Self {
            commands: Vec::new(),
            on_error: |error| {
                Box::pin(async move {
                    if let Err(e) = crate::builtins::on_error(error).await {
                        println!("Error while handling error: {}", e);
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            post_command: |_| Box::pin(async {}),
            command_check: None,
            allowed_mentions: Some({
                let mut f = serenity::CreateAllowedMentions::default();
                // Only support direct user pings by default
                f.empty_parse().parse(serenity::ParseValue::Users);
                f
            }),
            reply_callback: None,
            prefix_options: Default::default(),
            owners: Default::default(),
        }
    }
}
