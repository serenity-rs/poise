//! Just contains `FrameworkOptions`

use crate::{serenity_prelude as serenity, BoxFuture};

/// Framework configuration
#[derive(derivative::Derivative)]
#[derivative(Debug(bound = ""))]
pub struct FrameworkOptions<U, E> {
    /// List of commands in the framework
    pub commands: Vec<crate::Command<U, E>>,
    /// Provide a callback to be invoked when any user code yields an error.
    #[derivative(Debug = "ignore")]
    pub on_error: fn(crate::FrameworkError<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called before every command
    #[derivative(Debug = "ignore")]
    pub pre_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Called after every command if it was successful (returned Ok)
    #[derivative(Debug = "ignore")]
    pub post_command: fn(crate::Context<'_, U, E>) -> BoxFuture<'_, ()>,
    /// Provide a callback to be invoked before every command. The command will only be executed
    /// if the callback returns true.
    ///
    /// If individual commands add their own check, both callbacks are run and must return true.
    #[derivative(Debug = "ignore")]
    pub command_check: Option<fn(crate::Context<'_, U, E>) -> BoxFuture<'_, Result<bool, E>>>,
    /// Default set of allowed mentions to use for all responses
    ///
    /// By default, user pings are allowed and role pings and everyone pings are filtered
    pub allowed_mentions: Option<serenity::CreateAllowedMentions>,
    /// Invoked before every message sent using [`crate::Context::say`] or [`crate::Context::send`]
    ///
    /// Allows you to modify every outgoing message in a central place
    #[derivative(Debug = "ignore")]
    pub reply_callback:
        Option<for<'a> fn(crate::Context<'_, U, E>, crate::CreateReply) -> crate::CreateReply>,
    /// If `true`, disables automatic cooldown handling before every command invocation.
    ///
    /// Useful for implementing custom cooldown behavior. See [`crate::Command::cooldowns`] and
    /// the methods on [`crate::Cooldowns`] for how to do that.
    pub manual_cooldowns: bool,
    /// If `true`, changes behavior of guild_only command check to abort execution if the guild is
    /// not in cache.
    ///
    /// **If `cache` feature is disabled, this has no effect!**
    pub require_cache_for_guild_check: bool,
    /// Called on every Discord event. Can be used to react to non-command events, like messages
    /// deletions or guild updates.
    #[derivative(Debug = "ignore")]
    pub listener: for<'a> fn(
        &'a serenity::Context,
        &'a crate::Event,
        crate::FrameworkContext<'a, U, E>,
        // TODO: redundant with framework
        &'a U,
    ) -> BoxFuture<'a, Result<(), E>>,
    /// Prefix command specific options.
    pub prefix_options: crate::PrefixFrameworkOptions<U, E>,
    /// User IDs which are allowed to use owners_only commands
    ///
    /// If using [`crate::FrameworkBuilder`], automatically initialized with the bot application
    /// owner and team members
    pub owners: std::collections::HashSet<serenity::UserId>,
    // #[non_exhaustive] forbids struct update syntax for ?? reason
    #[doc(hidden)]
    pub __non_exhaustive: (),
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
                        log::error!("Error while handling error: {}", e);
                    }
                })
            },
            listener: |_, _, _, _| Box::pin(async { Ok(()) }),
            pre_command: |_| Box::pin(async {}),
            post_command: |_| Box::pin(async {}),
            command_check: None,
            allowed_mentions: Some(
                serenity::CreateAllowedMentions::default()
                    // Only support direct user pings by default
                    .all_users(true),
            ),
            reply_callback: None,
            manual_cooldowns: false,
            require_cache_for_guild_check: false,
            prefix_options: Default::default(),
            owners: Default::default(),
            __non_exhaustive: (),
        }
    }
}
