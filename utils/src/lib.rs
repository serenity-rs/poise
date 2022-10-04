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
        model::application::{command::*, component::*},
        model::prelude::*,
        prelude::*,
        utils::*,
        *,
    };
    #[doc(no_inline)]
    #[cfg(feature = "full_serenity")]
    pub use serenity::{
        client::{
            bridge::gateway::{event::*, *},
            *,
        },
        collector::*,
        http::*,
    };
}

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = std::pin::Pin<Box<dyn std::future::Future<Output = T> + Send + 'a>>;
