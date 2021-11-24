#![cfg_attr(docsrs, cfg_attr(all(), doc = include_str!("../README.md")))]
#![cfg_attr(
    not(docsrs),
    doc = "For an overview and usage information of the library, see the repository's README.md file"
)]
#![warn(rust_2018_idioms)]
#![warn(missing_docs)]
#![allow(clippy::type_complexity)]

mod prefix;
pub use prefix::*;

mod slash;
pub use slash::*;

mod event;
pub use event::{Event, EventWrapper};

mod structs;
pub use structs::*;

mod framework;
pub use framework::*;

mod reply;
pub use reply::*;

mod cooldown;
pub use cooldown::*;

pub mod builtins;
/// See [`builtins`]
#[deprecated = "`samples` module was renamed to `builtins`"]
pub mod samples {
    pub use crate::builtins::*;
}

#[doc(no_inline)]
pub use async_trait::async_trait;
pub use futures;
pub use poise_macros::*;
pub use serde_json;
pub use serenity;

/// This module re-exports a bunch of items from all over serenity. Useful if you can't
/// remember the full paths of serenity items.
///
/// One way to use this prelude module in your project is
/// ```rust
/// use poise::serenity_prelude as serenity;
/// ```
pub mod serenity_prelude {
    #[cfg(feature = "collector")]
    #[doc(no_inline)]
    pub use serenity::collector::*;
    #[doc(no_inline)]
    pub use serenity::{
        async_trait,
        builder::*,
        client::{bridge::gateway::*, *},
        http::*,
        model::{
            event::*,
            interactions::{application_command::*, autocomplete::*, message_component::*, *},
            prelude::*,
        },
        prelude::*,
        utils::*,
        *,
    };
}

use std::future::Future;
use std::pin::Pin;

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
