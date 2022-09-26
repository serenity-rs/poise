//! Application command argument handling code

mod slash_macro;
pub use slash_macro::*;

mod slash_trait;
pub use slash_trait::*;

mod autocompletable;
pub use autocompletable::*;

mod into_stream;
pub use into_stream::*;

mod interaction;
pub use interaction::*;

// Reexport utils
use poise_utils::*;

/// A single drop-down choice in a slash command choice parameter
#[derive(Debug, Clone)]
pub struct CommandParameterChoice {
    /// Label of this choice
    pub name: String,
    /// Localized labels with locale string as the key (slash-only)
    pub localizations: std::collections::HashMap<String, String>,
}
