//! Everything related to parsing command arguments from a text message

#![allow(unused)] // false positive from inside macro expansions

mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod macros;
pub use macros::*;

mod argument_trait;
pub use argument_trait::*;

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
        return Err(crate::TooFewArguments::default());
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

/// Error thrown if user passes too many arguments to a command
#[derive(Default, Debug)]
pub struct TooManyArguments {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for TooManyArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too many arguments were passed")
    }
}
impl std::error::Error for TooManyArguments {}

/// Error thrown if user passes too few arguments to a command
#[derive(Default, Debug)]
pub struct TooFewArguments {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for TooFewArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too few arguments were passed")
    }
}
impl std::error::Error for TooFewArguments {}

/// Error thrown in prefix invocation when there's too few attachments
#[derive(Default, Debug)]
pub struct MissingAttachment {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for MissingAttachment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("A required attachment is missing")
    }
}
impl std::error::Error for MissingAttachment {}

/// Error thrown when the user enters a string that is not recognized by a
/// ChoiceParameter-derived enum
#[derive(Default, Debug)]
pub struct InvalidChoice {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for InvalidChoice {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("You entered a non-existent choice")
    }
}
impl std::error::Error for InvalidChoice {}

/// Error thrown when the user enters a string that is not recognized as a boolean
#[derive(Default, Debug)]
pub struct InvalidBool {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for InvalidBool {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Expected a string like `yes` or `no` for the boolean parameter")
    }
}
impl std::error::Error for InvalidBool {}

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
