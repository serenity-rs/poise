//! Parse received slash command arguments into Rust types.

use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

#[allow(unused_imports)] // import is required if serenity simdjson feature is enabled
use crate::serenity::json::prelude::*;
use crate::serenity_prelude as serenity;

/// Possible errors when parsing slash command arguments
#[derive(Debug)]
pub enum SlashArgError {
    /// Expected a certain argument type at a certain position in the unstructured list of
    /// arguments, but found something else.
    ///
    /// Most often the result of the bot not having registered the command in Discord, so Discord
    /// stores an outdated version of the command and its parameters.
    CommandStructureMismatch(&'static str),
    /// A string parameter was found, but it could not be parsed into the target type.
    Parse {
        /// Error that occured while parsing the string into the target type
        error: Box<dyn std::error::Error + Send + Sync>,
        /// Original input string
        input: String,
    },
}
impl std::fmt::Display for SlashArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandStructureMismatch(detail) => {
                write!(
                    f,
                    "Bot author did not register their commands correctly ({})",
                    detail
                )
            }
            Self::Parse { error, input } => {
                write!(f, "Failed to parse `{}` as argument: {}", input, error)
            }
        }
    }
}
impl std::error::Error for SlashArgError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Parse { error, input: _ } => Some(&**error),
            Self::CommandStructureMismatch(_) => None,
        }
    }
}

/// Implement this trait on types that you want to use as a slash command parameter.
#[async_trait::async_trait]
pub trait SlashArgument: Sized {
    /// Extract a Rust value of type T from the slash command argument, given via a
    /// [`serenity::json::Value`].
    async fn extract(
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<Self, SlashArgError>;

    /// Create a slash command parameter equivalent to type T.
    ///
    /// Only fields about the argument type are filled in. The caller is still responsible for
    /// filling in `name()`, `description()`, and possibly `required()` or other fields.
    fn create(
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption;
}

/// Implemented for all types that can be used as a function parameter in a slash command.
///
/// Currently marked `#[doc(hidden)]` because implementing this trait requires some jank due to a
/// `PhantomData` hack and the auto-deref specialization hack.
#[doc(hidden)]
#[async_trait::async_trait]
pub trait SlashArgumentHack<T> {
    async fn extract(
        self,
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<T, SlashArgError>;

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption;
}

/// Handles arbitrary types that can be parsed from string.
#[async_trait::async_trait]
impl<T> SlashArgumentHack<T> for PhantomData<T>
where
    T: serenity::ArgumentConvert + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn extract(
        self,
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<T, SlashArgError> {
        let string = value
            .as_str()
            .ok_or(SlashArgError::CommandStructureMismatch("expected string"))?;
        T::convert(ctx, guild, channel, string)
            .await
            .map_err(|e| SlashArgError::Parse {
                error: e.into(),
                input: string.into(),
            })
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::String)
    }
}

/// Error thrown if an integer slash command argument is too large
///
/// For example: a user inputs `300` as an argument of type [`u8`]
#[derive(Debug)]
pub struct IntegerOutOfBounds;
impl std::fmt::Display for IntegerOutOfBounds {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("integer out of bounds for target type")
    }
}
impl std::error::Error for IntegerOutOfBounds {}

// Handles all integers, signed and unsigned. The TryFrom<i64> impl is not actually used, it's just
// to filter for integer types.
#[async_trait::async_trait]
impl<T: TryFrom<i64> + Send + Sync> SlashArgumentHack<T> for &PhantomData<T> {
    async fn extract(
        self,
        _: &serenity::Context,
        _: Option<serenity::GuildId>,
        _: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<T, SlashArgError> {
        let number = value
            .as_i64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected integer"))?;
        number.try_into().map_err(|_| SlashArgError::Parse {
            error: IntegerOutOfBounds.into(),
            input: number.to_string(),
        })
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::Integer)
    }
}

#[async_trait::async_trait]
impl SlashArgumentHack<f32> for &&PhantomData<f32> {
    async fn extract(
        self,
        _: &serenity::Context,
        _: Option<serenity::GuildId>,
        _: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<f32, SlashArgError> {
        Ok(value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))? as f32)
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::Number)
    }
}

