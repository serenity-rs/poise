//! Traits for slash command parameters and a macro to wrap the auto-deref specialization hack

use super::SlashArgError;
use std::convert::TryInto as _;
use std::marker::PhantomData;

#[allow(unused_imports)] // import is required if serenity simdjson feature is enabled
use crate::serenity::json::*;
use crate::serenity_prelude as serenity;

/// Implement this trait on types that you want to use as a slash command parameter.
#[async_trait::async_trait]
pub trait SlashArgument: Sized {
    /// Extract a Rust value of type T from the slash command argument, given via a
    /// [`serenity::json::Value`].
    ///
    /// Don't call this method directly! Use [`crate::extract_slash_argument!`]
    async fn extract<U: Send + Sync + 'static>(
        ctx: &serenity::Context<U>,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<Self, SlashArgError>;

    /// Create a slash command parameter equivalent to type T.
    ///
    /// Only fields about the argument type are filled in. The caller is still responsible for
    /// filling in `name()`, `description()`, and possibly `required()` or other fields.
    ///
    /// Don't call this method directly! Use [`crate::create_slash_argument!`]
    fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption;

    /// If this is a choice parameter, returns the choices
    ///
    /// Don't call this method directly! Use [`crate::slash_argument_choices!`]
    fn choices() -> Vec<crate::CommandParameterChoice> {
        Vec::new()
    }
}

/// Implemented for all types that can be used as a function parameter in a slash command.
///
/// Currently marked `#[doc(hidden)]` because implementing this trait requires some jank due to a
/// `PhantomData` hack and the auto-deref specialization hack.
#[doc(hidden)]
#[async_trait::async_trait]
pub trait SlashArgumentHack<T>: Sized {
    async fn extract<U: Send + Sync + 'static>(
        self,
        ctx: &serenity::Context<U>,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<T, SlashArgError>;

    fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption;

    fn choices(self) -> Vec<crate::CommandParameterChoice> {
        Vec::new()
    }
}

/// Full version of [`crate::SlashArgument::extract`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! extract_slash_argument {
    ($target:ty, $ctx:expr, $interaction:expr, $value:expr) => {{
        use $crate::SlashArgumentHack as _;
        (&&std::marker::PhantomData::<$target>).extract($ctx, $interaction, $value)
    }};
}
/// Full version of [`crate::SlashArgument::create`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! create_slash_argument {
    ($target:ty, $builder:expr) => {{
        use $crate::SlashArgumentHack as _;
        (&&std::marker::PhantomData::<$target>).create($builder)
    }};
}
/// Full version of [`crate::SlashArgument::choices`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! slash_argument_choices {
    ($target:ty) => {{
        use $crate::SlashArgumentHack as _;
        (&&std::marker::PhantomData::<$target>).choices()
    }};
}

/// Handles arbitrary types that can be parsed from string.
#[async_trait::async_trait]
impl<T> SlashArgumentHack<T> for PhantomData<T>
where
    T: serenity::ArgumentConvert + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn extract<U: Send + Sync + 'static>(
        self,
        ctx: &serenity::Context<U>,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<T, SlashArgError> {
        let string = match value {
            serenity::ResolvedValue::String(str) => *str,
            _ => {
                return Err(SlashArgError::CommandStructureMismatch {
                    description: "expected string",
                })
            }
        };

        T::convert(
            ctx,
            interaction.guild_id,
            Some(interaction.channel_id),
            string,
        )
        .await
        .map_err(|e| SlashArgError::Parse {
            error: e.into(),
            input: string.into(),
        })
    }

    fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        builder.kind(serenity::CommandOptionType::String)
    }
}

