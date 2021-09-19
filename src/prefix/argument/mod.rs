#![allow(unused)] // false positive from inside macro expansions

mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod parse;
pub use parse::*;

use crate::serenity_prelude as serenity;

/// Type used throughout the prefix parameter parsing code in this code to store the raw string input.
///
/// Deliberately not `Copy` with the intention to prevent accidental copies and confusion
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ArgString<'a>(pub &'a str);

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

#[doc(hidden)]
#[async_trait::async_trait]
pub trait PrefixArgumentHack<'a, T> {
    type Err;

    async fn pop(
        self,
        args: &ArgString<'a>,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(ArgString<'a>, T), Self::Err>;
}

/// When attempting to parse a string, it can fail either because it's empty, or because it's
/// invalid in some way. This error type covers both cases
#[derive(Debug)]
pub enum MaybeEmptyError<E> {
    /// If the input was empty and [`Wrapper`] was unable to pass any string to the underlying type
    EmptyArgs(crate::EmptyArgs),
    /// The underlying type threw a parse error
    ParseError(E),
}

impl<E: std::fmt::Display> std::fmt::Display for MaybeEmptyError<E> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            MaybeEmptyError::EmptyArgs(e) => e.fmt(f),
            MaybeEmptyError::ParseError(e) => e.fmt(f),
        }
    }
}

impl<E: std::error::Error + 'static> std::error::Error for MaybeEmptyError<E> {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            MaybeEmptyError::EmptyArgs(e) => Some(e),
            MaybeEmptyError::ParseError(e) => Some(e),
        }
    }
}

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert + Send> PrefixArgumentHack<'a, T>
    for std::marker::PhantomData<T>
{
    type Err = MaybeEmptyError<T::Err>;

    async fn pop(
        self,
        args: &ArgString<'a>,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(ArgString<'a>, T), Self::Err> {
        let (args, string) = String::pop_from(args).map_err(MaybeEmptyError::EmptyArgs)?;
        let object = T::convert(ctx, msg.guild_id, Some(msg.channel_id), &string)
            .await
            .map_err(MaybeEmptyError::ParseError)?;

        Ok((ArgString(args.0.trim_start()), object))
    }
}

#[async_trait::async_trait]
impl<'a, T: crate::PopArgument<'a> + Sync> PrefixArgumentHack<'a, T>
    for &std::marker::PhantomData<T>
{
    type Err = <T as crate::PopArgument<'a>>::Err;

    async fn pop(
        self,
        args: &ArgString<'a>,
        _: &serenity::Context,
        _: &serenity::Message,
    ) -> Result<(ArgString<'a>, T), Self::Err> {
        let (args, object) = T::pop_from(args)?;

        Ok((ArgString(args.0.trim_start()), object))
    }
}
