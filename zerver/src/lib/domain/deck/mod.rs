//! Deck management domain logic.
//!
//! This module provides comprehensive deck building and management for Magic: The Gathering.
//! Users can create decks, add/remove cards, and manage deck configurations like commander
//! selection and copy limits (singleton vs. standard).
//!
//! # Core Concepts
//!
//! ## Deck Structure
//!
//! A deck consists of two components:
//! - **DeckProfile**: Metadata (name, commander, copy limit, owner)
//! - **DeckCards**: Card inventory (which cards, how many copies)
//!
//! ## Copy Limits
//!
//! Decks enforce copy limits per Magic format rules:
//! - **Singleton (1)**: Commander format - max 1 copy of each card (except basic lands)
//! - **Standard (4)**: Standard format - max 4 copies of each card (except basic lands)
//!
//! ## Authorization
//!
//! All deck operations verify ownership - users can only view/modify their own decks.
//! This is enforced at the service layer by comparing user IDs.
//!
//! # Operations
//!
//! - **Create Deck**: Initialize deck profile with name and settings
//! - **Update Deck**: Modify name, commander, or copy limit
//! - **Delete Deck**: Remove deck and all its cards
//! - **Add Card**: Add card to deck with quantity
//! - **Update Card Quantity**: Change how many copies of a card
//! - **Remove Card**: Delete card from deck
//! - **Get Deck**: Retrieve complete deck with all card data
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::deck::models::deck::create_deck_profile::CreateDeckProfile;
//!
//! // Create a Commander deck
//! let request = CreateDeckProfile::new(
//!     "My EDH Deck",
//!     Some(commander_card_id),
//!     1, // singleton
//!     user_id
//! )?;
//!
//! let deck_profile = deck_service.create_deck_profile(request).await?;
//!
//! // Add cards to the deck
//! let add_card = CreateDeckCard::new(
//!     user_id,
//!     deck_profile.id,
//!     card_id,
//!     1 // quantity
//! )?;
//! deck_service.create_deck_card(add_card).await?;
//! ```

/// Deck models and value objects (DeckProfile, Deck, DeckCard, operations).
pub mod models;

/// Port traits (interfaces) for deck operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for deck business logic.
#[cfg(feature = "zerver")]
pub mod services;
