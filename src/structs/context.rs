//! Just contains Context and `PartialContext` structs

use std::borrow::Cow;

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

/// Wrapper around either [`crate::ApplicationContext`] or [`crate::PrefixContext`]
#[derive(Debug)]
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
/// Macro to generate Context methods and also PrefixContext and ApplicationContext methods that
/// delegate to Context
macro_rules! context_methods {
    ( $(
        $( #[$($attrs:tt)*] )*
        // pub $(async $($dummy:block)?)? fn $fn_name:ident $()
        // $fn_name:ident ($($sig:tt)*) $body:block
        $($await:ident)? ( $fn_name:ident $self:ident $($arg:ident)* )
        ( $($sig:tt)* ) $body:block
    )* ) => {
        impl<'a, U, E> Context<'a, U, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* $body
        )* }

        impl<'a, U, E> crate::PrefixContext<'a, U, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* {
                $crate::Context::Prefix($self).$fn_name($($arg)*) $(.$await)?
            }
        )* }

        impl<'a, U, E> crate::ApplicationContext<'a, U, E> { $(
            $( #[$($attrs)*] )*
            $($sig)* {
                $crate::Context::Application($self).$fn_name($($arg)*) $(.$await)?
            }
        )* }
    };
}
// Note how you have to surround the function signature in parantheses, and also add a line before
// the signature with the function name, parameter names and maybe `await` token
context_methods! {
    /// Defer the response, giving the bot multiple minutes to respond without the user seeing an
    /// "interaction failed error".
    ///
    /// Also sets the [`crate::ApplicationContext::has_sent_initial_response`] flag so subsequent
    /// responses will be sent in the correct manner.
    ///
    /// No-op if this is an autocomplete context
    ///
    /// This will make the response public; to make it ephemeral, use [`Self::defer_ephemeral()`].
    await (defer self)
    (pub async fn defer(self) -> Result<(), serenity::Error>) {
        if let Self::Application(ctx) = self {
            ctx.defer_response(false).await?;
        }
        Ok(())
    }

    /// See [`Self::defer()`]
    ///
    /// This will make the response ephemeral; to make it public, use [`Self::defer()`].
    await (defer_ephemeral self)
    (pub async fn defer_ephemeral(self) -> Result<(), serenity::Error>) {
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
    await (defer_or_broadcast self)
    (pub async fn defer_or_broadcast(self) -> Result<Option<serenity::Typing>, serenity::Error>) {
        Ok(match self {
            Self::Application(ctx) => {
                ctx.defer_response(false).await?;
                None
            }
            Self::Prefix(ctx) => Some(
                ctx.msg
                    .channel_id
                    .start_typing(&ctx.serenity_context.http)?,
            ),
        })
    }

    /// Shorthand of [`crate::say_reply`]
    ///
    /// Note: panics when called in an autocomplete context!
    await (say self text)
    (pub async fn say(
        self,
        text: impl Into<String>,
    ) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        crate::say_reply(self, text).await
    }

    /// Like [`Self::say`], but formats the message as a reply to the user's command
    /// message.
    ///
    /// Equivalent to `.send(|b| b.content("...").reply(true))`.
    ///
    /// Only has an effect in prefix context, because slash command responses are always
    /// formatted as a reply.
    ///
    /// Note: panics when called in an autocomplete context!
    await (reply self text)
    (pub async fn reply(
        self,
        text: impl Into<String>,
    ) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        self.send(|b| b.content(text).reply(true)).await
    }

    /// Shorthand of [`crate::send_reply`]
    ///
    /// Note: panics when called in an autocomplete context!
    await (send self builder)
    (pub async fn send<'att>(
        self,
        builder: impl for<'b> FnOnce(
            &'b mut crate::CreateReply<'att>,
        ) -> &'b mut crate::CreateReply<'att>,
    ) -> Result<crate::ReplyHandle<'a>, serenity::Error>) {
        crate::send_reply(self, builder).await
    }

    /// Return the stored [`serenity::Context`] within the underlying context type.
    (serenity_context self)
    (pub fn serenity_context(self) -> &'a serenity::Context) {
        match self {
            Self::Application(ctx) => ctx.serenity_context,
            Self::Prefix(ctx) => ctx.serenity_context,
        }
    }

    /// See [`Self::serenity_context`].
    #[deprecated = "poise::Context can now be passed directly into most serenity functions. Otherwise, use `.serenity_context()` now"]
    #[allow(deprecated)]
    (discord self)
    (pub fn discord(self) -> &'a serenity::Context) {
        self.serenity_context()
    }

    /// Returns a view into data stored by the framework, like configuration
    (framework self)
    (pub fn framework(self) -> crate::FrameworkContext<'a, U, E>) {
        match self {
            Self::Application(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    /// Return a reference to your custom user data
    (data self)
    (pub fn data(self) -> &'a U) {
        match self {
            Self::Application(ctx) => ctx.data,
            Self::Prefix(ctx) => ctx.data,
        }
    }

    /// Return the channel ID of this context
    (channel_id self)
    (pub fn channel_id(self) -> serenity::ChannelId) {
        match self {
            Self::Application(ctx) => ctx.interaction.channel_id(),
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    /// Returns the guild ID of this context, if we are inside a guild
    (guild_id self)
    (pub fn guild_id(self) -> Option<serenity::GuildId>) {
        match self {
            Self::Application(ctx) => ctx.interaction.guild_id(),
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the guild of this context, if we are inside a guild.
    ///
    /// Warning: clones the entire Guild instance out of the cache
    #[cfg(feature = "cache")]
    (guild self)
    (pub fn guild(self) -> Option<serenity::Guild>) {
        self.guild_id()?.to_guild_cached(self)
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the partial guild of this context, if we are inside a guild.
    ///
    /// Attempts to find the guild in cache, if cache feature is enabled. Otherwise, falls back to
    /// an HTTP request
    ///
    /// Returns None if in DMs, or if the guild HTTP request fails
    await (partial_guild self)
    (pub async fn partial_guild(self) -> Option<serenity::PartialGuild>) {
        #[cfg(feature = "cache")]
        if let Some(guild) = self.guild_id()?.to_guild_cached(self) {
            return Some(guild.into());
        }

        self.guild_id()?.to_partial_guild(self).await.ok()
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Returns the author of the invoking message or interaction, as a [`serenity::Member`]
    ///
    /// Returns a reference to the inner member object if in an [`crate::ApplicationContext`], otherwise
    /// clones the member out of the cache, or fetches from the discord API.
    ///
    /// Returns None if this command was invoked in DMs, or if the member cache lookup or HTTP
    /// request failed
    ///
    /// Warning: can clone the entire Member instance out of the cache
    await (author_member self)
    (pub async fn author_member(self) -> Option<Cow<'a, serenity::Member>>) {
        if let Self::Application(ctx) = self {
            ctx.interaction.member().map(Cow::Borrowed)
        } else {
            self.guild_id()?
                .member(self.serenity_context(), self.author().id)
                .await
                .ok()
                .map(Cow::Owned)
        }
    }

    /// Return the datetime of the invoking message or interaction
    (created_at self)
    (pub fn created_at(self) -> serenity::Timestamp) {
        match self {
            Self::Application(ctx) => ctx.interaction.id().created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    /// Get the author of the command message or application command.
    (author self)
    (pub fn author(self) -> &'a serenity::User) {
        match self {
            Self::Application(ctx) => ctx.interaction.user(),
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }

    /// Return a ID that uniquely identifies this command invocation.
    #[cfg(any(feature = "chrono", feature = "time"))]
    (id self)
    (pub fn id(self) -> u64) {
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

                    #[cfg(feature = "time")]
                    let timestamp_millis = edited_timestamp.unix_timestamp_nanos() / 1_000_000;

                    #[cfg(not(feature = "time"))]
                    let timestamp_millis = edited_timestamp.timestamp_millis();

                    id |= ((timestamp_millis - 1420070400000) as u64) << 22;
                }
                id
            }
        }
    }

    /// If the invoked command was a subcommand, these are the parent commands, ordered top-level
    /// downwards.
    (parent_commands self)
    (pub fn parent_commands(self) -> &'a [&'a crate::Command<U, E>]) {
        match self {
            Self::Prefix(x) => x.parent_commands,
            Self::Application(x) => x.parent_commands,
        }
    }

    /// Returns a reference to the command.
    (command self)
    (pub fn command(self) -> &'a crate::Command<U, E>) {
        match self {
            Self::Prefix(x) => x.command,
            Self::Application(x) => x.command,
        }
    }

    /// Returns the prefix this command was invoked with, or a slash (`/`), if this is an
    /// application command.
    (prefix self)
    (pub fn prefix(self) -> &'a str) {
        match self {
            Context::Prefix(ctx) => ctx.prefix,
            Context::Application(_) => "/",
        }
    }

    /// Returns the command name that this command was invoked with
    ///
    /// Mainly useful in prefix context, for example to check whether a command alias was used.
    ///
    /// In slash contexts, the given command name will always be returned verbatim, since there are
    /// no slash command aliases and the user has no control over spelling
    (invoked_command_name self)
    (pub fn invoked_command_name(self) -> &'a str) {
        match self {
            Self::Prefix(ctx) => ctx.invoked_command_name,
            Self::Application(ctx) => &ctx.interaction.data().name,
        }
    }

    /// Re-runs this entire command invocation
    ///
    /// Permission checks are omitted; the command code is directly executed as a function. The
    /// result is returned by this function
    await (rerun self)
    (pub async fn rerun(self) -> Result<(), E>) {
        match self.rerun_inner().await {
            Ok(()) => Ok(()),
            Err(crate::FrameworkError::Command { error, ctx: _ }) => Err(error),
            // The only code that runs before the actual user code (which would trigger Command
            // error) is argument parsing. And that's pretty much deterministic. So, because the
            // current command invocation parsed successfully, we can always expect that a command
            // rerun will still parse successfully.
            // Also: can't debug print error because then we need U: Debug + E: Debug bound arghhhhh
            Err(_other) => panic!("unexpected error before entering command"),
        }
    }

    /// Returns the string with which this command was invoked.
    ///
    /// For example `"/slash_command subcommand arg1:value1 arg2:value2"`.
    (invocation_string self)
    (pub fn invocation_string(self) -> String) {
        match self {
            Context::Application(ctx) => {
                let mut string = String::from("/");
                for parent_command in ctx.parent_commands {
                    string += &parent_command.name;
                    string += " ";
                }
                string += &ctx.command.name;
                for arg in ctx.args {
                    if let Some(value) = &arg.value {
                        #[allow(unused_imports)] // required for simd-json
                        use ::serenity::json::prelude::*;
                        use std::fmt::Write as _;

                        string += " ";
                        string += &arg.name;
                        string += ":";
                        if let Some(x) = value.as_bool() {
                            let _ = write!(string, "{}", x);
                        } else if let Some(x) = value.as_i64() {
                            let _ = write!(string, "{}", x);
                        } else if let Some(x) = value.as_u64() {
                            let _ = write!(string, "{}", x);
                        } else if let Some(x) = value.as_f64() {
                            let _ = write!(string, "{}", x);
                        } else if let Some(x) = value.as_str() {
                            let _ = write!(string, "{}", x);
                        }
                    }
                }
                string
            }
            Context::Prefix(ctx) => ctx.msg.content.clone(),
        }
    }

    /// Stores the given value as the data for this command invocation
    ///
    /// This data is carried across the `pre_command` hook, checks, main command execution, and
    /// `post_command`. It may be useful to cache data or pass information to later phases of command
    /// execution.
    await (set_invocation_data self data)
    (pub async fn set_invocation_data<T: 'static + Send + Sync>(self, data: T)) {
        *self.invocation_data_raw().lock().await = Box::new(data);
    }

    /// Attempts to get the invocation data with the requested type
    ///
    /// If the stored invocation data has a different type than requested, None is returned
    await (invocation_data self)
    (pub async fn invocation_data<T: 'static>(
        self,
    ) -> Option<impl std::ops::DerefMut<Target = T> + 'a>) {
        tokio::sync::MutexGuard::try_map(self.invocation_data_raw().lock().await, |any| {
            any.downcast_mut()
        })
        .ok()
    }

    /// If available, returns the locale (selected language) of the invoking user
    (locale self)
    (pub fn locale(self) -> Option<&'a str>) {
        match self {
            Context::Application(ctx) => Some(ctx.interaction.locale()),
            Context::Prefix(_) => None,
        }
    }

    /// Builds a [`crate::CreateReply`] by combining the builder closure with the defaults that were
    /// pre-configured in poise.
    ///
    /// This is primarily an internal function and only exposed for people who want to manually
    /// convert [`crate::CreateReply`] instances into Discord requests.
    (reply_builder self builder)
    (pub fn reply_builder<'att>(
        self,
        builder: impl for<'b> FnOnce(&'b mut crate::CreateReply<'att>) -> &'b mut crate::CreateReply<'att>,
    ) -> crate::CreateReply<'att>) {
        let mut reply = crate::CreateReply {
            ephemeral: self.command().ephemeral,
            allowed_mentions: self.framework().options().allowed_mentions.clone(),
            ..Default::default()
        };
        builder(&mut reply);
        if let Some(callback) = self.framework().options().reply_callback {
            callback(self, &mut reply);
        }
        reply
    }

    /// Returns serenity's cache which stores various useful data received from the gateway
    ///
    /// Shorthand for [`.serenity_context().cache`](serenity::Context::cache)
    #[cfg(feature = "cache")]
    (cache self)
    (pub fn cache(self) -> &'a serenity::Cache) {
        &self.serenity_context().cache
    }

    /// Returns serenity's raw Discord API client to make raw API requests, if needed.
    ///
    /// Shorthand for [`.serenity_context().http`](serenity::Context::http)
    (http self)
    (pub fn http(self) -> &'a serenity::Http) {
        &self.serenity_context().http
    }
}

impl<'a, U, E> Context<'a, U, E> {
    /// Actual implementation of rerun() that returns `FrameworkError` for implementation convenience
    async fn rerun_inner(self) -> Result<(), crate::FrameworkError<'a, U, E>> {
        match self {
            Self::Application(ctx) => {
                // Skip autocomplete interactions
                let interaction = match ctx.interaction {
                    crate::ApplicationCommandOrAutocompleteInteraction::ApplicationCommand(
                        interaction,
                    ) => interaction,
                    crate::ApplicationCommandOrAutocompleteInteraction::Autocomplete(_) => {
                        return Ok(())
                    }
                };

                // Check slash command
                if interaction.data.kind == serenity::CommandType::ChatInput {
                    return if let Some(action) = ctx.command.slash_action {
                        action(ctx).await
                    } else {
                        Ok(())
                    };
                }

                // Check context menu command
                if let (Some(action), Some(target)) =
                    (ctx.command.context_menu_action, &interaction.data.target())
                {
                    return match action {
                        crate::ContextMenuCommandAction::User(action) => {
                            if let serenity::ResolvedTarget::User(user, _) = target {
                                action(ctx, user.clone()).await
                            } else {
                                Ok(())
                            }
                        }
                        crate::ContextMenuCommandAction::Message(action) => {
                            if let serenity::ResolvedTarget::Message(message) = target {
                                action(ctx, *message.clone()).await
                            } else {
                                Ok(())
                            }
                        }
                    };
                }
            }
            Self::Prefix(ctx) => {
                if let Some(action) = ctx.command.prefix_action {
                    return action(ctx).await;
                }
            }
        }

        // Fallback if the Command doesn't have the action it needs to execute this context
        // (This should never happen, because if this context cannot be executed, how could this
        // method have been called)
        Ok(())
    }

    /// Returns the raw type erased invocation data
    fn invocation_data_raw(self) -> &'a tokio::sync::Mutex<Box<dyn std::any::Any + Send + Sync>> {
        match self {
            Context::Application(ctx) => ctx.invocation_data,
            Context::Prefix(ctx) => ctx.invocation_data,
        }
    }
}

// Forwards for serenity::Context's impls. With these, poise::Context can be passed in as-is to
// serenity API functions.
#[cfg(feature = "cache")]
impl<U, E> AsRef<serenity::Cache> for Context<'_, U, E> {
    fn as_ref(&self) -> &serenity::Cache {
        &self.serenity_context().cache
    }
}
impl<U, E> AsRef<serenity::Http> for Context<'_, U, E> {
    fn as_ref(&self) -> &serenity::Http {
        &self.serenity_context().http
    }
}
impl<U, E> AsRef<serenity::ShardMessenger> for Context<'_, U, E> {
    fn as_ref(&self) -> &serenity::ShardMessenger {
        &self.serenity_context().shard
    }
}
// Originally added as part of component interaction modals; not sure if this impl is really
// required by anything else... It makes sense to have though imo
impl<U, E> AsRef<serenity::Context> for Context<'_, U, E> {
    fn as_ref(&self) -> &serenity::Context {
        self.serenity_context()
    }
}
impl<U: Sync, E> serenity::CacheHttp for Context<'_, U, E> {
    fn http(&self) -> &serenity::Http {
        &self.serenity_context().http
    }

    #[cfg(feature = "cache")]
    fn cache(&self) -> Option<&std::sync::Arc<serenity::Cache>> {
        Some(&self.serenity_context().cache)
    }
}

/// Trimmed down, more general version of [`Context`]
pub struct PartialContext<'a, U, E> {
    /// ID of the guild, if not invoked in DMs
    pub guild_id: Option<serenity::GuildId>,
    /// ID of the invocation channel
    pub channel_id: serenity::ChannelId,
    /// ID of the invocation author
    pub author: &'a serenity::User,
    /// Serenity's context, like HTTP or cache
    pub serenity_context: &'a serenity::Context,
    /// Useful if you need the list of commands, for example for a custom help command
    pub framework: crate::FrameworkContext<'a, U, E>,
    /// Your custom user data
    // TODO: redundant with framework
    pub data: &'a U,
}

impl<U, E> Copy for PartialContext<'_, U, E> {}
impl<U, E> Clone for PartialContext<'_, U, E> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<'a, U, E> From<Context<'a, U, E>> for PartialContext<'a, U, E> {
    fn from(ctx: Context<'a, U, E>) -> Self {
        Self {
            guild_id: ctx.guild_id(),
            channel_id: ctx.channel_id(),
            author: ctx.author(),
            serenity_context: ctx.serenity_context(),
            framework: ctx.framework(),
            data: ctx.data(),
        }
    }
}
