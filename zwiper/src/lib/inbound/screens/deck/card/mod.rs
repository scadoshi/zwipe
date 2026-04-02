//! Deck card management screens.
//!
//! Provides screens for adding and removing cards from decks.

/// Undo/redo action history for card operations.
pub mod action_history;
/// Add cards to deck screen.
pub mod add;
/// View deck cards screen.
pub mod view;
/// Card search filter components.
pub mod filter;
/// Remove cards from deck screen.
pub mod remove;
/// Extracted components for card screens.
mod components;
