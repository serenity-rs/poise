use super::*;

// TODO: wrap serenity::utils::Parse instead of FromStr
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Wrapper<T>(pub T);

#[derive(Debug)]
pub enum WrapperError<E> {
    Missing,
    Parse(E),
}
impl<E: std::fmt::Display> std::fmt::Display for WrapperError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("Not enough arguments were given"),
            Self::Parse(e) => write!(f, "Failed to parse argument: {}", e),
        }
    }
}
impl<E: std::error::Error> std::error::Error for WrapperError<E> {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Parse(e) => Some(e),
            Self::Missing => None,
        }
    }
}

impl<'a, T: std::str::FromStr> ParseConsuming<'a> for Wrapper<T> {
    type Err = WrapperError<T::Err>;

    fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err> {
        let (args, token) = String::pop_from(args).map_err(|EmptyArgs| WrapperError::Missing)?;
        let token = token.parse().map_err(WrapperError::Parse)?;
        Ok((args, Self(token)))
    }
}
