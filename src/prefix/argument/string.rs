use super::*;

impl<'a> PopArgument<'a> for String {
    type Err = std::convert::Infallible;

    /// Pop a whitespace-separated word from the front of the arguments. Supports quotes and quote
    /// escaping.
    ///
    /// Leading whitespace will be trimmed; trailing whitespace is not consumed.
    ///
    /// ```rust
    /// # use poise::PopArgument as _;
    /// assert_eq!(
    ///     String::pop_from(r#""first arg" secondarg"#).unwrap().1,
    ///     r#"first arg"#
    /// );
    /// assert_eq!(
    ///     String::pop_from(r#""arg \" with \" quotes \" inside""#).unwrap().1,
    ///     r#"arg " with " quotes " inside"#
    /// );
    /// ```
    fn pop_from(args: &'a str) -> Result<(&'a str, Self), Option<Self::Err>> {
        // TODO: consider changing the behavior to parse quotes literally if they're in the middle
        // of the string:
        // - `"hello world"` => `hello world`
        // - `"hello "world"` => `"hello "world`
        // - `"hello" world"` => `hello`

        if args.is_empty() {
            return Err(None);
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
}

#[cfg(test)]
#[test]
fn test_pop_string() {
    // Test that trailing whitespace is not consumed
    assert_eq!(String::pop_from("AA BB").unwrap().0, " BB");

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
        assert_eq!(String::pop_from(string).unwrap().1, arg);
    }
}
