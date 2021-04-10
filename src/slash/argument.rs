//! Parse received slash command arguments into Rust types.

#[derive(Debug)]
pub enum SlashArgError {
    MissingRequired,
    UnexpectedSubcommand,
    ExpectedString,
    Parse(Box<dyn std::error::Error + Send + Sync>),
}
impl std::fmt::Display for SlashArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::MissingRequired => write!(f, "A required argument was missing"),
            Self::UnexpectedSubcommand => {
                write!(f, "Expected value, found subcommand/-group")
            }
            Self::ExpectedString => {
                write!(f, "Expected string argument, found other data type")
            }
            Self::Parse(e) => write!(f, "Failed to parse argument: {}", e),
        }
    }
}
impl std::error::Error for SlashArgError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Parse(e) => Some(&**e),
            Self::MissingRequired | Self::UnexpectedSubcommand | Self::ExpectedString => None,
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parse_slash {
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: Option<$type:ty>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == stringify!($name)) {
            let value = arg
                .value
                .as_ref()
                .ok_or($crate::SlashArgError::UnexpectedSubcommand)?
                .as_str()
                .ok_or($crate::SlashArgError::ExpectedString)?
                .parse::<$type>()
                .map_err(|e| $crate::SlashArgError::Parse(e.into()))?;
            Some(value)
        } else {
            None
        }
    };

    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: $type:ty) => {
        $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<$type>)
            .ok_or($crate::SlashArgError::MissingRequired)?
    };
}

#[macro_export]
macro_rules! parse_slash_args {
    ($ctx:expr, $guild_id:expr, $channel_id:expr, $args:expr => $(
        ( $name:ident: $($type:tt)* )
    ),* $(,)? ) => {
        (|| async /* not move! */ {
            let (ctx, guild_id, channel_id, args) = ($ctx, $guild_id, $channel_id, $args);

            Ok::<_, $crate::SlashArgError>(( $(
                $crate::_parse_slash!( ctx, guild_id, channel_id, args => $name: $($type)* )
            ),* ))
        })()
    };
}

#[test]
fn test_parse_slash_args() {}
