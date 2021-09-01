#![warn(rust_2018_idioms)]
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

pub mod defaults;

pub use async_trait::async_trait;
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
    pub use serenity::collector::*;
    pub use serenity::{
        async_trait,
        builder::*,
        client::{bridge::gateway::*, *},
        http::*,
        model::{
            event::*,
            interactions::{application_command::*, *},
            prelude::*,
        },
        prelude::*,
        utils::*,
        Error,
    };
}

use std::future::Future;
use std::pin::Pin;

/// Shorthand for a wrapped async future with a lifetime, used by many parts of this framework.
///
/// An owned future has the `'static` lifetime.
pub type BoxFuture<'a, T> = Pin<Box<dyn Future<Output = T> + Send + 'a>>;
