//! Top-level page components (screens).
//!
//! Each module represents a major section of the application with its own routing.

/// Authentication screens (login, register).
pub mod auth;
/// Release history screen (shared changelog).
pub mod changelog;
/// Deck builder and card management screens.
pub mod deck;
/// Home/landing page screen.
pub mod home;
/// Legal screens (privacy policy).
pub mod legal;
/// Oracle-tag dictionary (read-only, searchable reference of all oracle tags).
pub mod oracle_tag_dictionary;
/// User profile management screens (change email, password, username).
pub mod profile;
