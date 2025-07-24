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
