//! Trait implemented for all types usable as prefix command parameters.

use std::str::FromStr;

use super::{InvalidBool, MissingAttachment, TooFewArguments, pop_string};
use crate::argument_convert::ArgumentConvert;
use crate::serenity_prelude as serenity;

/// The result of [`PopArgument::pop_from`].
///  - If Ok, this is `(remaining, attachment_index, T)`
///  - If Err, this is `(error, failing_arg)`
pub(crate) type PopArgumentResult<'a, T> =
    Result<(&'a str, usize, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>;

/// Parse a value out of a string by popping off the front of the string. Discord message context
/// is available for parsing, and IO may be done as part of the parsing.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's sake and also because it keeps open the possibility of parsing
/// whitespace.
///
/// Similar in spirit to [`std::str::FromStr`].
#[async_trait::async_trait]
pub trait PopArgument<'a>: Sized {
    /// Pops an argument from the `args` string, and parses it as `Self`.
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self>;
}

#[async_trait::async_trait]
impl<'a> PopArgument<'a> for bool {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let (args, string) = pop_string(args).map_err(|e| (e.into(), None))?;

        let value = match string.to_ascii_lowercase().trim() {
            "yes" | "y" | "true" | "t" | "1" | "enable" | "on" => true,
            "no" | "n" | "false" | "f" | "0" | "disable" | "off" => false,
            _ => return Err((InvalidBool::default().into(), Some(string))),
        };

        Ok((args.trim_start(), attachment_index, value))
    }
}

#[async_trait::async_trait]
impl<'a> PopArgument<'a> for serenity::Attachment {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let attachment = msg
            .attachments
            .get(attachment_index)
            .ok_or_else(|| (MissingAttachment::default().into(), None))?
            .clone(); // `.clone()` is more clear than `.to_owned()` and is the same.

        Ok((args, attachment_index + 1, attachment))
    }
}

#[async_trait::async_trait]
impl<'a> PopArgument<'a> for String {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        match pop_string(args) {
            Ok((args, string)) => Ok((args, attachment_index, string)),
            Err(err) => Err((err.into(), Some(args.into()))),
        }
    }
}

/// Pops a string and then converts it to `T` using its `FromStr` implementation.
macro_rules! from_str_pop_argument {
    ( $(
        $type:ty,
    )* ) => {
        $(
            #[async_trait::async_trait]
            impl<'a> PopArgument<'a> for $type {
                async fn pop_from(
                    args: &'a str,
                    attachment_index: usize,
                    ctx: &serenity::Context,
                    msg: &serenity::Message,
                ) -> PopArgumentResult<'a, Self>
                where
                    Self: FromStr,
                {
                    let (args, string) = pop_string(args).map_err(|e| (e.into(), None))?;
                    let object = Self::from_str(&string).map_err(|e| (e.into(), Some(string)))?;
                    Ok((args.trim_start(), attachment_index, object))
                }
            }
        )*
    }
}

from_str_pop_argument! {
    f32, f64,
    u8, u16, u32, u64,
    i8, i16, i32, i64,
    serenity::Mention,
}

/// Pops a string and then converts it to `T` using its `ArgumentConvert` implementation.
macro_rules! argumentconvert_pop_argument {
    ( $(
        $( #[cfg(feature = $feature:literal)] )?
        $type:ty,
    )* ) => {
        $(
            $( #[cfg(feature = $feature)] )?
            #[async_trait::async_trait]
            impl<'a> PopArgument<'a> for $type {
                async fn pop_from(
                    args: &'a str,
                    attachment_index: usize,
                    ctx: &serenity::Context,
                    msg: &serenity::Message,
                ) -> PopArgumentResult<'a, Self>
                where
                    Self: ArgumentConvert,
                {
                    let (args, string) = pop_string(args).map_err(|e| (e.into(), None))?;
                    let object = Self::convert(ctx, msg.guild_id, Some(msg.channel_id), &string)
                        .await
                        .map_err(|e| (e.into(), Some(string)))?;

                    Ok((args.trim_start(), attachment_index, object))
                }
            }
        )*
    }
}

argumentconvert_pop_argument! {
    serenity::User, serenity::Member,
    serenity::Message,
    serenity::Channel, serenity::GuildChannel,
    serenity::EmojiId, serenity::Emoji,
    serenity::Role,

    serenity::GuildId,
    #[cfg(feature = "cache")]
    serenity::Guild,
}

/// Macro to allow for using mentions in snowflake types
macro_rules! snowflake_pop_argument {
    ($type:ty, $parse_fn:ident, $error_type:ident) => {
        /// Error thrown when the user enters a string that cannot be parsed correctly.
        #[derive(Default, Debug)]
        pub struct $error_type {
            #[doc(hidden)]
            pub __non_exhaustive: (),
        }

        impl std::error::Error for $error_type {}
        impl std::fmt::Display for $error_type {
            fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
                f.write_str(concat!("Enter a valid ", stringify!($error_type), " ID or a mention."))
            }
        }

        #[async_trait::async_trait]
        impl<'a> PopArgument<'a> for $type {
            async fn pop_from(
                args: &'a str,
                attachment_index: usize,
                ctx: &serenity::Context,
                msg: &serenity::Message,
            ) -> PopArgumentResult<'a, Self> {
                let (args, string) = pop_string(args).map_err(|e| (e.into(), None))?;

                if let Some(parsed_id) =
                    string.parse().ok().or_else(|| serenity::utils::$parse_fn(&string))
                {
                    Ok((args.trim_start(), attachment_index, parsed_id))
                } else {
                    Err(($error_type::default().into(), Some(string)))
                }
            }
        }
    };
}

snowflake_pop_argument!(serenity::UserId, parse_user_mention, InvalidUserId);
snowflake_pop_argument!(serenity::GenericChannelId, parse_channel_mention, InvalidChannelId);
snowflake_pop_argument!(serenity::RoleId, parse_role_mention, InvalidRoleId);
