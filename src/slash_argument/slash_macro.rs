//! Infrastructure to parse received slash command arguments into Rust types.

#[allow(unused_imports)] // import is required if serenity simdjson feature is enabled
use crate::serenity::json::*;
#[allow(unused_imports)] // required for introdoc-links in doc comments
use crate::serenity_prelude as serenity;

/// Possible errors when parsing slash command arguments
#[derive(Debug)]
pub enum SlashArgError {
    /// Expected a certain argument type at a certain position in the unstructured list of
    /// arguments, but found something else.
    ///
    /// Most often the result of the bot not having registered the command in Discord, so Discord
    /// stores an outdated version of the command and its parameters.
    #[non_exhaustive]
    CommandStructureMismatch {
        /// A short string describing what specifically is wrong/unexpected
        description: &'static str,
    },
    /// A string parameter was found, but it could not be parsed into the target type.
    #[non_exhaustive]
    Parse {
        /// Error that occurred while parsing the string into the target type
        error: Box<dyn std::error::Error + Send + Sync>,
        /// Original input string
        input: String,
    },
    /// The argument passed by the user is invalid in this context. E.g. a Member parameter in DMs
    #[non_exhaustive]
    Invalid(
        /// Human readable description of the error
        &'static str,
    ),
    /// HTTP error occurred while retrieving the model type from Discord
    Http(serenity::Error),
    #[doc(hidden)]
    __NonExhaustive,
}

/// Support functions for macro which can't create #[non_exhaustive] enum variants
#[doc(hidden)]
impl SlashArgError {
    pub fn new_command_structure_mismatch(description: &'static str) -> Self {
        Self::CommandStructureMismatch { description }
    }

    pub fn to_framework_error<U, E>(
        self,
        ctx: crate::ApplicationContext<'_, U, E>,
    ) -> crate::FrameworkError<'_, U, E> {
        match self {
            Self::CommandStructureMismatch { description } => {
                crate::FrameworkError::CommandStructureMismatch { ctx, description }
            }
            Self::Parse { error, input } => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error,
                input: Some(input),
            },
            Self::Invalid(description) => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error: description.into(),
                input: None,
            },
            Self::Http(error) => crate::FrameworkError::ArgumentParse {
                ctx: ctx.into(),
                error: error.into(),
                input: None,
            },
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

impl std::fmt::Display for SlashArgError {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::CommandStructureMismatch { description } => {
                write!(
                    f,
                    "Bot author did not register their commands correctly ({description})",
                )
            }
            Self::Parse { error, input } => {
                write!(f, "Failed to parse `{input}` as argument: {error}")
            }
            Self::Invalid(description) => {
                write!(f, "You can't use this parameter here: {description}",)
            }
            Self::Http(error) => {
                write!(
                    f,
                    "Error occurred while retrieving data from Discord: {error}",
                )
            }
            Self::__NonExhaustive => unreachable!(),
        }
    }
}
impl std::error::Error for SlashArgError {
    fn cause(&self) -> Option<&dyn std::error::Error> {
        match self {
            Self::Http(error) => Some(error),
            Self::Parse { error, input: _ } => Some(&**error),
            Self::Invalid { .. } | Self::CommandStructureMismatch { .. } => None,
            Self::__NonExhaustive => unreachable!(),
        }
    }
}

#[doc(hidden)]
#[macro_export]
macro_rules! _parse_slash {
    // Extract #[choices(...)] (no Option supported ;-;)
    ($ctx:ident, $interaction:ident, $args:ident => $name:literal: INLINE_CHOICE $type:ty [$($index:literal: $value:literal),*]) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            let $crate::serenity_prelude::ResolvedValue::Integer(index) = arg.value else {
                return Err($crate::SlashArgError::new_command_structure_mismatch("expected integer, as the index for an inline choice parameter"));
            };
            match index {
                $( $index => $value, )*
                _ => return Err($crate::SlashArgError::new_command_structure_mismatch("out of range index for inline choice parameter")),
            }
        } else {
            return Err($crate::SlashArgError::new_command_structure_mismatch("a required argument is missing"));
        }
    };

    // Extract Option<T>
    ($ctx:ident, $interaction:ident, $args:ident => $name:literal: Option<$type:ty $(,)*>) => {
        if let Some(arg) = $args.iter().find(|arg| arg.name == $name) {
            Some($crate::extract_slash_argument!($type, $ctx, $interaction, &arg.value)
                .await?)
        } else {
            None
        }
    };

    // Extract Vec<T> (delegating to Option<T> because slash commands don't support variadic
    // arguments right now)
    ($ctx:ident, $interaction:ident, $args:ident => $name:literal: Vec<$type:ty $(,)*>) => {
        match $crate::_parse_slash!($ctx, $interaction, $args => $name: Option<$type>) {
            Some(value) => vec![value],
            None => vec![],
        }
    };

    // Extract #[flag]
    ($ctx:ident, $interaction:ident, $args:ident => $name:literal: FLAG) => {
        $crate::_parse_slash!($ctx, $interaction, $args => $name: Option<bool>)
            .unwrap_or(false)
    };

    // Extract T
    ($ctx:ident, $interaction:ident, $args:ident => $name:literal: $($type:tt)*) => {
        $crate::_parse_slash!($ctx, $interaction, $args => $name: Option<$($type)*>)
            .ok_or($crate::SlashArgError::new_command_structure_mismatch("a required argument is missing"))?
    };
}

/**
Macro for extracting and parsing slash command arguments out of an array of
[`serenity::CommandDataOption`].

An invocation of this macro is generated by `crate::command`, so you usually don't need this macro
directly.

```rust,no_run
# #[tokio::main] async fn main() -> Result<(), Box<dyn std::error::Error>> {
# use poise::serenity_prelude as serenity;
let ctx: serenity::Context = todo!();
let interaction: serenity::CommandInteraction = todo!();
let args: &[serenity::ResolvedOption] = todo!();

let (arg1, arg2) = poise::parse_slash_args!(
    &ctx, &interaction,
    args => ("arg1": String), ("arg2": Option<u32>)
).await?;

# Ok(()) }
```
*/
#[macro_export]
macro_rules! parse_slash_args {
    ($ctx:expr, $interaction:expr, $args:expr => $(
        ( $name:literal: $($type:tt)* )
    ),* $(,)? ) => {
        async /* not move! */ {
            use $crate::SlashArgumentHack;

            // ctx here is a serenity::Context, so it doesn't already contain interaction!
            let (ctx, interaction, args) = ($ctx, $interaction, $args);

            Ok::<_, $crate::SlashArgError>(( $(
                $crate::_parse_slash!( ctx, interaction, args => $name: $($type)* ),
            )* ))
        }
    };
}
