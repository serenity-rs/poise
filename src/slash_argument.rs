//! Application command argument handling code

use crate::argument_convert::ArgumentConvert;
use crate::{BoxFuture, CowVec, serenity_prelude as serenity};

/// Implement this trait on types that you want to use as a slash command parameter.
#[async_trait::async_trait]
pub trait SlashArgument: Sized {
    /// Extract a Rust value of type T from the slash command argument, given via a
    /// [`serenity::ResolvedValue`].
    async fn extract(
        ctx: &serenity::Context,
        interaction: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<Self, SlashArgError>;

    /// Create a slash command parameter equivalent to type T.
    ///
    /// Only fields about the argument type are filled in. The caller is still responsible for
    /// filling in `name()`, `description()`, and possibly `required()` or other fields.
    fn create(builder: serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_>;

    /// If this is a choice parameter, returns the choices
    #[must_use]
    fn choices() -> CowVec<crate::CommandParameterChoice> {
        CowVec::default()
    }
}

/// Extracts a string argument and then converts it to `T` using its `ArgumentConvert`
/// implementation.
async fn extract_via_argumentconvert<T>(
    ctx: &serenity::Context,
    interaction: &serenity::CommandInteraction,
    value: &serenity::ResolvedValue<'_>,
) -> Result<T, SlashArgError>
where
    T: ArgumentConvert + Send + Sync,
{
    let string = match value {
        serenity::ResolvedValue::String(str) => *str,
        _ => {
            return Err(SlashArgError::CommandStructureMismatch { description: "expected string" });
        },
    };

    T::convert(ctx, interaction.guild_id, Some(interaction.channel_id), string)
        .await
        .map_err(|e| SlashArgError::Parse { error: e.into(), input: string.into() })
}

/// Auto-impls `SlashArgument` for a type by deferring to [`extract_via_argumentconvert`].
macro_rules! argumentconvert_slash_argument {
    ( $(
        $( #[cfg(feature = $feature:literal)] )?
        $type:ty,
    ) *) => {
        $(
            $( #[cfg(feature = $feature)] )?
            #[async_trait::async_trait]
            impl SlashArgument for $type {
                async fn extract(
                    ctx: &serenity::Context,
                    interaction: &serenity::CommandInteraction,
                    value: &serenity::ResolvedValue<'_>,
                ) -> Result<Self, SlashArgError> {
                    extract_via_argumentconvert(ctx, interaction, value).await
                }

                fn create(builder: serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_> {
                    builder.kind(serenity::CommandOptionType::String)
                }
            }
        )*
    }
}

argumentconvert_slash_argument! {
    serenity::Message,
    serenity::EmojiId, serenity::Emoji,
    #[cfg(feature = "cache")]
    serenity::GuildId,
    #[cfg(feature = "cache")]
    serenity::Guild,
}

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

            #[allow(clippy::cast_lossless, clippy::cast_precision_loss)]
            fn create(builder: serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_> {
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

            fn create(builder: serenity::CreateCommandOption<'_>) -> serenity::CreateCommandOption<'_> {
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
    member.ok_or(SlashArgError::Invalid("cannot use member parameter in DMs"))?.clone()
});
impl_slash_argument!(serenity::User, |_, _, User(user, _)| user.clone());
impl_slash_argument!(serenity::UserId, |_, _, User(user, _)| user.id);
impl_slash_argument!(serenity::Channel, |ctx, inter, Channel(channel)| {
    channel.id().to_channel(ctx, inter.guild_id).await.map_err(SlashArgError::Http)?
});
impl_slash_argument!(serenity::GenericChannelId, |_, _, Channel(channel)| channel.id());
impl_slash_argument!(serenity::GenericInteractionChannel, |_, _, Channel(channel)| channel.clone());
impl_slash_argument!(serenity::GuildChannel, |ctx, inter, Channel(channel)| {
    channel
        .id()
        .expect_channel()
        .to_guild_channel(ctx, inter.guild_id)
        .await
        .map_err(SlashArgError::Http)?
});
impl_slash_argument!(serenity::Role, |_, _, Role(role)| role.clone());
impl_slash_argument!(serenity::RoleId, |_, _, Role(role)| role.id);

/// Possible errors when parsing slash command arguments
#[derive(Debug)]
pub enum SlashArgError {
    /// Expected a certain argument type at a certain position in the unstructured list of
    /// arguments, but found something else.
    ///
    /// Most often the result of the bot not having registered the command in Discord, so Discord
    /// stores an outdated version of the command and its parameters.
    #[non_exhaustive]
    CommandStructureMismatch {
        /// A short string describing what specifically is wrong/unexpected
        description: &'static str,
    },
    /// A string parameter was found, but it could not be parsed into the target type.
    #[non_exhaustive]
    Parse {
        /// Error that occurred while parsing the string into the target type
        error: Box<dyn std::error::Error + Send + Sync>,
        /// Original input string
        input: String,
    },
    /// The argument passed by the user is invalid in this context. E.g. a Member parameter in DMs
    #[non_exhaustive]
    Invalid(
        /// Human readable description of the error
        &'static str,
    ),
    /// HTTP error occurred while retrieving the model type from Discord
    Http(serenity::Error),
    #[doc(hidden)]
    __NonExhaustive,
}

/// Support functions for macro which can't create #[non_exhaustive] enum variants
#[doc(hidden)]
impl SlashArgError {
    #[must_use]
    pub fn new_command_structure_mismatch(description: &'static str) -> Self {
        Self::CommandStructureMismatch { description }
    }

    #[must_use]
    pub fn new_parse(error: Box<dyn std::error::Error + Send + Sync>, input: String) -> Self {
        Self::Parse { error, input }
    }

    #[must_use]
    pub fn to_framework_error<U, E>(
        self,
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> crate::FrameworkError<'_, U, E> {
        match self {
            Self::CommandStructureMismatch { description } => {
                crate::FrameworkError::CommandStructureMismatch { ctx, description }
            },
            Self::Parse { error, input } => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error,
                input: Some(Box::new(input)),
            },
            Self::Invalid(description) => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error: description.into(),
                input: None,
            },
            Self::Http(error) => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error: error.into(),
                input: None,
            },
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

impl std::fmt::Display for SlashArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandStructureMismatch { description } => {
                write!(f, "Bot author did not register their commands correctly ({description})")
            },
            Self::Parse { error, input } => {
                write!(f, "Failed to parse `{input}` as argument: {error}")
            },
            Self::Invalid(description) => {
                write!(f, "You can't use this parameter here: {description}")
            },
            Self::Http(error) => {
                write!(f, "Error occurred while retrieving data from Discord: {error}")
            },
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

impl std::error::Error for SlashArgError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Http(error) => Some(error),
            Self::Parse { error, input: _ } => Some(&**error),
            Self::Invalid { .. } | Self::CommandStructureMismatch { .. } => None,
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

/// Implemented for all types that can be used in a context menu command
pub trait ContextMenuParameter<U, E> {
    /// Convert an action function pointer that takes Self as an argument into the appropriate
    /// [`crate::ContextMenuCommandAction`] variant.
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E>;
}

impl<U, E> ContextMenuParameter<U, E> for serenity::User {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E> {
        crate::ContextMenuCommandAction::User(action)
    }
}

impl<U, E> ContextMenuParameter<U, E> for serenity::Message {
    fn to_action(
        action: fn(
            crate::ApplicationContext<'_, U, E>,
            Self,
        ) -> BoxFuture<'_, Result<(), crate::FrameworkError<'_, U, E>>>,
    ) -> crate::ContextMenuCommandAction<U, E> {
        crate::ContextMenuCommandAction::Message(action)
    }
}
