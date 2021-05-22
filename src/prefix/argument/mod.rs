mod code_block;
pub use code_block::*;

mod key_value_args;
pub use key_value_args::*;

mod string;
pub use string::*;

mod wrapper;
pub use wrapper::*;

use crate::serenity_prelude as serenity;

// deliberately not copy with the intention to prevent accidental copies and confusion
#[derive(Clone, PartialEq, Eq, Debug)]
pub struct ArgString<'a>(pub &'a str);

impl<'a> ArgString<'a> {
    /// Parse a single argument and return the remaining arguments.
    ///
    /// Uses `crate::ParseConsuming` internally.
    ///
    /// ```rust
    /// # use poise::{ArgString, CodeBlock};
    /// let args = ArgString("hello `foo bar`");
    ///
    /// let (args, hello) = args.sync_pop::<String>().unwrap();
    /// assert_eq!(hello, "hello");
    ///
    /// let (args, block) = args.sync_pop::<CodeBlock>().unwrap();
    /// assert_eq!(block.code, "foo bar");
    /// ```
    pub async fn pop<T: ParseConsuming<'a>>(
        &self,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(ArgString<'a>, T), <T as ParseConsuming<'a>>::Err> {
        let (args, obj) = T::pop_from(ctx, msg, self).await?;
        Ok((ArgString(args.0.trim_start()), obj))
    }

    /// Like [`Self::pop`] but synchronous.
    pub fn sync_pop<T: ParseConsumingSync<'a>>(
        &self,
    ) -> Result<(ArgString<'a>, T), <T as ParseConsuming<'a>>::Err> {
        let (args, obj) = T::sync_pop_from(self)?;
        Ok((ArgString(args.0.trim_start()), obj))
    }
}

/// Superset of [`ParseConsuming`] without Discord context available and no async support.
///
/// Similar in spirit to [`std::str::FromStr`].
pub trait ParseConsumingSync<'a>: Sized {
    /// This error type should implement [`std::error::Error`] most of the time
    type Err;

    fn sync_pop_from(args: &ArgString<'a>) -> Result<(ArgString<'a>, Self), Self::Err>;
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
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err>;
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
        args: &ArgString<'a>,
    ) -> Result<(ArgString<'a>, Self), Self::Err> {
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

/// The error type returned from [parse_prefix_args!]. It contains a `Box<dyn Error>`
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
macro_rules! _parse_prefix {
    // All arguments have been consumed
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $( $name:ident )* ] ) => {
        if $args.0.is_empty() {
            return Ok(( $( $name, )* ));
        }
    };

    // Consume Option<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        if let Ok(($args, token)) = $args.pop($ctx, $msg).await {
            let token: Option<$type> = Some(token);
            $crate::_parse_prefix!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        }
        let token: Option<$type> = None;
        $crate::_parse_prefix!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
    };

    // Consume Option<T> lazy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (#[lazy] Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        let token: Option<$type> = None;
        $crate::_parse_prefix!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        if let Ok(($args, token)) = $args.pop($ctx, $msg).await {
            let token: Option<$type> = Some(token);
            $crate::_parse_prefix!($ctx $msg $args => [ $($preamble)* token ] $($rest)* );
        }
    };

    // Consume Option<T> until the end of the input
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (#[rest] Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        if $args.0.trim_start().is_empty() {
            let token: Option<$type> = None;
            $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ]);
        } else {
            match <$type as $crate::serenity_prelude::ArgumentConvert>::convert(
                $ctx, $msg.guild_id, Some($msg.channel_id), $args.0.trim_start()
            ).await {
                Ok(token) => {
                    let $args = $crate::ArgString("");
                    let token = Some(token);
                    $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ]);
                },
                Err(e) => $error = Box::new(e),
            }
        }
    };

    // Consume Vec<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident => [ $($preamble:tt)* ]
        (Vec<$type:ty $(,)?>)
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
            $crate::_parse_prefix!($ctx $msg token_rest_args => [ $($preamble)* tokens ] $($rest)* );
            tokens.pop();
        }
    };

    // deliberately no `#[rest] &str` here because &str isn't supported anywhere else and this
    // inconsistency and also the further implementation work makes it not worth it.

    // Consume #[rest] T as the last argument
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        // question to my former self: why the $(poise::)* ?
        (#[rest] $(poise::)* $type:ty)
    ) => {
        match <$type as $crate::serenity_prelude::ArgumentConvert>::convert(
            $ctx, $msg.guild_id, Some($msg.channel_id), $args.0.trim_start()
        ).await {
            Ok(token) => {
                let $args = $crate::ArgString("");
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ]);
            },
            Err(e) => $error = Box::new(e),
        }
    };

    // Consume #[flag] FLAGNAME as the last argument
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (#[flag] $name:literal)
        $( $rest:tt )*
    ) => {
        if let Ok(($args, token)) = $args.pop::<String>($ctx, $msg).await {
            if token.eq_ignore_ascii_case($name) {
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* true ] $($rest)* );
            }
        }
        $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* false ] $($rest)* );
    };

    // Consume T
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        ($type:ty)
        $( $rest:tt )*
    ) => {
        match $args.pop::<$type>($ctx, $msg).await {
            Ok(($args, token)) => {
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = Box::new(e),
        }
    };

    // ( $($t:tt)* ) => {
    //     compile_error!( stringify!($($t)*) );
    // };
}

#[macro_export]
macro_rules! parse_prefix_args {
    ($ctx:expr, $msg:expr, $args:expr => $(
        $( #[$attr:ident] )?
        ( $($type:tt)* )
    ),* $(,)? ) => {
        async {
            let ctx = $ctx;
            let msg = $msg;
            let args = $crate::ArgString($args);

            #[allow(unused_mut)] // can happen when few args are requested
            let mut error = Box::new($crate::TooManyArguments) as Box<dyn std::error::Error + Send + Sync>;

            $crate::_parse_prefix!(
                ctx msg args => [error]
                $(
                    ($( #[$attr] )? $($type)*)
                )*
            );
            Err($crate::ArgumentParseError(error))
        }
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
            parse_prefix_args!(&ctx, &msg, "hello" => (Option<String>), (String))
                .await
                .unwrap(),
            (None, "hello".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c" => (Vec<String>), (String))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], "c".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c" => (Vec<String>), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into(), "c".into()], vec![]),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b 8 c" => (Vec<String>), (Wrapper<u32>), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], Wrapper(8), vec!["c".into()]),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "yoo `that's cool` !" => (String), (CodeBlock), (String))
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
            parse_prefix_args!(&ctx, &msg, "hi" => #[lazy] (Option<String>), (Option<String>))
                .await
                .unwrap(),
            (None, Some("hi".into())),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c" => (String), #[rest] (Wrapper<String>))
                .await
                .unwrap(),
            ("a".into(), Wrapper("b c".into())),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c" => (String), #[rest] (String))
                .await
                .unwrap(),
            ("a".into(), "b c".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "hello" => #[flag] ("hello"), #[rest] (String))
                .await
                .unwrap(),
            (true, "".into())
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "helloo" => #[flag] ("hello"), #[rest] (String))
                .await
                .unwrap(),
            (false, "helloo".into())
        );
    }
}
