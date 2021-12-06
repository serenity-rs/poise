use crate::serenity_prelude as serenity;

/// Wrapper around either [`crate::ApplicationContext`] or [`crate::PrefixContext`]
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
impl<'a, U, E> Context<'a, U, E> {
    /// Defer the response, giving the bot multiple minutes to respond without the user seeing an
    /// "interaction failed error".
    ///
    /// Also sets the [`crate::ApplicationContext::has_sent_initial_response`] flag so subsequent
    /// responses will be sent in the correct manner.
    ///
    /// No-op if this is an autocomplete context
    ///
    /// This will make the response public; to make it ephemeral, use [`Self::defer_ephemeral()`].
    pub async fn defer(self) -> Result<(), serenity::Error> {
        if let Self::Application(ctx) = self {
            ctx.defer_response(false).await?;
        }
        Ok(())
    }

    /// See [`Self::defer()`]
    ///
    /// This will make the response ephemeral; to make it public, use [`Self::defer()`].
    pub async fn defer_ephemeral(self) -> Result<(), serenity::Error> {
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
    pub async fn defer_or_broadcast(self) -> Result<Option<serenity::Typing>, serenity::Error> {
        Ok(match self {
            Self::Application(ctx) => {
                ctx.defer_response(false).await?;
                None
            }
            Self::Prefix(ctx) => Some(ctx.msg.channel_id.start_typing(&ctx.discord.http)?),
        })
    }

    /// Shorthand of [`crate::say_reply`]
    pub async fn say(
        self,
        text: impl Into<String>,
    ) -> Result<Option<crate::ReplyHandle<'a>>, serenity::Error> {
        crate::say_reply(self, text).await
    }

    /// Shorthand of [`crate::send_reply`]
    pub async fn send<'b>(
        self,
        builder: impl for<'c> FnOnce(&'c mut crate::CreateReply<'b>) -> &'c mut crate::CreateReply<'b>,
    ) -> Result<Option<crate::ReplyHandle<'a>>, serenity::Error> {
        crate::send_reply(self, builder).await
    }
}

impl<'a, U, E> Context<'a, U, E> {
    /// Return the stored [`serenity::Context`] within the underlying context type.
    pub fn discord(&self) -> &'a serenity::Context {
        match self {
            Self::Application(ctx) => ctx.discord,
            Self::Prefix(ctx) => ctx.discord,
        }
    }

    /// Return a read-only reference to [`crate::Framework`].
    pub fn framework(&self) -> &'a crate::Framework<U, E> {
        match self {
            Self::Application(ctx) => ctx.framework,
            Self::Prefix(ctx) => ctx.framework,
        }
    }

    /// Return a reference to your custom user data
    pub fn data(&self) -> &'a U {
        match self {
            Self::Application(ctx) => ctx.data,
            Self::Prefix(ctx) => ctx.data,
        }
    }

    /// Return the channel ID of this context
    pub fn channel_id(&self) -> serenity::ChannelId {
        match self {
            Self::Application(ctx) => ctx.interaction.channel_id(),
            Self::Prefix(ctx) => ctx.msg.channel_id,
        }
    }

    /// Returns the guild ID of this context, if we are inside a guild
    pub fn guild_id(&self) -> Option<serenity::GuildId> {
        match self {
            Self::Application(ctx) => ctx.interaction.guild_id(),
            Self::Prefix(ctx) => ctx.msg.guild_id,
        }
    }

    // Doesn't fit in with the rest of the functions here but it's convenient
    /// Return the guild of this context, if we are inside a guild.
    ///
    /// Warning: clones the entire Guild instance out of the cache
    pub fn guild(&self) -> Option<serenity::Guild> {
        self.guild_id()?.to_guild_cached(self.discord())
    }

    /// Return the datetime of the invoking message or interaction
    pub fn created_at(&self) -> chrono::DateTime<chrono::Utc> {
        match self {
            Self::Application(ctx) => ctx.interaction.id().created_at(),
            Self::Prefix(ctx) => ctx.msg.timestamp,
        }
    }

    /// Get the author of the command message or application command.
    pub fn author(&self) -> &'a serenity::User {
        match self {
            Self::Application(ctx) => ctx.interaction.user(),
            Self::Prefix(ctx) => &ctx.msg.author,
        }
    }

    /// Return a ID that uniquely identifies this command invocation.
    pub fn id(&self) -> u64 {
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
                    id |= ((edited_timestamp.timestamp_millis() - 1420070400000) as u64) << 22;
                }
                id
            }
        }
    }

    /// Returns a reference to the command.
    pub fn command(&self) -> Option<crate::CommandRef<'a, U, E>> {
        Some(match self {
            Self::Prefix(x) => crate::CommandRef::Prefix(x.command?),
            Self::Application(x) => crate::CommandRef::Application(x.command),
        })
    }

    /// Returns the prefix this command was invoked with, or a slash (`/`), if this is an
    /// application command.
    pub fn prefix(&self) -> &'a str {
        match self {
            Context::Prefix(ctx) => ctx.prefix,
            Context::Application(_) => "/",
        }
    }
}

pub struct PartialContext<'a, U, E> {
    /// ID of the guild, if not invoked in DMs
    pub guild_id: Option<serenity::GuildId>,
    /// ID of the invocation channel
    pub channel_id: serenity::ChannelId,
    /// ID of the invocation author
    pub author: &'a serenity::User,
    /// Serenity's context, like HTTP or cache
    pub discord: &'a serenity::Context,
    /// Useful if you need the list of commands, for example for a custom help command
    pub framework: &'a crate::Framework<U, E>,
    /// Your custom user data
    pub data: &'a U,
}

impl<'a, U, E> From<Context<'a, U, E>> for PartialContext<'a, U, E> {
    fn from(ctx: Context<'a, U, E>) -> Self {
        Self {
            guild_id: ctx.guild_id(),
            channel_id: ctx.channel_id(),
            author: ctx.author(),
            discord: ctx.discord(),
            framework: ctx.framework(),
            data: ctx.data(),
        }
    }
}
