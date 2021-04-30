//! Parse received slash command arguments into Rust types.

use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

use crate::serenity_prelude as serenity;

#[derive(Debug)]
pub enum SlashArgError {
    CommandStructureMismatch(&'static str),
    Parse(Box<dyn std::error::Error + Send + Sync>),
    IntegerOutOfBounds,
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
            Self::Parse(e) => write!(f, "Failed to parse argument: {}", e),
            Self::IntegerOutOfBounds => write!(f, "Integer out of bounds for target type"),
        }
    }
}
impl std::error::Error for SlashArgError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Parse(e) => Some(&**e),
            Self::CommandStructureMismatch(_) => None,
            Self::IntegerOutOfBounds => None,
        }
    }
}

/// Implemented for all types that can be used as a function parameter in a slash command.
///
/// Currently marked `#[doc(hidden)]` because implementing this trait requires some jank due to a
/// `PhantomData` hack and the auto-deref specialization hack.
#[doc(hidden)]
#[async_trait::async_trait]
pub trait SlashArgument<T> {
    /// Extract a Rust value of type T from the slash command argument, given via a
    /// [`serde_json::Value`].
    async fn extract(
        self,
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serde_json::Value,
    ) -> Result<T, SlashArgError>;

    /// Create a slash command parameter equivalent to type T.
    ///
    /// Only fields about the argument type are filled in. The caller is still responsible for
    /// filling in `name()`, `description()`, and possibly `required()` or other fields.
    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption;
}

/// Handles arbitrary types that can be parsed from string.
#[async_trait::async_trait]
impl<T> SlashArgument<T> for PhantomData<T>
where
    T: serenity::Parse + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn extract(
        self,
        ctx: &serenity::Context,
        guild: Option<serenity::GuildId>,
        channel: Option<serenity::ChannelId>,
        value: &serde_json::Value,
    ) -> Result<T, SlashArgError> {
        let string = value
            .as_str()
            .ok_or(SlashArgError::CommandStructureMismatch("expected string"))?;
        T::parse(ctx, guild, channel, string)
            .await
            .map_err(|e| SlashArgError::Parse(e.into()))
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::String)
    }
}

// Handles all integers, signed and unsigned, via TryFrom<i64>.
#[async_trait::async_trait]
impl<T: TryFrom<i64> + Send + Sync> SlashArgument<T> for &PhantomData<T> {
    async fn extract(
        self,
        _: &serenity::Context,
        _: Option<serenity::GuildId>,
        _: Option<serenity::ChannelId>,
        value: &serde_json::Value,
    ) -> Result<T, SlashArgError> {
        value
            .as_i64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected integer"))?
            .try_into()
            .ok()
            .ok_or(SlashArgError::IntegerOutOfBounds)
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::Integer)
    }
}

#[async_trait::async_trait]
impl SlashArgument<bool> for &&PhantomData<bool> {
    async fn extract(
        self,
        _: &serenity::Context,
        _: Option<serenity::GuildId>,
        _: Option<serenity::ChannelId>,
        value: &serde_json::Value,
    ) -> Result<bool, SlashArgError> {
        value
            .as_bool()
            .ok_or(SlashArgError::CommandStructureMismatch("expected bool"))
    }

    fn create(
        self,
        builder: &mut serenity::CreateApplicationCommandOption,
    ) -> &mut serenity::CreateApplicationCommandOption {
        builder.kind(serenity::ApplicationCommandOptionType::Boolean)
    }
}

// Implement slash argument for a model type that is represented in interactions via an ID
macro_rules! impl_slash_argument {
    ($type:ty, $slash_param_type:ident) => {
        #[async_trait::async_trait]
        impl SlashArgument<$type> for &&PhantomData<$type> {
            async fn extract(
                self,
                ctx: &serenity::Context,
                guild: Option<serenity::GuildId>,
                channel: Option<serenity::ChannelId>,
                value: &serde_json::Value,
            ) -> Result<$type, SlashArgError> {
                // We can parse IDs by falling back to the generic serenity::Parse impl
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
// TODO: uncomment when the corresponding Parse impls in serenity are there
// impl_slash_argument!(serenity::User, User);
// impl_slash_argument!(serenity::Channel, Channel);
// impl_slash_argument!(serenity::Role, Role);

#[doc(hidden)]
#[macro_export]
macro_rules! _parse_slash {
    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: Option<$type:ty $(,)*>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == stringify!($name)) {
            let arg = arg.value
            .as_ref()
            .ok_or($crate::SlashArgError::CommandStructureMismatch("expected argument value"))?;
            Some(
                #[allow(clippy::eval_order_dependence)] // idk what it's going on about
                (&&std::marker::PhantomData::<$type>)
                .extract($ctx, $guild_id, $channel_id, arg)
                .await?
            )
        } else {
            None
        }
    };

    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: FLAG) => {
        $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<bool>)
            .unwrap_or(false)
    };

    ($ctx:ident, $guild_id:ident, $channel_id:ident, $args:ident => $name:ident: $($type:tt)*) => {
        $crate::_parse_slash!($ctx, $guild_id, $channel_id, $args => $name: Option<$($type)*>)
            .ok_or($crate::SlashArgError::CommandStructureMismatch("a required argument is missing"))?
    };
}

#[macro_export]
macro_rules! parse_slash_args {
    ($ctx:expr, $guild_id:expr, $channel_id:expr, $args:expr => $(
        ( $name:ident: $($type:tt)* )
    ),* $(,)? ) => {
        async /* not move! */ {
            use $crate::SlashArgument;

            let (ctx, guild_id, channel_id, args) = ($ctx, $guild_id, $channel_id, $args);

            Ok::<_, $crate::SlashArgError>(( $(
                $crate::_parse_slash!( ctx, guild_id, channel_id, args => $name: $($type)* ),
            )* ))
        }
    };
}
