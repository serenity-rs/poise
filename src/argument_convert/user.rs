//! Contains the implementation of [`ArgumentConvert`] for [`serenity::User`].

use std::fmt;

use super::ArgumentConvert;
use crate::serenity_prelude as serenity;

/// Error that can be returned from [`serenity::User::convert`].
#[derive(Debug)]
pub enum UserParseError {
    /// The provided user string failed to parse, or the parsed result cannot be found in the guild
    /// cache data.
    NotFoundOrMalformed,
}

impl std::error::Error for UserParseError {}

impl fmt::Display for UserParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFoundOrMalformed => f.write_str("User not found or unknown format"),
        }
    }
}

#[async_trait::async_trait]
impl ArgumentConvert for serenity::User {
    type Err = UserParseError;

    async fn convert(
        ctx: impl serenity::CacheHttp,
        guild_id: Option<serenity::GuildId>,
        channel_id: Option<serenity::GenericChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        // Convert as a Member which uses HTTP endpoints instead of cache
        if let Ok(member) = serenity::Member::convert(&ctx, guild_id, channel_id, s).await {
            return Ok(member.user);
        }

        // If string is a raw user ID or a mention
        if let Some(user_id) = s
            .parse()
            .ok()
            .or_else(|| serenity::utils::parse_user_mention(s))
        {
            // Now, we can still try UserId::to_user because it works for all users from all guilds
            // the bot is joined
            if let Ok(user) = user_id.to_user(&ctx).await {
                return Ok(user);
            }
        }

        Err(UserParseError::NotFoundOrMalformed)
    }
}
