//! Everything related to parsing command arguments from a text message

#![allow(unused)] // false positive from inside macro expansions

mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod macros;
pub use macros::*;

use crate::serenity_prelude as serenity;

/// Pop a whitespace-separated word from the front of the arguments. Supports quotes and quote
/// escaping.
///
/// Leading whitespace will be trimmed; trailing whitespace is not consumed.
fn pop_string(args: &str) -> Result<(&str, String), crate::TooFewArguments> {
    // TODO: consider changing the behavior to parse quotes literally if they're in the middle
    // of the string:
    // - `"hello world"` => `hello world`
    // - `"hello "world"` => `"hello "world`
    // - `"hello" world"` => `hello`

    let args = args.trim_start();
    if args.is_empty() {
        return Err(crate::TooFewArguments);
    }

    let mut output = String::new();
    let mut inside_string = false;
    let mut escaping = false;

    let mut chars = args.chars();
    // .clone().next() is poor man's .peek(), but we can't do peekable because then we can't
    // call as_str on the Chars iterator
    while let Some(c) = chars.clone().next() {
        if escaping {
            output.push(c);
            escaping = false;
        } else if !inside_string && c.is_whitespace() {
            break;
        } else if c == '"' {
            inside_string = !inside_string;
        } else if c == '\\' {
            escaping = true;
        } else {
            output.push(c);
        }

        chars.next();
    }

    Ok((chars.as_str(), output))
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
    async fn pop_from(
        args: &'a str,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, Self), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>;
}

#[async_trait::async_trait]
impl<'a, T: serenity::ArgumentConvert + Send> PopArgument<'a> for T
where
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn pop_from(
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

#[cfg(test)]
#[test]
fn test_pop_string() {
    // Test that trailing whitespace is not consumed
    assert_eq!(pop_string("AA BB").unwrap().0, " BB");

    for &(string, arg) in &[
        (r#"AA BB"#, r#"AA"#),
        (r#""AA BB""#, r#"AA BB"#),
        (r#""AA BB"#, r#"AA BB"#),
        (r#""AA "BB"#, r#"AA BB"#),
        (r#"""""A""A" "B"""B"#, r#"AA BB"#),
        (r#"\"AA BB\""#, r#""AA"#),
        (r#"\"AA\ BB\""#, r#""AA BB""#),
        (r#""\"AA BB\"""#, r#""AA BB""#),
    ] {
        assert_eq!(pop_string(string).unwrap().1, arg);
    }
}
