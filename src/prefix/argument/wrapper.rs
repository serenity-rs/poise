use super::*;

macro_rules! impl_parse_consuming {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl<'a> PopArgumentAsync<'a> for $t {
            type Err = <$t as serenity::ArgumentConvert>::Err;

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

// Direct PopArgumentAsync implementation for all known types to avoid the Wrapper indirection for
// at least some types
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
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
pub struct Wrapper<T>(pub T);

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert> PopArgumentAsync<'a> for Wrapper<T> {
    type Err = T::Err;

    async fn async_pop_from(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err> {
        let (args, string) = String::pop_from(args).unwrap_or((args.clone(), String::new()));
        let token = T::convert(ctx, msg.guild_id, Some(msg.channel_id), &string).await?;
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
