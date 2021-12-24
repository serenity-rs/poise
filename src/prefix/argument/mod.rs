#![allow(unused)] // false positive from inside macro expansions

mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod macros;
pub use macros::*;

use crate::serenity_prelude as serenity;

/// Parse a value out of a string by popping off the front of the string. Discord message context
/// is available for parsing, and IO may be done as part of the parsing.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's
/// sake and also because it keeps open the possibility of parsing whitespace.
///
/// Similar in spirit to [`std::str::FromStr`].
pub trait PopArgument<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    /// Parse [`Self`] from the front of the given string and return a tuple of the remaining string
    /// and [`Self`].
    ///
    /// If parsing fails because the string is empty, `Err(None)` should be returned
    fn pop_from(args: &'a str) -> Result<(&'a str, Self), Option<Self::Err>>;
}

/// Error thrown if user passes too many arguments to a command
#[derive(Debug)]
pub struct TooManyArguments;
impl std::fmt::Display for TooManyArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too many arguments were passed")
    }
}
impl std::error::Error for TooManyArguments {}

/// Error thrown if user passes too few arguments to a command
#[derive(Debug)]
pub struct TooFewArguments;
impl std::fmt::Display for TooFewArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too few arguments were passed")
    }
}
impl std::error::Error for TooFewArguments {}

/// Error thrown when the user enters a string that is not recognized by a
/// SlashChoiceParameter-derived enum
#[derive(Debug)]
pub struct InvalidChoice;
impl std::fmt::Display for InvalidChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("You entered a non-existent choice")
    }
}
impl std::error::Error for InvalidChoice {}

#[doc(hidden)]
#[async_trait::async_trait]
pub trait PrefixArgumentHack<'a, T> {
    async fn pop(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>;
}

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert + Send> PrefixArgumentHack<'a, T>
    for std::marker::PhantomData<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn pop(
        self,
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        let (args, string) = String::pop_from(args).map_err(|_| (TooFewArguments.into(), None))?;
        let object = T::convert(ctx, msg.guild_id, Some(msg.channel_id), &string)
            .await
            .map_err(|e| (e.into(), Some(string)))?;

        Ok((args.trim_start(), object))
    }
}

#[async_trait::async_trait]
impl<'a, T: crate::PopArgument<'a> + Sync> PrefixArgumentHack<'a, T>
    for &std::marker::PhantomData<T>
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn pop(
        self,
        args: &'a str,
        _: &serenity::Context,
        _: &serenity::Message,
    ) -> Result<(&'a str, T), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        let (args, object) = T::pop_from(args)
            .map_err(|e| (e.map_or(TooFewArguments.into(), |e| e.into()), None))?;

        Ok((args.trim_start(), object))
    }
}
