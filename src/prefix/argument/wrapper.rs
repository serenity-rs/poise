use super::*;

macro_rules! impl_parse_consuming {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl<'a> PopArgumentAsync<'a> for $t {
            type Err = WrapperArgumentParseError<<$t as serenity::ArgumentConvert>::Err>;

            async fn async_pop_from(
                ctx: &serenity::Context,
                msg: &serenity::Message,
                args: &ArgString<'a>
            ) -> Result<(ArgString<'a>, Self), Self::Err> {
                let (args, value) = Wrapper::async_pop_from(ctx, msg, args).await?;
                Ok((args, value.0))
            }
        }
    )* }
}

// Direct PopArgumentAsync implementation for all known ArgumentConvert/FromStr types to avoid the
// Wrapper indirection for at least some types
impl_parse_consuming!(
    bool char f32 f64 i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize
    std::net::IpAddr std::net::Ipv4Addr std::net::Ipv6Addr
    std::net::SocketAddr std::net::SocketAddrV4 std::net::SocketAddrV6
    std::num::NonZeroI8 std::num::NonZeroI16 std::num::NonZeroI32
    std::num::NonZeroI64 std::num::NonZeroI128 std::num::NonZeroIsize
    std::num::NonZeroU8 std::num::NonZeroU16 std::num::NonZeroU32
    std::num::NonZeroU64 std::num::NonZeroU128 std::num::NonZeroUsize
    std::path::PathBuf
    serenity::Channel serenity::GuildChannel serenity::ChannelCategory serenity::Emoji
    serenity::Member serenity::Message serenity::Role serenity::User
    serenity::ChannelId serenity::UserId serenity::RoleId
    serenity::EmojiIdentifier serenity::ReactionType
);

/// Command parameter type wrapper to support arbitrary [`serenity::ArgumentConvert`]/[`std::str::FromStr`]
/// instances.
///
/// This workaround is currently needed due to overlap rules disallowing overlapping blanket
/// implementations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
pub struct Wrapper<T>(pub T);

/// Error returned from parsing [`Wrapper`].
#[derive(Debug)]
pub enum WrapperArgumentParseError<E> {
    /// If the input was empty and [`Wrapper`] was unable to pass any string to the underlying type
    EmptyArgs(crate::EmptyArgs),
    /// The underlying type threw a parse error
    ParseError(E),
}

impl<E: std::fmt::Display> std::fmt::Display for WrapperArgumentParseError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            WrapperArgumentParseError::EmptyArgs(e) => e.fmt(f),
            WrapperArgumentParseError::ParseError(e) => e.fmt(f),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for WrapperArgumentParseError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            WrapperArgumentParseError::EmptyArgs(e) => Some(e),
            WrapperArgumentParseError::ParseError(e) => Some(e),
        }
    }
}

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert> PopArgumentAsync<'a> for Wrapper<T> {
    type Err = WrapperArgumentParseError<T::Err>;

    async fn async_pop_from(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err> {
        let (args, string) =
            String::pop_from(args).map_err(WrapperArgumentParseError::EmptyArgs)?;
        let token = T::convert(ctx, msg.guild_id, Some(msg.channel_id), &string)
            .await
            .map_err(WrapperArgumentParseError::ParseError)?;
        Ok((args, Self(token)))
    }
}

#[async_trait::async_trait]
impl<T: serenity::ArgumentConvert> serenity::ArgumentConvert for Wrapper<T> {
    type Err = T::Err;

    async fn convert(
        ctx: &serenity::Context,
        guild_id: Option<serenity::GuildId>,
        channel_id: Option<serenity::ChannelId>,
        s: &str,
    ) -> Result<Self, Self::Err> {
        T::convert(ctx, guild_id, channel_id, s).await.map(Self)
    }
}

// TODO: this really shouldn't be here but it works for now (until Wrapper is hopefully gone)
#[async_trait::async_trait]
impl<T: crate::SlashArgument> crate::SlashArgument for Wrapper<T> {
    fn create(
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        T::create(builder)
    }

    async fn extract(
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serde_json::Value,
    ) -> Result<Self, crate::SlashArgError> {
        T::extract(ctx, guild, channel, value).await.map(Self)
    }
}
