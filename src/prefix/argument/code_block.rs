use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum CodeBlockError {
    Missing,
    Malformed,
}

impl std::fmt::Display for CodeBlockError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::Missing => f.write_str("Missing code block"),
            Self::Malformed => f.write_str("Malformed code block"),
        }
    }
}

impl std::error::Error for CodeBlockError {}

#[derive(Debug, PartialEq, Eq, Clone, Hash)]
pub struct CodeBlock {
    pub code: String,
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

impl<'a> ParseConsumingSync<'a> for CodeBlock {
    type Err = CodeBlockError;

    /// Parse a single-line or multi-line code block. The output of `Self::code` should mirror what
    /// the official Discord client renders, and the output of `Self::language` should mirror the
    /// official Discord client's syntax highlighting, if existent.
    ///
    /// ```rust
    /// # use poise::{CodeBlock, ArgString, ParseConsuming as _};
    /// assert_eq!(
    ///     ArgString("`hello world`").sync_pop::<CodeBlock>().unwrap().1,
    ///     CodeBlock { code: "hello world".into(), language: None },
    /// );
    /// assert_eq!(
    ///     ArgString("```rust\nprintln!(\"Hello world!\");\n```").sync_pop::<CodeBlock>().unwrap().1,
    ///     CodeBlock { code: "println!(\"Hello world!\");".into(), language: Some("rust".into()) },
    /// );
    /// ```
    fn sync_pop_from(args: &ArgString<'a>) -> Result<(ArgString<'a>, Self), Self::Err> {
        let rest;
        let mut code_block = if let Some(code_block) = args.0.strip_prefix("```") {
            let code_block_end = code_block.find("```").ok_or(CodeBlockError::Malformed)?;
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

            Self {
                code: code_block.to_owned(),
                language: language.map(|x| x.to_owned()),
            }
        } else if let Some(code_line) = args.0.strip_prefix("`") {
            let code_line_end = code_line.find('`').ok_or(CodeBlockError::Malformed)?;
            rest = &code_line[(code_line_end + 1)..];
            let code_line = &code_line[..code_line_end];

            Self {
                code: code_line.to_owned(),
                language: None,
            }
        } else {
            return Err(CodeBlockError::Missing);
        };

        // Empty codeblocks like `` are not rendered as codeblocks by Discord
        if code_block.code.is_empty() {
            Err(CodeBlockError::Malformed)
        } else {
            // discord likes to insert hair spaces at the end of code blocks sometimes for no reason
            code_block.code = code_block.code.trim_end_matches('\u{200a}').to_owned();

            Ok((ArgString(rest), code_block))
        }
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
            CodeBlock::sync_pop_from(&ArgString(string)).unwrap().1,
            CodeBlock {
                code: code.into(),
                language: language.map(|x| x.into())
            }
        );
    }

    assert_eq!(
        CodeBlock::sync_pop_from(&ArgString("``")),
        Err(CodeBlockError::Malformed)
    );
    assert_eq!(
        CodeBlock::sync_pop_from(&ArgString("``````")),
        Err(CodeBlockError::Malformed)
    );
}
