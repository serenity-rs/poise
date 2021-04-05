use super::*;

macro_rules! impl_for_from_str {
    ($($t:ty)*) => { $(
        impl<'a> ParseConsuming<'a> for $t {
            type Err = <$t as std::str::FromStr>::Err;

            fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err> {
                let (args, value) = Wrapper::pop_from(args)?;
                Ok((args, value.0))
            }
        }
    )* }
}

// Implement ParseConsuming for at least all std FromStr implementors, so the user doesn't have to
// use the clunky wrapper for simple types
impl_for_from_str!(
    bool char f32 f64 i8 i16 i32 i64 i128 isize u8 u16 u32 u64 u128 usize
    std::net::IpAddr std::net::Ipv4Addr std::net::Ipv6Addr
    std::net::SocketAddr std::net::SocketAddrV4 std::net::SocketAddrV6
    std::num::NonZeroI8 std::num::NonZeroI16 std::num::NonZeroI32
    std::num::NonZeroI64 std::num::NonZeroI128 std::num::NonZeroIsize
    std::num::NonZeroU8 std::num::NonZeroU16 std::num::NonZeroU32
    std::num::NonZeroU64 std::num::NonZeroU128 std::num::NonZeroUsize
    std::path::PathBuf
);

// TODO: wrap serenity::utils::Parse instead of FromStr
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapper<T>(pub T);

impl<'a, T: std::str::FromStr> ParseConsuming<'a> for Wrapper<T> {
    type Err = T::Err;

    fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err> {
        let (args, string) = String::pop_from(args).unwrap_or((args.clone(), String::new()));
        let token = string.parse()?;
        Ok((args, Self(token)))
    }
}
