use super::*;

macro_rules! impl_parse_consuming {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl<'a> ParseConsuming<'a> for $t {
            type Err = <$t as serenity::Parse>::Err;

            async fn pop_from(
                ctx: &serenity::Context,
                msg: &serenity::Message,
                args: &ArgString<'a>
            ) -> Result<(ArgString<'a>, Self), Self::Err> {
                let (args, value) = Wrapper::pop_from(ctx, msg, args).await?;
                Ok((args, value.0))
            }
        }
    )* }
}

// Direct ParseConsuming implementation for all known types to avoid the Wrapper indirection for
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
    serenity::Message serenity::Member
);

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Hash, Ord, PartialOrd)]
pub struct Wrapper<T>(pub T);

#[async_trait::async_trait]
impl<'a, T: serenity::Parse> ParseConsuming<'a> for Wrapper<T> {
    type Err = T::Err;

    async fn pop_from(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err> {
        let (args, string) = String::sync_pop_from(args).unwrap_or((args.clone(), String::new()));
        let token = T::parse(ctx, msg, &string).await?;
        Ok((args, Self(token)))
    }
}

#[async_trait::async_trait]
impl<T: serenity::Parse> serenity::Parse for Wrapper<T> {
    type Err = T::Err;

    async fn parse(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        s: &str,
    ) -> Result<Self, Self::Err> {
        T::parse(ctx, msg, s).await.map(Self)
    }
}