#[async_trait::async_trait]
impl SlashArgumentHack<f64> for &&PhantomData<f64> {
    async fn extract(
        self,
        _: &serenity::Context,
        _: Option<serenity::GuildId>,
        _: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<f64, SlashArgError> {
        value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::Number)
    }
}

#[async_trait::async_trait]
impl<T: SlashArgument + Sync> SlashArgumentHack<T> for &&PhantomData<T> {
    async fn extract(
        self,
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serenity::json::Value,
    ) -> Result<T, SlashArgError> {
        <T as SlashArgument>::extract(ctx, guild, channel, value).await
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        <T as SlashArgument>::create(builder)
    }
}

/// Implements SlashArgumentHack for a model type that is represented in interactions via an ID
macro_rules! impl_slash_argument {
    ($type:ty, $slash_param_type:ident) => {
        #[async_trait::async_trait]
        impl SlashArgumentHack<$type> for &&PhantomData<$type> {
            async fn extract(
                self,
                ctx: &serenity::Context,
                guild: Option<serenity::GuildId>,
                channel: Option<serenity::ChannelId>,
                value: &serenity::json::Value,
            ) -> Result<$type, SlashArgError> {
                // We can parse IDs by falling back to the generic serenity::ArgumentConvert impl
                PhantomData::<$type>
                    .extract(ctx, guild, channel, value)
                    .await
            }

            fn create(
                self,
                builder: &mut serenity::CreateApplicationCommandOption,
            ) -> &mut serenity::CreateApplicationCommandOption {
                builder.kind(serenity::ApplicationCommandOptionType::$slash_param_type)
            }
        }
    };
}
impl_slash_argument!(serenity::Member, User);
impl_slash_argument!(serenity::User, User);
impl_slash_argument!(serenity::Channel, Channel);
impl_slash_argument!(serenity::GuildChannel, Channel);
impl_slash_argument!(serenity::Role, Role);

#[doc(hidden)]
#[macro_export]
macro_rules! _parse_slash {
    // Extract Option<T>
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: Option<$type:ty $(,)*>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == stringify!($name)) {
            let arg = arg.value
            .as_ref()
            .ok_or($crate::SlashArgError::CommandStructureMismatch("expected argument value"))?;
            Some(
                (&&&&&std::marker::PhantomData::<$type>)
                .extract($ctx, $guild_id, Some($channel_id), arg)
                .await?
            )
        } else {
            None
        }
    };

    // Extract Vec<T> (delegating to Option<T> because slash commands don't support variadic
    // arguments right now)
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: Vec<$type:ty $(,)*>) => {
        match $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<$type>) {
            Some(value) => vec![value],
            None => vec![],
        }
    };

    // Extract #[flag]
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: FLAG) => {
        $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<bool>)
            .unwrap_or(false)
    };

    // Extract T
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: $($type:tt)*) => {
        $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<$($type)*>)
            .ok_or($crate::SlashArgError::CommandStructureMismatch("a required argument is missing"))?
    };
}

/**
Macro for extracting and parsing slash command arguments out of an array of
[`serenity::ApplicationCommandInteractionDataOption`].

An invocation of this macro is generated by `crate::command`, so you usually don't need this macro
directly.

```rust,no_run
# #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
# use poise::serenity_prelude as serenity;
let ctx: serenity::Context = todo!();
let guild_id: Option<serenity::GuildId> = todo!();
let channel_id: serenity::ChannelId = todo!();
let args: &[serenity::ApplicationCommandInteractionDataOption] = todo!();

let (arg1, arg2) = poise::parse_slash_args!(
    &ctx, guild_id, channel_id,
    args => (arg1: String), (arg2: Option<u32>)
).await?;

# Ok(()) }
```
*/
#[macro_export]
macro_rules! parse_slash_args {
    ($ctx:expr, $guild_id:expr, $channel_id:expr, $args:expr => $(
        ( $name:ident: $($type:tt)* )
    ),* $(,)? ) => {
        async /* not move! */ {
            use $crate::SlashArgumentHack;

            let (ctx, guild_id, channel_id, args) = ($ctx, $guild_id, $channel_id, $args);

            Ok::<_, $crate::SlashArgError>(( $(
                $crate::_parse_slash!( ctx, guild_id, channel_id, args => $name: $($type)* ),
            )* ))
        }
    };
}
