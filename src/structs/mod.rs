//! Plain data structs that define the framework configuration.
#![allow(clippy::needless_lifetimes)] // Triggered from inside derivative

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
