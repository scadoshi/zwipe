//! The screen logic every Zwipe client shares: zwiper (the app), zwiper-web
//! and zwiper-tui.
//!
//! Each client draws its screens its own way. What a screen shows and does
//! is decided here once, as pure functions and plain types, so the three
//! clients can't drift apart. Nothing here names a UI crate, so it builds for
//! iOS, Android, the terminal and the browser alike. Domain rules the server
//! shares live in zwipe-core instead.
#![warn(clippy::must_use_candidate)]

pub mod account;
pub mod activity;
pub mod add;
pub mod buy;
pub mod card;
pub mod changelog;
pub mod colors;
pub mod commander;
pub mod deck;
pub mod deck_cards;
pub mod deck_signals;
pub mod dictionary;
pub mod filter;
pub mod generation;
pub mod hint;
pub mod home;
pub mod playstyle;
pub mod preferences;
pub mod screens;
pub mod session;
pub mod stack;
pub mod stores;
pub mod support;
pub mod telemetry;
pub mod timing;
pub mod toast;

#[cfg(test)]
mod test_support;
