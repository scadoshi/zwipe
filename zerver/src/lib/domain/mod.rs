//! Core domain models and business logic.
//!
//! Contains the heart of the application: cards, decks, users, and authentication.
//! All validation and business rules are enforced at this layer.

pub mod auth;
pub mod card;
pub mod deck;
pub mod email;
pub mod health;
pub mod metrics;
pub mod upkeep;
pub mod user;
