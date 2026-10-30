//! ZWIPER: the ZWIPE app's screens, components, routing and the platform
//! stores behind them.
//!
//! - [`domain`]: user-facing error messages and language names
//! - [`config`]: values baked in at build time
//! - [`inbound`]: the Dioxus UI, its screens, components and router
//! - [`outbound`]: the session, on-device stores and platform calls

#![warn(missing_docs)]

/// Values baked in at build time.
pub mod config;
/// User-facing error messages and language names.
pub mod domain;
/// The Dioxus UI: screens, components and the router.
pub mod inbound;
/// The session, on-device stores and platform calls.
pub mod outbound;

#[cfg(test)]
mod test_fixtures;
