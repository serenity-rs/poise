#![allow(unused)] // false positive from inside macro expansions

mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod wrapper;
pub use wrapper::*;

mod parse;
pub use parse::*;

use crate::serenity_prelude as serenity;

/// Type used throughout the prefix parameter parsing code in this code to store the raw string input.
///
/// Deliberately not `Copy` with the intention to prevent accidental copies and confusion
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ArgString<'a>(pub &'a str);

impl<'a> ArgString<'a> {
    /// Parse a single argument and return the remaining arguments.
    ///
    /// Uses `crate::PopArgumentAsync` internally.
    ///
    /// ```rust
    /// # use poise::{ArgString, CodeBlock};
    /// let args = ArgString("hello `foo bar`");
    ///
    /// let (args, hello) = args.sync_pop::<String>().unwrap();
    /// assert_eq!(hello, "hello");
    ///
    /// let (args, block) = args.sync_pop::<CodeBlock>().unwrap();
    /// assert_eq!(block.code, "foo bar");
    /// ```
    pub async fn pop<T: PopArgumentAsync<'a>>(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(ArgString<'a>, T), <T as PopArgumentAsync<'a>>::Err> {
        let (args, obj) = T::async_pop_from(ctx, msg, self).await?;
        Ok((ArgString(args.0.trim_start()), obj))
    }

    /// Like [`Self::pop`] but synchronous.
    pub fn sync_pop<T: PopArgument<'a>>(
        &self,
    ) -> Result<(ArgString<'a>, T), <T as PopArgumentAsync<'a>>::Err> {
        let (args, obj) = T::pop_from(self)?;
        Ok((ArgString(args.0.trim_start()), obj))
    }
}

/// Superset of [`PopArgumentAsync`] without Discord context available and no async support.
///
/// Similar in spirit to [`std::str::FromStr`].
pub trait PopArgument<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    /// Parse [`Self`] from the front of the given string and return a tuple of the remaining string
    /// and [`Self`].
    fn pop_from(args: &ArgString<'a>) -> Result<(ArgString<'a>, Self), Self::Err>;
}

/// Parse a value out of a string by popping off the front of the string. Discord message context
/// is available for parsing, and IO may be done as part of the parsing.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's
/// sake and also because it keeps open the possibility of parsing whitespace.
#[async_trait::async_trait]
pub trait PopArgumentAsync<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    /// Parse [`Self`] from the front of the given string and return a tuple of the remaining string
    /// and [`Self`].
    async fn async_pop_from(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err>;
}

#[async_trait::async_trait]
impl<'a, T> PopArgumentAsync<'a> for T
where
    T: PopArgument<'a>,
{
    type Err = <Self as PopArgument<'a>>::Err;

    async fn async_pop_from(
        _: &serenity::Context,
        _: &serenity::Message,
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err> {
        <Self as PopArgument>::pop_from(args)
    }
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

/// The error type returned from [parse_prefix_args!]. It contains a `Box<dyn Error>`
#[derive(Debug)]
pub struct ArgumentParseError(pub Box<dyn std::error::Error + Send + Sync>);

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse argument: {}", self.0)
    }
}

impl std::error::Error for ArgumentParseError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&*self.0)
    }
}

/// Emitted when the user enters a string that is not recognized by a SlashChoiceParameter-derived
/// enum
#[derive(Debug)]
pub struct InvalidChoice;

impl std::fmt::Display for InvalidChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("You entered a non-existent choice")
    }
}

impl std::error::Error for InvalidChoice {}
