use std::str::FromStr;

use crate::{
    serenity_prelude as serenity, PopArgument, PopArgumentResult, SlashArgError, SlashArgument,
};

/// A wrapper for `T` to implement [`SlashArgument`] and [`PopArgument`] via [`FromStr`].
///
/// This is useful if you need to take an argument via a string, but immediately convert it via [`FromStr`].
pub struct StrArg<T>(pub T);

#[async_trait::async_trait]
impl<T> SlashArgument for StrArg<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    fn create(builder: serenity::CreateCommandOption) -> serenity::CreateCommandOption {
        builder.kind(serenity::CommandOptionType::String)
    }

    async fn extract(
        _: &serenity::Context,
        _: &serenity::CommandInteraction,
        value: &serenity::ResolvedValue<'_>,
    ) -> Result<Self, SlashArgError> {
        let serenity::ResolvedValue::String(value) = value else {
            return Err(SlashArgError::new_command_structure_mismatch(
                "expected a String",
            ));
        };

        match T::from_str(value) {
            Ok(value) => Ok(Self(value)),
            Err(err) => Err(SlashArgError::Parse {
                error: err.into(),
                input: String::from(*value),
            }),
        }
    }
}

#[async_trait::async_trait]
impl<'a, T> PopArgument<'a> for StrArg<T>
where
    T: FromStr,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    async fn pop_from(
        args: &'a str,
        attachment_index: usize,
        ctx: &serenity::Context,
        msg: &serenity::Message,
    ) -> PopArgumentResult<'a, Self> {
        let (args, attach_idx, value) = String::pop_from(args, attachment_index, ctx, msg).await?;
        match T::from_str(&value) {
            Ok(value) => Ok((args, attach_idx, Self(value))),
            Err(err) => Err((Box::new(err), Some(value))),
        }
    }
}
