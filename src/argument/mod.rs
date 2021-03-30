mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod wrapper;
pub use wrapper::*;

// deliberately not copy with the intention to prevent accidental copies and confusion
#[derive(Clone, Debug, PartialEq, Eq, Hash)]
pub struct Arguments<'a>(pub &'a str);

impl<'a> Arguments<'a> {
    /// Parse a single argument and return the remaining arguments.
    ///
    /// Uses `crate::ParseConsuming` internally.
    ///
    /// ```rust
    /// # use poise::{Arguments, Wrapper};
    /// let args = Arguments("hello 123");
    ///
    /// let (args, hello) = args.pop::<String>().expect("a");
    /// assert_eq!(hello, "hello");
    ///
    /// dbg!(&args);
    /// let (args, number) = args.pop::<Wrapper<u32>>().expect("b");
    /// assert_eq!(number.0, 123);
    /// ```
    pub fn pop<T: ParseConsuming<'a>>(&self) -> Result<(Self, T), T::Err> {
        let (args, obj) = T::pop_from(self)?;
        Ok((Arguments(args.0.trim_start()), obj))
    }
}

/// Parse a value out of a string by popping off the front of the string.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's
/// sake and also because it keeps open the possibility of parsing whitespace.
pub trait ParseConsuming<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    fn pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err>;
}

#[derive(Debug)]
pub struct TooManyArguments;

impl std::fmt::Display for TooManyArguments {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("Too many arguments were passed")
    }
}

impl std::error::Error for TooManyArguments {}

/// The error type returned from [parse_args!]. It contains a `Box<dyn Error>`
#[derive(Debug)]
pub struct ArgumentParseError(pub Box<dyn std::error::Error + Send + Sync>);

impl std::fmt::Display for ArgumentParseError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "Failed to parse argument: {}", self.0)
    }
}

impl std::error::Error for ArgumentParseError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        Some(&*self.0)
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parse {
    // All arguments have been consumed
    ( $args:ident => [ $error:ident $( $name:ident )* ] ) => {
        if $args.0.is_empty() {
            return Ok(( $( $name ),* ));
        }
    };

    // Consume Option<T> greedy-first
    ( $args:ident => [ $($preamble:tt)* ]
        (Option<$type:ty>)
        $( $rest:tt )*
    ) => {
        if let Ok(($args, token)) = $args.pop() {
            let token: Option<$type> = Some(token);
            $crate::_parse!($args => [ $($preamble)* token ] $($rest)* );
        }
        let token: Option<$type> = None;
        $crate::_parse!($args => [ $($preamble)* token ] $($rest)* );
    };

    // Consume Option<T> lazy-first
    ( $args:ident => [ $($preamble:tt)* ]
        (lazy Option<$type:ty>)
        $( $rest:tt )*
    ) => {
        let token: Option<$type> = None;
        $crate::_parse!($args => [ $($preamble)* token ] $($rest)* );
        if let Ok(($args, token)) = $args.pop() {
            let token: Option<$type> = Some(token);
            $crate::_parse!($args => [ $($preamble)* token ] $($rest)* );
        }
    };

    // Consume Vec<T> greedy-first
    ( $args:ident => [ $($preamble:tt)* ]
        (Vec<$type:ty>)
        $( $rest:tt )*
    ) => {
        let mut tokens = Vec::new();
        let mut token_rest_args = Vec::new();
        token_rest_args.push($args.clone());

        let mut running_args = $args.clone();
        while let Ok((popped_args, token)) = running_args.pop::<$type>() {
            tokens.push(token);
            token_rest_args.push(popped_args.clone());
            running_args = popped_args;
        }

        // This will run at least once
        while let Some(token_rest_args) = token_rest_args.pop() {
            $crate::_parse!(token_rest_args => [ $($preamble)* tokens ] $($rest)* );
            tokens.pop();
        }
    };

    // Consume T
    ( $args:ident => [ $error:ident $( $preamble:ident )* ]
        ($type:ty)
        $( $rest:tt )*
    ) => {
        match $args.pop::<$type>() {
            Ok(($args, token)) => {
                $crate::_parse!($args => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = Box::new(e),
        }
    };
}

#[macro_export]
macro_rules! parse_args {
    ($args:expr => $(
        $( #[$attr:ident] )?
        ( $($type:tt)* )
    ),* $(,)? ) => {
        move || -> Result<( $($($type)*),* ), $crate::ArgumentParseError> {
            let args = $crate::Arguments($args);
            #[allow(unused_mut)] // can happen when few args are requested
            let mut error = Box::new($crate::TooManyArguments) as Box<dyn std::error::Error + Send + Sync>;
            $crate::_parse!(
                args => [error]
                $(
                    ($($attr)? $($type)*)
                )*
            );
            Err($crate::ArgumentParseError(error))
        }()
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[test]
    fn test_parse_args() {
        assert_eq!(
            parse_args!("hello" => (Option<String>), (String)).unwrap(),
            (None, "hello".into()),
        );
        assert_eq!(
            parse_args!("a b c" => (Vec<String>), (String)).unwrap(),
            (vec!["a".into(), "b".into()], "c".into()),
        );
        assert_eq!(
            parse_args!("a b c" => (Vec<String>), (Vec<String>)).unwrap(),
            (vec!["a".into(), "b".into(), "c".into()], vec![]),
        );
        assert_eq!(
            parse_args!("a b 8 c" => (Vec<String>), (Wrapper<u32>), (Vec<String>)).unwrap(),
            (vec!["a".into(), "b".into()], Wrapper(8), vec!["c".into()]),
        );
        assert_eq!(
            parse_args!("yoo `that's cool` !" => (String), (CodeBlock), (String)).unwrap(),
            (
                "yoo".into(),
                CodeBlock {
                    code: "that's cool".into(),
                    language: None
                },
                "!".into()
            ),
        );
        assert_eq!(
            parse_args!("hi" => #[lazy] (Option<String>), (Option<String>)).unwrap(),
            (None, Some("hi".into())),
        );
    }
}
