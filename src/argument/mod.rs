mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod wrapper;
pub use wrapper::*;

use crate::serenity;

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
    pub async fn pop<T: ParseConsuming<'a>>(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(Arguments<'a>, T), <T as ParseConsuming<'a>>::Err> {
        let (args, obj) = T::pop_from(ctx, msg, self).await?;
        Ok((Arguments(args.0.trim_start()), obj))
    }
}

/// Superset of [`DiscordParseConsuming`] without Discord context available and no async support.
///
/// Similar in spirit to [`std::str::FromStr`].
pub trait ParseConsumingSync<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    fn sync_pop_from(args: &Arguments<'a>) -> Result<(Arguments<'a>, Self), Self::Err>;
}

/// Parse a value out of a string by popping off the front of the string. Discord message context
/// is available for parsing, and IO may be done as part of the parsing.
///
/// Implementors should assume that a string never starts with whitespace, and fail to parse if it
/// does. This is for consistency's
/// sake and also because it keeps open the possibility of parsing whitespace.
#[async_trait::async_trait]
pub trait ParseConsuming<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    async fn pop_from(
        ctx: &serenity::Context,
        msg: &serenity::Message,
        args: &Arguments<'a>,
    ) -> Result<(Arguments<'a>, Self), Self::Err>;
}

#[async_trait::async_trait]
impl<'a, T> ParseConsuming<'a> for T
where
    T: ParseConsumingSync<'a>,
{
    type Err = <Self as ParseConsumingSync<'a>>::Err;

    async fn pop_from(
        _: &serenity::Context,
        _: &serenity::Message,
        args: &Arguments<'a>,
    ) -> Result<(Arguments<'a>, Self), Self::Err> {
        <Self as ParseConsumingSync>::sync_pop_from(args)
    }
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
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $( $name:ident )* ] ) => {
        if $args.0.is_empty() {
            return Ok(( $( $name ),* ));
        }
    };

    // Consume Option<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (Option<$type:ty>)
        $( $rest:tt )*
    ) => {
        if let Ok(($args, token)) = $args.pop($ctx, $msg).await {
            let token: Option<$type> = Some(token);
            $crate::_parse!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        }
        let token: Option<$type> = None;
        $crate::_parse!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
    };

    // Consume Option<T> lazy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (#[lazy] Option<$type:ty>)
        $( $rest:tt )*
    ) => {
        let token: Option<$type> = None;
        $crate::_parse!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        if let Ok(($args, token)) = $args.pop($ctx, $msg).await {
            let token: Option<$type> = Some(token);
            $crate::_parse!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        }
    };

    // Consume Vec<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (Vec<$type:ty>)
        $( $rest:tt )*
    ) => {
        let mut tokens = Vec::new();
        let mut token_rest_args = vec![$args.clone()];

        let mut running_args = $args.clone();
        while let Ok((popped_args, token)) = running_args.pop::<$type>($ctx, $msg).await {
            tokens.push(token);
            token_rest_args.push(popped_args.clone());
            running_args = popped_args;
        }

        // This will run at least once
        while let Some(token_rest_args) = token_rest_args.pop() {
            $crate::_parse!($ctx $msg token_rest_args => [ $($preamble)* tokens ] $($rest)* );
            tokens.pop();
        }
    };

    // Consume #[rest] &str as the last argument
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (#[rest] &str)
    ) => {
        let token = $args.0;
        let $args = $crate::Arguments("");
        $crate::_parse!($ctx $msg $args => [ $($preamble)* token ] );
    };

    // Consume #[rest] Wrapper<T> as the last argument
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (#[rest] $(poise::)* Wrapper<$type:ty>)
    ) => {
        match $args.0.trim_start().parse::<$type>() {
            Ok(token) => {
                let $args = $crate::Arguments("");
                let token = $crate::Wrapper(token);
                $crate::_parse!($ctx $msg $args => [ $error $($preamble)* token ]);
            },
            Err(e) => $error = Box::new(e),
        }
    };

    // Consume T
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        ($type:ty)
        $( $rest:tt )*
    ) => {
        match $args.pop::<$type>($ctx, $msg).await {
            Ok(($args, token)) => {
                $crate::_parse!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = Box::new(e),
        }
    };

    // ( $($t:tt)* ) => {
    //     compile_error!( stringify!($($t)*) );
    // }
}

#[macro_export]
macro_rules! parse_args {
    ($ctx:expr, $msg:expr, $args:expr => $(
        $( #[$attr:ident] )?
        ( $($type:tt)* )
    ),* $(,)? ) => {
        (|| async {
            let ctx = $ctx;
            let msg = $msg;
            let args = $crate::Arguments($args);

            #[allow(unused_mut)] // can happen when few args are requested
            let mut error = Box::new($crate::TooManyArguments) as Box<dyn std::error::Error + Send + Sync>;

            $crate::_parse!(
                ctx msg args => [error]
                $(
                    ($( #[$attr] )? $($type)*)
                )*
            );
            Err($crate::ArgumentParseError(error))
        })()
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_parse_args() {
        // Create dummy discord context; it will not be accessed in this test
        let ctx = serenity::Context {
            data: std::sync::Arc::new(serenity::RwLock::new(serenity::TypeMap::new())),
            shard: ::serenity::client::bridge::gateway::ShardMessenger::new(
                futures::channel::mpsc::unbounded().0,
            ),
            shard_id: Default::default(),
            http: Default::default(),
            cache: Default::default(),
        };
        let msg = serenity::CustomMessage::new().build();

        assert_eq!(
            parse_args!(&ctx, &msg, "hello" => (Option<String>), (String))
                .await
                .unwrap(),
            (None, "hello".into()),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "a b c" => (Vec<String>), (String))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], "c".into()),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "a b c" => (Vec<String>), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into(), "c".into()], vec![]),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "a b 8 c" => (Vec<String>), (Wrapper<u32>), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], Wrapper(8), vec!["c".into()]),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "yoo `that's cool` !" => (String), (CodeBlock), (String))
                .await
                .unwrap(),
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
            parse_args!(&ctx, &msg, "hi" => #[lazy] (Option<String>), (Option<String>))
                .await
                .unwrap(),
            (None, Some("hi".into())),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "a b c" => (String), #[rest] (Wrapper<String>))
                .await
                .unwrap(),
            ("a".into(), Wrapper("b c".into())),
        );
        assert_eq!(
            parse_args!(&ctx, &msg, "a b c" => (String), #[rest] (&str))
                .await
                .unwrap(),
            ("a".into(), "b c"),
        );
    }
}
