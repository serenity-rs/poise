/// This module re-exports a bunch of items from all over serenity. Useful if you can't
/// remember the full paths of serenity items.
///
/// One way to use this prelude module in your project is
/// ```rust
/// use poise::serenity_prelude as serenity;
/// ```
pub mod serenity_prelude {
    #[doc(no_inline)]
    #[cfg(feature = "cache")]
    pub use serenity::cache::*;
    #[doc(no_inline)]
    pub use serenity::{
        async_trait,
        builder::*,
        client::{
            bridge::gateway::{event::*, *},
            *,
        },
        collector::*,
        http::*,
        // Explicit imports to resolve ambiguity between model::prelude::* and
        // model::application::interaction::* due to deprecated same-named type aliases
        model::{
            application::interaction::{
                Interaction, InteractionResponseType, InteractionType,
                MessageFlags as InteractionResponseFlags, MessageInteraction,
            },
            // There's two MessageFlags in serenity. The interaction response specific one was
            // renamed to InteractionResponseFlags above so we can keep this one's name the same
            channel::MessageFlags,
        },
        model::{
            application::{
                command::*,
                component::*,
                interaction::{application_command::*, message_component::*, modal::*, *},
            },
            event::*,
            prelude::*,
        },
        prelude::*,
        utils::*,
        *,
    };
}

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
