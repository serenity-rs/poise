mod autocompletable;
pub use autocompletable::*;

mod into_stream_hack;
pub use into_stream_hack::*;

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
