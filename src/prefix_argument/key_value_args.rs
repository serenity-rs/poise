//! Parsing code for [`KeyValueArgs`], a prefix-specific command parameter type

use super::*;

/// A command parameter type for key-value args
///
/// For example `key1=value1 key2="value2 with spaces"`
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct KeyValueArgs(pub std::collections::HashMap<String, String>);

impl KeyValueArgs {
    /// Retrieve a single value by its key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|x| x.as_str())
    }

    /// Reads a single key value pair ("key=value") from the front of the arguments
    fn pop_single_key_value_pair(args: &str) -> Option<(&str, (String, String))> {
        // TODO: share quote parsing machinery with PopArgument impl for String

        if args.is_empty() {
            return None;
        }

        let mut key = String::new();
        let mut inside_string = false;
        let mut escaping = false;

        let mut chars = args.trim_start().chars();
        loop {
            let c = chars.next()?;
            if escaping {
                key.push(c);
                escaping = false;
            } else if !inside_string && c.is_whitespace() {
                return None;
            } else if c == '"' {
                inside_string = !inside_string;
            } else if c == '\\' {
                escaping = true;
            } else if !inside_string && c == '=' {
                break;
            } else if !inside_string && c.is_ascii_punctuation() {
                // If not enclosed in quotes, keys mustn't contain special characters.
                // Otherwise this command invocation: "?eval `0..=5`" is parsed as key-value args
                // with key "`0.." and value "5`". (This was a long-standing issue in rustbot)
                return None;
            } else {
                key.push(c);
            }
        }

        let args = chars.as_str();
        // `args` used to contain "key=value ...", now it contains "value ...", so pop the value off
        let (args, value) = super::pop_string(args).unwrap_or((args, String::new()));

        Some((args, (key, value)))
    }

    /// Reads as many key-value args as possible from the front of the string and produces a
    /// [`KeyValueArgs`] out of those
    fn pop_from(mut args: &str) -> (&str, Self) {
        let mut pairs = std::collections::HashMap::new();

        while let Some((remaining_args, (key, value))) = Self::pop_single_key_value_pair(args) {
            args = remaining_args;
            pairs.insert(key, value);
        }

        (args, Self(pairs))
    }
}

#[async_trait::async_trait]
impl<'a> PopArgument<'a> for KeyValueArgs {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        _: &serenity::Context,
        _: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let (a, b) = Self::pop_from(args);

        Ok((a, attachment_index, b))
    }
}

#[cfg(test)]
#[test]
fn test_key_value_args() {
    for &(string, pairs, remaining_args) in &[
        (
            r#"key1=value1 key2=value2"#,
            &[("key1", "value1"), ("key2", "value2")][..],
            "",
        ),
        (
            r#""key 1"=value\ 1 key\ 2="value 2""#,
            &[("key 1", "value 1"), ("key 2", "value 2")],
            "",
        ),
        (
            r#"key1"=value1 key2=value2"#,
            &[],
            r#"key1"=value1 key2=value2"#,
        ),
        (r#"dummyval"#, &[], "dummyval"),
        (r#"dummyval="#, &[("dummyval", "")], ""),
    ] {
        let (args, kv_args) = KeyValueArgs::pop_from(string);

        assert_eq!(
            kv_args.0,
            pairs
                .iter()
                .map(|&(k, v)| (k.to_owned(), v.to_owned()))
                .collect(),
        );
        assert_eq!(args, remaining_args);
    }
}
