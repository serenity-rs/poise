use super::*;

/// A command parameter type for key-value args
///
/// For example `key1=value1 key2="value2 with spaces"`.
///
/// ```rust
/// use poise::PopArgument;
///
/// let string = r#"key1=value key2="value with spaces" "key with spaces"="value with \"quotes\"""#;
/// let key_value_args = poise::KeyValueArgs::pop_from(string).unwrap().1;
///
/// let mut expected_result = std::collections::HashMap::new();
/// expected_result.insert("key1".into(), "value".into());
/// expected_result.insert("key2".into(), "value with spaces".into());
/// expected_result.insert("key with spaces".into(), r#"value with "quotes""#.into());
///
/// assert_eq!(key_value_args.0, expected_result);
/// ```
#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct KeyValueArgs(pub std::collections::HashMap<String, String>);

impl KeyValueArgs {
    /// Retrieve a single value by its key
    pub fn get(&self, key: &str) -> Option<&str> {
        self.0.get(key).map(|x| x.as_str())
    }

    fn pop_single_key_value_pair(args: &str) -> Option<(&str, (String, String))> {
        // TODO: share quote parsing machinery with PopArgumentAsync impl for String

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
        let (args, value) = String::pop_from(args).unwrap_or((args, String::new()));

        Some((args, (key, value)))
    }
}

impl<'a> PopArgument<'a> for KeyValueArgs {
    type Err = std::convert::Infallible;

    fn pop_from(mut args: &'a str) -> Result<(&'a str, Self), Option<Self::Err>> {
        let mut pairs = std::collections::HashMap::new();

        while let Some((remaining_args, (key, value))) = Self::pop_single_key_value_pair(args) {
            args = remaining_args;
            pairs.insert(key, value);
        }

        Ok((args, Self(pairs)))
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
        let (args, kv_args) = KeyValueArgs::pop_from(string).unwrap();

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
