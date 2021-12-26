//! Parsing code for [`CodeBlock`], a prefix-specific command parameter type

use super::*;

/// Error thrown when parsing a malformed [`CodeBlock`] ([`CodeBlock::pop_from`])
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct CodeBlockError;
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
#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CodeBlock {
    /// The text inside the code block
    pub code: String,
    /// In multiline code blocks, the language code, if present
    pub language: Option<String>,
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
        let code_block_end = code_block.find("```").ok_or(CodeBlockError)?;
        rest = &code_block[(code_block_end + 3)..];
        let mut code_block = &code_block[..code_block_end];

        // If a word is preceded directly by the three backticks and succeeded directly by a
        // newline, it's interpreted as the code block language
        let mut language = None;
        if let Some(first_newline) = code_block.find('\n') {
            if !code_block[..first_newline].contains(char::is_whitespace) {
                language = Some(&code_block[..first_newline]);
                code_block = &code_block[(first_newline + 1)..];
            }
        }

        // Discord strips empty lines from start and end, but only if their really empty (even
        // whitespace on the line cancels the stripping)
        let code_block = code_block.trim_start_matches('\n').trim_end_matches('\n');

        CodeBlock {
            code: code_block.to_owned(),
            language: language.map(|x| x.to_owned()),
        }
    } else if let Some(code_line) = args.strip_prefix('`') {
        let code_line_end = code_line.find('`').ok_or(CodeBlockError)?;
        rest = &code_line[(code_line_end + 1)..];
        let code_line = &code_line[..code_line_end];

        CodeBlock {
            code: code_line.to_owned(),
            language: None,
        }
    } else {
        return Err(CodeBlockError);
    };

    // Empty codeblocks like `` are not rendered as codeblocks by Discord
    if code_block.code.is_empty() {
        Err(CodeBlockError)
    } else {
        // discord likes to insert hair spaces at the end of code blocks sometimes for no reason
        code_block.code = code_block.code.trim_end_matches('\u{200a}').to_owned();

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
        _: &serenity::Context,
        _: &serenity::Message,
    ) -> Result<(&'a str, Self), (Box<dyn std::error::Error + Send + Sync>, Option<String>)> {
        pop_from(args).map_err(|e| (e.into(), None))
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
    ] {
        assert_eq!(
            pop_from(string).unwrap().1,
            CodeBlock {
                code: code.into(),
                language: language.map(|x| x.into())
            }
        );
    }

    assert_eq!(pop_from(""), Err(None));
    assert_eq!(pop_from("``"), Err(Some(MalformedCodeBlock)));
    assert_eq!(pop_from("``````"), Err(Some(MalformedCodeBlock)));
}
