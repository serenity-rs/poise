//! Contains implementations of [`ArgumentConvert`] for [`serenity::Guild`] and
//! [`serenity::GuildId`].

use std::fmt;
use std::str::FromStr;

use super::ArgumentConvert;
use crate::serenity_prelude as serenity;

/// Error that can be returned from [`serenity::Guild::convert`].
#[derive(Debug)]
pub enum GuildParseError {
    /// The parsed guild id could not be found in the cache.
    NotFound,
    /// The provided guild Id failed to parse.
    Malformed(serenity::ParseIdError),
    /// No cache, so no guild search could be done.
    NoCache,
}

impl std::error::Error for GuildParseError {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Malformed(e) => Some(e),
            Self::NotFound | Self::NoCache => None,
        }
    }
}

impl fmt::Display for GuildParseError {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::NotFound => f.write_str("Guild not found"),
            Self::Malformed(_) => f.write_str("Unknown format for guild id"),
            Self::NoCache => f.write_str("No cached list of guilds was provided"),
        }
    }
}

/// Look up a Guild by Id.
///
/// Requires the cache feature to be enabled.
#[cfg(feature = "cache")]
#[async_trait::async_trait]
impl ArgumentConvert for serenity::Guild {
    type Err = GuildParseError;

    async fn convert(
        ctx: impl serenity::CacheHttp,
        _: Option<serenity::GuildId>,
        _: Option<serenity::GenericChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        let cache = ctx.cache().ok_or(GuildParseError::NoCache)?;
        let guild_id = s.parse().map_err(GuildParseError::Malformed)?;
        cache
            .guild(guild_id)
            .map(|g| g.clone())
            .ok_or(GuildParseError::NotFound)
    }
}

#[async_trait::async_trait]
impl ArgumentConvert for serenity::GuildId {
    type Err = <serenity::EmojiId as FromStr>::Err;

    async fn convert(
        _: impl serenity::CacheHttp,
        _: Option<serenity::GuildId>,
        _: Option<serenity::GenericChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        s.parse()
    }
}
