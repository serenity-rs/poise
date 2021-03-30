use super::*;

#[derive(Clone, PartialEq, Eq, Debug, Default)]
pub struct KeyValueArgs {
    pairs: std::collections::HashMap<String, String>,
}

impl KeyValueArgs {
    pub fn get(&self, key: &str) -> Option<&str> {
        self.pairs.get(key).map(|x| x.as_str())
    }

    fn pop_single_key_value_pair<'a>(
        args: &Arguments<'a>,
    ) -> Option<(Arguments<'a>, (String, String))> {
        // TODO: share quote parsing machinery with ParseConsuming impl for String

        if args.0.is_empty() {
            return None;
        }

        let mut key = String::new();
        let mut inside_string = false;
        let mut escaping = false;

        let mut chars = args.0.trim_start().chars();
        while let Some(c) = chars.next() {
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
            } else {
                key.push(c);
            }
        }

        let args = Arguments(chars.as_str());
        // `args` used to contain "key=value ...", now it contains "value ...", so pop the value off
        let (args, value) = String::pop_from(&args).unwrap_or((args, String::new()));

        Some((args, (key, value)))
    }
}

impl<'a> ParseConsuming<'a> for KeyValueArgs {
    type Err = std::convert::Infallible;

    fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err> {
        let mut pairs = std::collections::HashMap::new();

        let mut args = args.clone();
        while let Some((new_args, (key, value))) = Self::pop_single_key_value_pair(&args) {
            args = new_args;
            pairs.insert(key, value);
        }

        Ok((args, Self { pairs }))
    }
}

#[cfg(test)]
#[test]
fn test_key_value_args() {
	for (string, pairs) in &[
		(
			r#"key1=value1 key2=value2"#,
			&[("key1", "value1"), ("key2", "value2")],
		),
		(
			r#""key 1"=value\ 1 key\ 2="value 2""#,
			&[("key 1", "value 1"), ("key 2", "value 2")],
		),
	] {
		assert_eq!(
			KeyValueArgs::pop_from(&Arguments(string)).unwrap().1,
			KeyValueArgs {
				pairs: pairs
					.iter()
					.map(|&(k, v)| (k.to_owned(), v.to_owned()))
					.collect(),
			}
		);
	}
}