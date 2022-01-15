//! Trait implemented for all types usable as prefix command parameters. This file also includes
//! the auto-deref specialization emulation code to e.g. support more strings for bool parameters
//! instead of the FromStr ones

use super::{pop_string, InvalidBool, TooFewArguments};
use crate::serenity_prelude as serenity;
use std::marker::PhantomData;

/// Full version of [`crate::PopArgument::pop_from`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! pop_prefix_argument {
    ($target:ty, $args:expr, $ctx:expr, $msg:expr) => {{
        use $crate::PopArgumentHack as _;
        (&std::marker::PhantomData::<$target>).pop_from($args, $ctx, $msg)
    }};
}

/// Parse a value out of a string by popping off the front of the string. Discord message context
/// is available for parsing, and IO may be done as part of the parsing.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's
/// sake and also because it keeps open the possibility of parsing whitespace.
///
/// Similar in spirit to [`std::str::FromStr`].
#[async_trait::async_trait]
pub trait PopArgument<'a>: Sized {
    /// Parse [`Self`] from the front of the given string and return a tuple of the remaining string
    /// and [`Self`]. If parsing failed, an error is returned and, if applicable, the string on
    /// which parsing failed.
    ///
    /// If parsing fails because the string is empty, use the `TooFewArguments` type as the error.
    ///
    /// Don't call this method directly! Use [`crate::pop_prefix_argument!`]
    async fn pop_from(
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, Self), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>;
}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait PopArgumentHack<'a, T>: Sized {
    async fn pop_from(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>;
}

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert + Send> PopArgumentHack<'a, T> for PhantomData<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn pop_from(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        let (args, string) = pop_string(args).map_err(|_| (TooFewArguments.into(), None))?;
        let object = T::convert(ctx, msg.guild_id, Some(msg.channel_id), &string)
            .await
            .map_err(|e| (e.into(), Some(string)))?;

        Ok((args.trim_start(), object))
    }
}

#[async_trait::async_trait]
impl<'a, T: PopArgument<'a> + Send + Sync> PopArgumentHack<'a, T> for &PhantomData<T> {
    async fn pop_from(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        T::pop_from(args, ctx, msg).await
    }
}

#[async_trait::async_trait]
impl<'a> PopArgumentHack<'a, bool> for &PhantomData<bool> {
    async fn pop_from(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, bool), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        let (args, string) = pop_string(args).map_err(|_| (TooFewArguments.into(), None))?;

        let value = match string.to_ascii_lowercase().trim() {
            "yes" | "y" | "true" | "t" | "1" | "enable" | "on" => true,
            "no" | "n" | "false" | "f" | "0" | "disable" | "off" => false,
            _ => return Err((InvalidBool.into(), Some(string))),
        };

        Ok((args.trim_start(), value))
    }
}
