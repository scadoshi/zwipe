//! Every API call, one [`Endpoint`](crate::http::endpoint::Endpoint) impl each.
//!
//! Grouped to mirror [`contracts`](crate::http::contracts). An endpoint names the
//! method, path, auth and types for one call, so the method a path is used
//! with can't drift from the path itself.

pub mod auth;
pub mod card;
pub mod deck;
pub mod meta;
pub mod metrics;
pub mod suggestions;
pub mod user;
