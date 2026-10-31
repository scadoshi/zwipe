//! zwiper-web: every Zwipe app screen for a big browser window.
//!
//! The same hexagonal split as zwiper. `domain` holds the app's rules as pure
//! functions with tests, `outbound` the browser's storage and timer, and
//! `inbound` the router, the shell and the screens.

pub mod config;
pub mod domain;
pub mod inbound;
pub mod outbound;

#[cfg(test)]
mod test_fixtures;
