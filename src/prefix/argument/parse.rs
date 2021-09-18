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
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        match $args.pop($ctx, $msg).await {
            Ok(($args, token)) => {
                let token: Option<$type> = Some(token);
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = Box::new(e),
        }
        let token: Option<$type> = None;
        $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
    };

    // Consume Option<T> lazy-first
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (#[lazy] Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        let token: Option<$type> = None;
        $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
        match $args.pop($ctx, $msg).await {
            Ok(($args, token)) => {
                let token: Option<$type> = Some(token);
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = Box::new(e),
        }
    };

    // Consume #[rest] Option<T> until the end of the input
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
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (Vec<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        let mut tokens = Vec::new();
        let mut token_rest_args = vec![$args.clone()];

        let mut running_args = $args.clone();
        loop {
            match running_args.pop::<$type>($ctx, $msg).await {
                Ok((popped_args, token)) => {
                    tokens.push(token);
                    token_rest_args.push(popped_args.clone());
                    running_args = popped_args;
                },
                Err(e) => {
                    $error = Box::new(e);
                    break;
                }

            }
        }

        // This will run at least once
        while let Some(token_rest_args) = token_rest_args.pop() {
            $crate::_parse_prefix!($ctx $msg token_rest_args => [ $error $($preamble)* tokens ] $($rest)* );
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

    // Consume #[flag] FLAGNAME
    ( $ctx:ident $msg:ident $args:ident => [ $error:ident $($preamble:tt)* ]
        (#[flag] $name:literal)
        $( $rest:tt )*
    ) => {
        if let Ok(($args, token)) = $args.pop::<String>($ctx, $msg).await {
            if token.eq_ignore_ascii_case($name) {
                $crate::_parse_prefix!($ctx $msg $args => [ $error $($preamble)* true ] $($rest)* );
            }
        }
        $error = concat!("Must use either `", $name, "` or nothing as a modifier").into();
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

/**
Macro for parsing an argument string into multiple parameter types.

An invocation of this macro is generated by the [`crate::command`] macro, so you usually don't need
to use this macro directly.

```rust
# #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
# use poise::serenity_prelude as serenity;
# let ctx = serenity::Context {
#     data: std::sync::Arc::new(serenity::RwLock::new(serenity::TypeMap::new())),
#     shard: ::serenity::client::bridge::gateway::ShardMessenger::new(
#         futures::channel::mpsc::unbounded().0,
#     ),
#     shard_id: Default::default(),
#     http: Default::default(),
#     cache: Default::default(),
# };
# let msg = serenity::CustomMessage::new().build();

assert_eq!(
    poise::parse_prefix_args!(
        &ctx, &msg,
        "one two three four" => (String), (Option<u32>), #[rest] (String)
    ).await?,
    (
        String::from("one"),
        None,
        String::from("two three four"),
    ),
);

assert_eq!(
    poise::parse_prefix_args!(
        &ctx, &msg,
        "1 2 3 4" => (String), (Option<u32>), #[rest] (String)
    ).await?,
    (
        String::from("1"),
        Some(2),
        String::from("3 4"),
    ),
);

# Ok(()) }
```
*/
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
