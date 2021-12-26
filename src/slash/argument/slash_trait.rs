//! Traits for slash command parameters and a macro to wrap the auto-deref specialization hack

use super::SlashArgError;
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

#[allow(unused_imports)] // import is required if serenity simdjson feature is enabled
use crate::serenity::json::prelude::*;
use crate::serenity_prelude as serenity;

/// Implement this trait on types that you want to use as a slash command parameter.
#[async_trait::async_trait]
pub trait SlashArgument: Sized {
    /// Extract a Rust value of type T from the slash command argument, given via a
    /// [`serenity::json::Value`].
    ///
    /// Don't call this method directly! Use [`crate::extract_slash_argument!`]
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
    ///
    /// Don't call this method directly! Use [`crate::create_slash_argument!`]
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

/// Full version of [`crate::SlashArgument::extract`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! extract_slash_argument {
    ($target:ty, $ctx:expr, $guild:expr, $channel:expr, $value:expr) => {{
        use $crate::SlashArgumentHack as _;
        (&&&&&std::marker::PhantomData::<$target>).extract($ctx, $guild, $channel, $value)
    }};
}
/// Full version of [`crate::SlashArgument::create`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! create_slash_argument {
    ($target:ty, $builder:expr) => {{
        use $crate::SlashArgumentHack as _;
        (&&&&&std::marker::PhantomData::<$target>).create($builder)
    }};
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
