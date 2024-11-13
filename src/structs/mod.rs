//! Plain data structs that define the framework configuration.
#![allow(clippy::needless_lifetimes)] // Triggered from inside derivative

use std::borrow::Cow;

mod context;
pub use context::*;

mod framework_options;
pub use framework_options::*;

mod command;
pub use command::*;

mod prefix;
pub use prefix::*;

mod slash;
pub use slash::*;

mod framework_error;
pub use framework_error::*;

/// A type alias for `&'static str` or `String`
pub(crate) type CowStr = Cow<'static, str>;

/// A type alias for `&'static [T]` or `Vec<T>`
pub(crate) type CowVec<T> = Cow<'static, [T]>;
