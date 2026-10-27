//! Top-level page components (screens).
//!
//! Each module represents a major section of the application with its own routing.

/// Account screen and its sheets (username, email, password, delete).
pub mod account;
/// How the user picks: the heatmap, swipe outcomes, rates and hard passes.
pub mod activity;
/// Authentication screens (login, register).
pub mod auth;
/// Release history screen (shared changelog).
pub mod changelog;
/// Appearance and card settings (theme, app icon, Universes Beyond).
pub mod config;
/// Deck builder and card management screens.
pub mod deck;
/// Home/landing page screen.
pub mod home;
/// Legal screens (privacy policy).
pub mod legal;
/// Oracle-tag dictionary (read-only, searchable reference of all oracle tags).
pub mod oracle_tag_dictionary;
/// Oracle-tag example cards (deck-free swipe browse of a tag's cards).
pub mod oracle_tag_examples;
