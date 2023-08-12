//! Contains the [`ChoiceParameter`] trait and the blanket [`crate::SlashArgument`] and
//! [`crate::PopArgument`] impl

use crate::serenity_prelude as serenity;

/// This trait is implemented by [`crate::macros::ChoiceParameter`]. See its docs for more
/// information
pub trait ChoiceParameter: Sized {
    /// Returns all possible choices for this parameter, in the order they will appear in Discord.
    fn list() -> Vec<crate::CommandParameterChoice>;

    /// Returns an instance of [`Self`] corresponding to the given index into [`Self::list()`]
    fn from_index(index: usize) -> Option<Self>;

    /// Parses the name as returned by [`Self::name()`] into an instance of [`Self`]
    fn from_name(name: &str) -> Option<Self>;

    /// Returns the non-localized name of this choice
    fn name(&self) -> &'static str;

    /// Returns the localized name for the given locale, if one is set
    fn localized_name(&self, locale: &str) -> Option<&'static str>;
}

#[async_trait::async_trait]
impl<T: ChoiceParameter> crate::SlashArgument for T {
    async fn extract(
        _: &impl serenity::CacheHttp,
        _: crate::CommandOrAutocompleteInteraction<'_>,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<Self, crate::SlashArgError> {
        #[allow(unused_imports)]
        use ::serenity::json::prelude::*; // Required for simd-json :|
        use std::convert::TryInto as _;

        let choice_key = match *value {
            serenity::ResolvedValue::Integer(x) => {
                x.try_into()
                    .map_err(|_| crate::SlashArgError::CommandStructureMismatch {
                        description: "received out of bounds integer",
                    })
            }
            _ => Err(crate::SlashArgError::CommandStructureMismatch {
                description: "expected integer",
            }),
        }?;

        Self::from_index(choice_key).ok_or(crate::SlashArgError::CommandStructureMismatch {
            description: "out of bounds choice key",
        })
    }

    fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        builder.kind(serenity::CommandOptionType::Integer)
    }

    fn choices() -> Vec<crate::CommandParameterChoice> {
        Self::list()
    }
}

#[async_trait::async_trait]
impl<'a, T: ChoiceParameter> crate::PopArgument<'a> for T {
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> Result<(&'a str, usize, Self), (Box<dyn std::error::Error + Send + Sync>, Option<String>)>
    {
        let (args, attachment_index, s) =
            crate::pop_prefix_argument!(String, args, attachment_index, ctx, msg).await?;

        Ok((
            args,
            attachment_index,
            Self::from_name(&s).ok_or((
                Box::new(crate::InvalidChoice {
                    __non_exhaustive: (),
                }) as Box<dyn std::error::Error + Send + Sync>,
                Some(s),
            ))?,
        ))
    }
}
