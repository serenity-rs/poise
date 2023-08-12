//! Hosts just the `AutocompleteChoice` type. This type will probably move somewhere else

use serenity::all as serenity;

/// A single autocomplete choice, displayed in Discord UI
///
/// This type can be returned by functions set via the `#[autocomplete = ]` attribute on slash
/// command parameters.
///
/// For more information, see the autocomplete.rs file in the `framework_usage` example
pub struct AutocompleteChoice<T> {
    /// Label of the choice, displayed in the Discord UI
    pub label: String,
    /// Value of the choice, sent to the bot
    pub value: T,
    #[doc(hidden)]
    pub __non_exhaustive: (),
}

impl<T> AutocompleteChoice<T> {
    /// Creates a new autocomplete choice with the given text
    pub fn new(value: T) -> AutocompleteChoice<T>
    where
        T: ToString,
    {
        Self {
            label: value.to_string(),
            value,
            __non_exhaustive: (),
        }
    }

    /// Like [`Self::new()`], but you can customize the JSON value sent to Discord as the unique
    /// identifier of this autocomplete choice.
    pub fn new_with_value(label: impl Into<String>, value: T) -> Self {
        Self {
            label: label.into(),
            value,
            __non_exhaustive: (),
        }
    }
}

impl<T> AutocompleteChoice<T> {
    /// Converts this type to the serenity equivalent in order to pass it to serenity's API endpoint
    /// functions.
    pub fn to_serenity(self) -> serenity::AutocompleteChoice
    where
        T: Into<serenity::json::Value>,
    {
        serenity::AutocompleteChoice::new(self.label, self.value)
    }
}

impl<T: ToString> From<T> for AutocompleteChoice<T> {
    fn from(value: T) -> Self {
        Self {
            label: value.to_string(),
            value,
            __non_exhaustive: (),
        }
    }
}
