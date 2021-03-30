use super::*;

#[derive(Debug, Clone, PartialEq, Eq, Hash, Default)]
pub struct EmptyArgs;
impl std::fmt::Display for EmptyArgs {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Not enough arguments were given")
    }
}

impl std::error::Error for EmptyArgs {}

impl<'a> ParseConsuming<'a> for String {
    type Err = EmptyArgs;

    /// Pop a whitespace-separated word from the front of the arguments. Supports quotes and quote
    /// escaping.
    ///
    /// Leading whitespace will be trimmed; trailing whitespace is not consumed.
    ///
    /// ```rust
    /// # use poise::{Arguments, ParseConsuming as _};
    /// assert_eq!(
    ///     String::pop_from(&Arguments(r#""first arg" secondarg"#)).unwrap().1,
    ///     r#"first arg"#
    /// );
    /// assert_eq!(
    ///     String::pop_from(&Arguments(r#""arg \" with \" quotes \" inside""#)).unwrap().1,
    ///     r#"arg " with " quotes " inside"#
    /// );
    /// ```
    fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err> {
        // TODO: consider changing the behavior to parse quotes literally if they're in the middle
        // of the string:
        // - `"hello world"` => `hello world`
        // - `"hello "world"` => `"hello "world`
        // - `"hello" world"` => `hello`

        if args.0.is_empty() {
            return Err(EmptyArgs);
        }

        let mut output = String::new();
        let mut inside_string = false;
        let mut escaping = false;

        let mut chars = args.0.chars();
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

        Ok((Arguments(chars.as_str()), output))
    }
}

#[cfg(test)]
#[test]
fn test_pop_string() {
	// Test that trailing whitespace is not consumed
	assert_eq!(
		String::pop_from(&Arguments("AA BB")).unwrap().0,
		Arguments(" BB")
	);

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
		assert_eq!(String::pop_from(&Arguments(string)).unwrap().1, arg);
	}
}