/// Implements slash argument trait for integer types
macro_rules! impl_for_integer {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl SlashArgument for $t {
            async fn extract<U: Send + Sync + 'static>(
                _: &serenity::Context<U>,
                _: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<$t, SlashArgError> {
                let value = match value {
                    serenity::ResolvedValue::Integer(int) => *int,
                    _ => return Err(SlashArgError::CommandStructureMismatch { description: "expected integer" })
                };

                value
                    .try_into()
                    .map_err(|_| SlashArgError::CommandStructureMismatch { description: "received out of bounds integer" })
            }

            fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
                builder
                    .min_number_value(f64::max(<$t>::MIN as f64, -9007199254740991.))
                    .max_number_value(f64::min(<$t>::MAX as f64, 9007199254740991.))
                    .kind(serenity::CommandOptionType::Integer)
            }
        }
    )* };
}
impl_for_integer!(i8 i16 i32 i64 isize u8 u16 u32 u64 usize);

/// Implements slash argument trait for float types
macro_rules! impl_for_float {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl SlashArgumentHack<$t> for &PhantomData<$t> {
            async fn extract<U: Send + Sync + 'static>(
                self,
                _: &serenity::Context<U>,
                _: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<$t, SlashArgError> {
                match value {
                    serenity::ResolvedValue::Number(float) => Ok(*float as $t),
                    _ => Err(SlashArgError::CommandStructureMismatch { description: "expected float" })
                }
            }

            fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
                builder.kind(serenity::CommandOptionType::Number)
            }
        }
    )* };
}
impl_for_float!(f32 f64);

#[async_trait::async_trait]
impl SlashArgumentHack<bool> for &PhantomData<bool> {
    async fn extract<U: Send + Sync + 'static>(
        self,
        _: &serenity::Context<U>,
        _: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<bool, SlashArgError> {
        match value {
            serenity::ResolvedValue::Boolean(val) => Ok(*val),
            _ => Err(SlashArgError::CommandStructureMismatch {
                description: "expected bool",
            }),
        }
    }

    fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        builder.kind(serenity::CommandOptionType::Boolean)
    }
}

#[async_trait::async_trait]
impl SlashArgumentHack<serenity::Attachment> for &PhantomData<serenity::Attachment> {
    async fn extract<U: Send + Sync + 'static>(
        self,
        _: &serenity::Context<U>,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<serenity::Attachment, SlashArgError> {
        let attachment_id = match value {
            serenity::ResolvedValue::String(val) => {
                val.parse()
                    .map_err(|_| SlashArgError::CommandStructureMismatch {
                        description: "improper attachment id passed",
                    })?
            }
            _ => {
                return Err(SlashArgError::CommandStructureMismatch {
                    description: "expected attachment id",
                })
            }
        };

        interaction
            .data
            .resolved
            .attachments
            .get(&attachment_id)
            .cloned()
            .ok_or(SlashArgError::CommandStructureMismatch {
                description: "attachment id with no attachment",
            })
    }

    fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        builder.kind(serenity::CommandOptionType::Attachment)
    }
}

#[async_trait::async_trait]
impl<T: SlashArgument + Sync> SlashArgumentHack<T> for &PhantomData<T> {
    async fn extract<U: Send + Sync + 'static>(
        self,
        ctx: &serenity::Context<U>,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<T, SlashArgError> {
        <T as SlashArgument>::extract(ctx, interaction, value).await
    }

    fn create(self, builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        <T as SlashArgument>::create(builder)
    }

    fn choices(self) -> Vec<crate::CommandParameterChoice> {
        <T as SlashArgument>::choices()
    }
}

/// Implements `SlashArgumentHack` for a model type that is represented in interactions via an ID
macro_rules! impl_slash_argument {
    ($type:ty, $slash_param_type:ident) => {
        #[async_trait::async_trait]
        impl SlashArgument for $type {
            async fn extract<U: Send + Sync + 'static>(
                ctx: &serenity::Context<U>,
                interaction: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<$type, SlashArgError> {
                // We can parse IDs by falling back to the generic serenity::ArgumentConvert impl
                PhantomData::<$type>.extract(ctx, interaction, value).await
            }

            fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
                builder.kind(serenity::CommandOptionType::$slash_param_type)
            }
        }
    };
}
impl_slash_argument!(serenity::Member, User);
impl_slash_argument!(serenity::User, User);
impl_slash_argument!(serenity::Channel, Channel);
impl_slash_argument!(serenity::GuildChannel, Channel);
impl_slash_argument!(serenity::Role, Role);
