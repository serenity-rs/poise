//! A macro that generates backtracking-capable argument parsing code, given a list of parameter
//! types and attributes

#[doc(hidden)]
#[macro_export]
macro_rules! _parse_prefix {
    // All arguments have been consumed
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $( $name:ident )* ] ) => {
        if $args.is_empty() {
            return Ok(( $( $name, )* ));
        }
    };

    // Consume Option<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        (Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        // Try parse the next argument
        match $crate::pop_prefix_argument!($type, &$args, $attachment_index, $ctx, $msg).await {
            // On success, we get a new `$args` which contains only the rest of the args
            Ok(($args, $attachment_index, token)) => {
                // On success, store `Some(token)` for the parsed argument
                let token: Option<$type> = Some(token);
                // And parse the rest of the arguments
                $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ] $($rest)* );
                // If the code gets here, parsing the rest of the argument has failed
            },
            Err(e) => $error = e,
        }
        let token: Option<$type> = None;
        // Parse the next arguments without changing the current arg string, thereby skipping the
        // current param
        $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ] $($rest)* );
    };

    // Consume Option<T> lazy-first
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        (#[lazy] Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        let token: Option<$type> = None;
        $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ] $($rest)* );
        match $crate::pop_prefix_argument!($type, &$args, $attachment_index, $ctx, $msg).await {
            Ok(($args, $attachment_index, token)) => {
                let token: Option<$type> = Some(token);
                $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = e,
        }
    };

    // Consume #[rest] Option<T> until the end of the input
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        (#[rest] Option<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        if $args.trim_start().is_empty() {
            let token: Option<$type> = None;
            $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ]);
        } else {
            let input = $args.trim_start();
            match <$type as $crate::serenity_prelude::ArgumentConvert>::convert(
                $ctx, $msg.guild_id, Some($msg.channel_id), input
            ).await {
                Ok(token) => {
                    let $args = "";
                    let token = Some(token);
                    $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ]);
                },
                Err(e) => $error = (e.into(), Some(input.to_owned())),
            }
        }
    };

    // Consume Vec<T> greedy-first
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        (Vec<$type:ty $(,)?>)
        $( $rest:tt )*
    ) => {
        let mut tokens = Vec::new();
        let mut token_rest_args = vec![$args.clone()];

        let mut running_args = $args.clone();
        let mut attachment = $attachment_index;

        loop {
            match $crate::pop_prefix_argument!($type, &running_args, attachment, $ctx, $msg).await {
                Ok((popped_args, new_attachment, token)) => {
                    tokens.push(token);
                    token_rest_args.push(popped_args.clone());
                    running_args = popped_args;
                    attachment = new_attachment;
                },
                Err(e) => {
                    // No `$error = e`, because e.g. parsing into a Vec<Attachment> parameter with
                    // spare arguments would cause the error from the spare arguments to be the
                    // Attachment parse error ("missing attachment"), which is confusing
                    break;
                }

            }
        }

        // This will run at least once
        while let Some(token_rest_args) = token_rest_args.pop() {
            $crate::_parse_prefix!($ctx $msg token_rest_args attachment => [ $error $($preamble)* tokens ] $($rest)* );
            tokens.pop();
        }
    };

    // deliberately no `#[rest] &str` here because &str isn't supported anywhere else and this
    // inconsistency and also the further implementation work makes it not worth it.

    // Consume #[rest] T as the last argument
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        // question to my former self: why the $(poise::)* ?
        (#[rest] $(poise::)* $type:ty)
    ) => {
        let input = $args.trim_start();
        if input.is_empty() {
            $error = ($crate::TooFewArguments.into(), None);
        } else {
            match <$type as $crate::serenity_prelude::ArgumentConvert>::convert(
                $ctx, $msg.guild_id, Some($msg.channel_id), input
            ).await {
                Ok(token) => {
                    let $args = "";
                    $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ]);
                },
                Err(e) => $error = (e.into(), Some(input.to_owned())),
            }
        }
    };

    // Consume #[flag] FLAGNAME
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        (#[flag] $name:literal)
        $( $rest:tt )*
    ) => {
        match $crate::pop_prefix_argument!(String, &$args, $attachment_index, $ctx, $msg).await {
            Ok(($args, $attachment_index, token)) if token.eq_ignore_ascii_case($name) => {
                $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* true ] $($rest)* );
            },
            // only allow backtracking if the flag didn't match: it's confusing for the user if they
            // precisely set the flag but it's ignored
            _ => {
                $error = (concat!("Must use either `", $name, "` or nothing as a modifier").into(), None);
                $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* false ] $($rest)* );
            }
        }
    };

    // Consume T
    ( $ctx:ident $msg:ident $args:ident $attachment_index:ident => [ $error:ident $($preamble:tt)* ]
        ($type:ty)
        $( $rest:tt )*
    ) => {
        match $crate::pop_prefix_argument!($type, &$args, $attachment_index, $ctx, $msg).await {
            Ok(($args, $attachment_index, token)) => {
                $crate::_parse_prefix!($ctx $msg $args $attachment_index => [ $error $($preamble)* token ] $($rest)* );
            },
            Err(e) => $error = e,
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
#     data: std::sync::Arc::new(::serenity::prelude::RwLock::new(::serenity::prelude::TypeMap::new())),
#     shard: ::serenity::client::bridge::gateway::ShardMessenger::new(
#         futures::channel::mpsc::unbounded().0,
#     ),
#     shard_id: Default::default(),
#     http: std::sync::Arc::new(::serenity::http::Http::new("example")),
#     #[cfg(feature = "cache")]
#     cache: Default::default(),
# };
# let msg = serenity::CustomMessage::new().build();

assert_eq!(
    poise::parse_prefix_args!(
        &ctx, &msg,
        "one two three four", 0 => (String), (Option<u32>), #[rest] (String)
    ).await.unwrap(),
    (
        String::from("one"),
        None,
        String::from("two three four"),
    ),
);

assert_eq!(
    poise::parse_prefix_args!(
        &ctx, &msg,
        "1 2 3 4", 0 => (String), (Option<u32>), #[rest] (String)
    ).await.unwrap(),
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
    ($ctx:expr, $msg:expr, $args:expr, $attachment_index:expr => $(
        $( #[$attr:ident] )?
        ( $($type:tt)* )
    ),* $(,)? ) => {
        async {
            use $crate::PopArgument as _;

            let ctx = $ctx;
            let msg = $msg;
            let args = $args;
            let attachment_index = $attachment_index;

            let mut error: (Box<dyn std::error::Error + Send + Sync>, Option<String>)
                = (Box::new($crate::TooManyArguments) as _, None);

            $crate::_parse_prefix!(
                ctx msg args attachment_index => [error]
                $(
                    ($( #[$attr] )? $($type)*)
                )*
            );
            Err(error)
        }
    };
}

#[cfg(test)]
mod test {
    use super::*;

    #[tokio::test]
    async fn test_parse_args() {
        use crate::serenity_prelude as serenity;

        // Create dummy discord context; it will not be accessed in this test
        let ctx = serenity::Context {
            data: std::sync::Arc::new(
                tokio::sync::RwLock::new(::serenity::prelude::TypeMap::new()),
            ),
            shard: ::serenity::client::bridge::gateway::ShardMessenger::new(
                futures::channel::mpsc::unbounded().0,
            ),
            shard_id: Default::default(),
            http: std::sync::Arc::new(::serenity::http::Http::new("example")),
            #[cfg(feature = "cache")]
            cache: Default::default(),
        };
        let msg = serenity::CustomMessage::new().build();

        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "hello", 0 => (Option<String>), (String))
                .await
                .unwrap(),
            (None, "hello".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c", 0 => (Vec<String>), (String))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], "c".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c", 0 => (Vec<String>), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into(), "c".into()], vec![]),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b 8 c", 0 => (Vec<String>), (u32), (Vec<String>))
                .await
                .unwrap(),
            (vec!["a".into(), "b".into()], 8, vec!["c".into()]),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "yoo `that's cool` !", 0 => (String), (crate::CodeBlock), (String))
                .await
                .unwrap(),
            (
                "yoo".into(),
                crate::CodeBlock {
                    code: "that's cool".into(),
                    language: None
                },
                "!".into()
            ),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "hi", 0 => #[lazy] (Option<String>), (Option<String>))
                .await
                .unwrap(),
            (None, Some("hi".into())),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c", 0 => (String), #[rest] (String))
                .await
                .unwrap(),
            ("a".into(), "b c".into()),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "a b c", 0 => (String), #[rest] (String))
                .await
                .unwrap(),
            ("a".into(), "b c".into()),
        );
        assert!(
            parse_prefix_args!(&ctx, &msg, "hello", 0 => #[flag] ("hello"), #[rest] (String))
                .await
                .unwrap_err()
                .0
                .is::<crate::TooFewArguments>(),
        );
        assert_eq!(
            parse_prefix_args!(&ctx, &msg, "helloo", 0 => #[flag] ("hello"), #[rest] (String))
                .await
                .unwrap(),
            (false, "helloo".into())
        );
    }
}
