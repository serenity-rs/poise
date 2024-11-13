//! Prefix command permissions calculation

#[cfg(feature = "cache")]
mod cache;
#[cfg(not(feature = "cache"))]
mod http;

#[cfg(feature = "cache")]
pub(super) use cache::get_author_and_bot_permissions;

#[cfg(not(feature = "cache"))]
pub(super) use http::get_author_and_bot_permissions;
