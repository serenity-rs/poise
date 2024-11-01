//! Parsing code for [`CodeBlock`], a prefix-specific command parameter type

use super::*;
use trim_in_place::TrimInPlace;

/// Error thrown when parsing a malformed [`CodeBlock`] ([`CodeBlock::pop_from`])
#[derive(Default, Debug, Clone)]
pub struct CodeBlockError {
    #[doc(hidden)]
    pub __non_exhaustive: (),
}
impl std::fmt::Display for CodeBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("couldn't find a valid code block")
    }
}
impl std::error::Error for CodeBlockError {}

/// A command parameter type for Discord code blocks
///
/// ```text
/// `code here`
/// ```
///
/// or
///
/// ```text
/// ``​`language
/// code here
/// ``​`
/// ```
///
/// Can be used as a command parameter. For more information, see [`Self::pop_from`].
#[derive(Default, Debug, PartialEq, Eq, Clone, Hash)]
pub struct CodeBlock {
    /// The text inside the code block
    pub code: String,
    /// In multiline code blocks, the language code, if present
    pub language: Option<String>,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl std::fmt::Display for CodeBlock {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "```{}\n{}\n```",
            self.language.as_deref().unwrap_or(""),
            &self.code
        )
    }
}

/// Reads a [`CodeBlock`] from the front of the string and returns the remaining string.
fn pop_from(args: &str) -> Result<(&str, CodeBlock), CodeBlockError> {
    let args = args.trim_start();

    let rest;
    let mut code_block = if let Some(code_block) = args.strip_prefix("```") {
        let code_block_end = code_block.find("```").ok_or(CodeBlockError::default())?;
        rest = &code_block[(code_block_end + 3)..];
        let mut code_block = &code_block[..code_block_end];

        // If a string is preceded directly by the three backticks and succeeded directly by a
        // newline, it's interpreted as the code block language
        let mut language = None;
        if let Some(first_newline) = code_block.find('\n') {
            // Experimentation revealed language idents may only consist of [A-Za-z0-9+-._#]
            let is_valid = code_block[..first_newline]
                .chars()
                .all(|c| c.is_ascii_alphanumeric() || "+-._#".contains(c));
            if is_valid {
                language = Some(&code_block[..first_newline]);
                code_block = &code_block[(first_newline + 1)..];
            }
        }

        // Discord strips empty lines from start and end, but only if their really empty (even
        // whitespace on the line cancels the stripping)
        let code_block = code_block.trim_matches('\n');

        CodeBlock {
            code: code_block.to_owned(),
            language: language.map(|x| x.to_owned()),
            __non_exhaustive: (),
        }
    } else if let Some(code_line) = args.strip_prefix('`') {
        let code_line_end = code_line.find('`').ok_or(CodeBlockError::default())?;
        rest = &code_line[(code_line_end + 1)..];
        let code_line = &code_line[..code_line_end];

        CodeBlock {
            code: code_line.to_owned(),
            language: None,
            __non_exhaustive: (),
        }
    } else {
        return Err(CodeBlockError::default());
    };

    // Empty codeblocks like `` are not rendered as codeblocks by Discord
    if code_block.code.is_empty() {
        Err(CodeBlockError::default())
    } else {
        // discord likes to insert hair spaces at the end of code blocks sometimes for no reason
        code_block.code.trim_end_matches_in_place('\u{200a}');
        Ok((rest, code_block))
    }
}

#[async_trait::async_trait]
impl<'a> PopArgument<'a> for CodeBlock {
    /// Parse a single-line or multi-line code block. The output of `Self::code` should mirror what
    /// the official Discord client renders, and the output of `Self::language` should mirror the
    /// official Discord client's syntax highlighting, if existent.
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        _: &serenity::Context,
        _: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let (a, b) = pop_from(args).map_err(|e| (e.into(), None))?;

        Ok((a, attachment_index, b))
    }
}

#[cfg(test)]
#[test]
fn test_pop_code_block() {
    for &(string, code, language) in &[
        ("`hello world`", "hello world", None),
        ("` `", " ", None),
        ("``` hi ```", " hi ", None),
        ("```rust```", "rust", None),
        ("```rust\nhi```", "hi", Some("rust")),
        ("```rust  hi```", "rust  hi", None),
        ("```rust\n\n\n\n\nhi\n\n\n\n```", "hi", Some("rust")),
        ("```+__....-.+.++\nhi\n```", "hi", Some("+__....-.+.++")),
        ("```+__.:...-.+.++\nhi\n```", "+__.:...-.+.++\nhi", None),
        // https://discord.com/channels/273534239310479360/592856094527848449/997519220369797131
        (
            "```#![feature(never_type)]\nfn uwu(_: &!) {}\n```",
            "#![feature(never_type)]\nfn uwu(_: &!) {}",
            None,
        ),
        ("```c#\nusing System;\n```", "using System;", Some("c#")),
    ] {
        assert_eq!(
            pop_from(string).unwrap().1,
            CodeBlock {
                code: code.into(),
                language: language.map(|x| x.into()),
                __non_exhaustive: (),
            }
        );
    }

    assert!(pop_from("").is_err());
    assert!(pop_from("''").is_err());
    assert!(pop_from("``").is_err());
    assert!(pop_from("``````").is_err());
}
