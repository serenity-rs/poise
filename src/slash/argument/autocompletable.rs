//! A struct and trait and an instance of the auto-deref specialization trick for the purposes of
//! converting between Discord's autocomplete data and Rust types, in order to run the parameter
//! autocomplete callbacks

#[allow(unused_imports)] // required if serenity simdjson feature is enabled
use crate::serenity::json::prelude::*;
use crate::{serenity_prelude as serenity, SlashArgError};
use std::convert::{TryFrom, TryInto};
use std::marker::PhantomData;

/// A single autocomplete choice, displayed in Discord UI
///
/// This type should be returned by functions set via the `#[autocomplete = ]` attribute on slash
/// command parameters.
pub struct AutocompleteChoice<T> {
    /// Name of the choice, displayed in the Discord UI
    pub name: String,
    /// Value of the choice, sent to the bot
    pub value: T,
}

impl<T: ToString> From<T> for AutocompleteChoice<T> {
    fn from(value: T) -> Self {
        Self {
            name: value.to_string(),
            value,
        }
    }
}

/// Types that can be marked autocompletable in a slash command parameter.
///
/// Includes almost all types that can be used as a slash command parameter in general,
/// except some built-in model types (User, Member, Role...)
// Note to self: this CANNOT be integrated into SlashArgumentHack because some types (User, Channel,
// etc.) cannot be used for autocomplete!
pub trait Autocompletable {
    /// Type of the partial input. This should be `Self` except in cases where a partial input
    /// cannot be parsed into `Self` (e.g. an IP address)
    type Partial;

    /// Try extracting the partial input from the JSON value
    ///
    /// Equivalent to [`crate::SlashArgument::extract`]
    ///
    /// Don't call this method directly! Use [`crate::extract_autocomplete_argument!`]
    fn extract_partial(value: &serenity::json::Value) -> Result<Self::Partial, SlashArgError>;

    /// Serialize an autocompletion choice as a JSON value.
    ///
    /// This is the counterpart to [`Self::extract_partial`]
    ///
    /// Don't call this method directly! Use [`crate::autocomplete_argument_into_json!`]
    fn into_json(self) -> serenity::json::Value;
}

#[doc(hidden)]
pub trait AutocompletableHack<T> {
    type Partial;

    fn extract_partial(self, value: &serenity::json::Value)
        -> Result<Self::Partial, SlashArgError>;

    fn into_json(self, value: T) -> serenity::json::Value;
}

/// Full version of [`crate::Autocompletable::extract_partial`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! extract_autocomplete_argument {
    ($target:ty, $value:expr) => {{
        use $crate::AutocompletableHack as _;
        (&&&&&std::marker::PhantomData::<$target>).extract_partial($value)
    }};
}
/// Full version of [`crate::Autocompletable::into_json`].
///
/// Uses specialization to get full coverage of types. Pass the type as the first argument
#[macro_export]
macro_rules! autocomplete_argument_into_json {
    ($target:ty, $value:expr) => {{
        use $crate::AutocompletableHack as _;
        (&&&&&std::marker::PhantomData::<$target>).into_json($value)
    }};
}

/// Handles arbitrary types that can be parsed from string.
#[async_trait::async_trait]
impl<T> AutocompletableHack<T> for PhantomData<T>
where
    T: serenity::ArgumentConvert + ToString + Send + Sync,
    T::Err: std::error::Error + Send + Sync + 'static,
{
    type Partial = String;

    fn extract_partial(self, value: &serenity::json::Value) -> Result<String, SlashArgError> {
        let string = value
            .as_str()
            .ok_or(SlashArgError::CommandStructureMismatch("expected string"))?;
        Ok(string.to_owned())
    }

    fn into_json(self, value: T) -> serenity::json::Value {
        serenity::json::Value::String(value.to_string())
    }
}

// Handles all integers, signed and unsigned.
#[async_trait::async_trait]
impl<T: TryFrom<i64> + Into<serenity::json::Value> + Send + Sync> AutocompletableHack<T>
    for &PhantomData<T>
{
    type Partial = T;

    fn extract_partial(self, value: &serenity::json::Value) -> Result<T, SlashArgError> {
        let number = value
            .as_i64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected integer"))?;
        number.try_into().map_err(|_| SlashArgError::Parse {
            error: crate::IntegerOutOfBounds.into(),
            input: number.to_string(),
        })
    }

    fn into_json(self, value: T) -> serenity::json::Value {
        value.into()
    }
}

#[async_trait::async_trait]
impl AutocompletableHack<f32> for &&PhantomData<f32> {
    type Partial = f32;

    fn extract_partial(self, value: &serenity::json::Value) -> Result<f32, SlashArgError> {
        Ok(value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))? as f32)
    }

    fn into_json(self, value: f32) -> serenity::json::Value {
        serenity::json::Value::from(value as f32)
    }
}

#[async_trait::async_trait]
impl AutocompletableHack<f64> for &&PhantomData<f64> {
    type Partial = f64;

    fn extract_partial(self, value: &serenity::json::Value) -> Result<f64, SlashArgError> {
        value
            .as_f64()
            .ok_or(SlashArgError::CommandStructureMismatch("expected float"))
    }

    fn into_json(self, value: f64) -> serenity::json::Value {
        serenity::json::Value::from(value as f64)
    }
}

#[async_trait::async_trait]
impl<T: Autocompletable> AutocompletableHack<T> for &&PhantomData<T> {
    type Partial = T::Partial;

    fn extract_partial(self, value: &serenity::json::Value) -> Result<T::Partial, SlashArgError> {
        <T as Autocompletable>::extract_partial(value)
    }

    fn into_json(self, value: T) -> serenity::json::Value {
        value.into_json()
    }
}
