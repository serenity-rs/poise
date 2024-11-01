//! Traits for slash command parameters and a macro to wrap the auto-deref specialization hack

use super::SlashArgError;
use std::{borrow::Cow, convert::TryInto as _};

#[allow(unused_imports)] // import is required if serenity simdjson feature is enabled
use crate::serenity::json::*;
use crate::{serenity_prelude as serenity, CowVec};

/// Implement this trait on types that you want to use as a slash command parameter.
#[async_trait::async_trait]
pub trait SlashArgument: Sized {
    /// Extract a Rust value of type T from the slash command argument, given via a [`serenity::ResolvedValue`].
    async fn extract(
        ctx: &serenity::Context,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<Self, SlashArgError>;

    /// Create a slash command parameter equivalent to type T.
    ///
    /// Only fields about the argument type are filled in. The caller is still responsible for
    /// filling in `name()`, `description()`, and possibly `required()` or other fields.
    fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption;

    /// If this is a choice parameter, returns the choices
    fn choices() -> CowVec<crate::CommandParameterChoice> {
        Cow::Borrowed(&[])
    }
}

/// Converts a Command value via serenity's ArgumentConvert trait
async fn extract_via_argumentconvert<T>(
    ctx: &serenity::Context,
    interaction: &serenity::CommandInteraction,
    value: &serenity::ResolvedValue<'_>,
) -> Result<T, SlashArgError>
where
    T: serenity::ArgumentConvert + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
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

/// Implements `SlashArgument` via `serenity::ArgumentConvert`
macro_rules! impl_for_argumentconvert {
    ($type:ty) => {
        #[async_trait::async_trait]
        impl SlashArgument for $type {
            async fn extract(
                ctx: &serenity::Context,
                interaction: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<Self, SlashArgError> {
                extract_via_argumentconvert::<$type>(ctx, interaction, value).await
            }

            fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
                builder.kind(serenity::CommandOptionType::String)
            }
        }
    };
}

impl_for_argumentconvert!(serenity::Message);

/// Implements slash argument trait for integer types
macro_rules! impl_for_integer {
    ($($t:ty)*) => { $(
        #[async_trait::async_trait]
        impl SlashArgument for $t {
            async fn extract(
                _: &serenity::Context,
                _: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<$t, SlashArgError> {
                match *value {
                    serenity::ResolvedValue::Integer(x) => x
                        .try_into()
                        .map_err(|_| SlashArgError::CommandStructureMismatch {
                            description: "received out of bounds integer",
                        }),
                    _ => Err(SlashArgError::CommandStructureMismatch {
                        description: "expected integer",
                    }),
                }
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

/// Versatile macro to implement `SlashArgument` for simple types
macro_rules! impl_slash_argument {
    ($type:ty, |$ctx:pat, $interaction:pat, $slash_param_type:ident ( $($arg:pat),* )| $extractor:expr) => {
        #[async_trait::async_trait]
        impl SlashArgument for $type {
            async fn extract(
                $ctx: &serenity::Context,
                $interaction: &serenity::CommandInteraction,
                value: &serenity::ResolvedValue<'_>,
            ) -> Result<$type, SlashArgError> {
                match *value {
                    serenity::ResolvedValue::$slash_param_type( $($arg),* ) => Ok( $extractor ),
                    _ => Err(SlashArgError::CommandStructureMismatch {
                        description: concat!("expected ", stringify!($slash_param_type))
                    }),
                }
            }

            fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
                builder.kind(serenity::CommandOptionType::$slash_param_type)
            }
        }
    };
}

impl_slash_argument!(f32, |_, _, Number(x)| x as f32);
impl_slash_argument!(f64, |_, _, Number(x)| x);
impl_slash_argument!(bool, |_, _, Boolean(x)| x);
impl_slash_argument!(String, |_, _, String(x)| x.into());
impl_slash_argument!(serenity::Attachment, |_, _, Attachment(att)| att.clone());
impl_slash_argument!(serenity::Member, |ctx, interaction, User(user, _)| {
    interaction
        .guild_id
        .ok_or(SlashArgError::Invalid("cannot use member parameter in DMs"))?
        .member(ctx, user.id)
        .await
        .map_err(SlashArgError::Http)?
});
impl_slash_argument!(serenity::PartialMember, |_, _, User(_, member)| {
    member
        .ok_or(SlashArgError::Invalid("cannot use member parameter in DMs"))?
        .clone()
});
impl_slash_argument!(serenity::User, |_, _, User(user, _)| user.clone());
impl_slash_argument!(serenity::UserId, |_, _, User(user, _)| user.id);
impl_slash_argument!(serenity::Channel, |ctx, _, Channel(channel)| {
    channel
        .id
        .to_channel(ctx)
        .await
        .map_err(SlashArgError::Http)?
});
impl_slash_argument!(serenity::ChannelId, |_, _, Channel(channel)| channel.id);
impl_slash_argument!(serenity::PartialChannel, |_, _, Channel(channel)| channel
    .clone());
impl_slash_argument!(serenity::GuildChannel, |ctx, _, Channel(channel)| {
    let channel_res = channel.id.to_channel(ctx).await;
    let channel = channel_res.map_err(SlashArgError::Http)?.guild();
    channel.ok_or(SlashArgError::Http(serenity::Error::Model(
        serenity::ModelError::InvalidChannelType,
    )))?
});
impl_slash_argument!(serenity::Role, |_, _, Role(role)| role.clone());
impl_slash_argument!(serenity::RoleId, |_, _, Role(role)| role.id);
