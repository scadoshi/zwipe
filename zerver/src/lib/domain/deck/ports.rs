//! Port traits for deck building operations.
//!
//! This module defines the interfaces (ports) for deck management in hexagonal architecture.
//! Decks are collections of Magic: The Gathering cards with metadata like name, commander, and copy limits.
//!
//! # Hexagonal Architecture
//!
//! - **DeckRepository**: Database port (deck and deck_card persistence)
//! - **DeckService**: Service port (business logic + authorization checks)
//!
//! # Deck Structure
//!
//! - **DeckProfile**: Metadata (name, commander, copy limit, owner)
//! - **DeckCard**: Card in deck (card_id, quantity, category)
//! - **Deck**: Complete view (profile + all cards)
//!
//! # Implementation
//!
//! - Repositories: `outbound/sqlx/deck` (PostgreSQL)
//! - Services: `domain/deck/services` (authorization + validation)

use std::future::Future;

use crate::domain::deck::models::{
    deck::{
        create_deck_profile::{CreateDeckProfile, CreateDeckProfileError},
        deck_profile::DeckProfile,
        delete_deck::{DeleteDeck, DeleteDeckError},
        get_deck::GetDeckError,
        get_deck_profile::{GetDeckProfile, GetDeckProfileError},
        get_deck_profiles::GetDeckProfiles,
        update_deck_profile::{UpdateDeckProfile, UpdateDeckProfileError},
        Deck,
    },
    deck_card::{
        create_deck_card::{CreateDeckCard, CreateDeckCardError},
        delete_deck_card::{DeleteDeckCard, DeleteDeckCardError},
        get_deck_card::GetDeckCardError,
        update_deck_card::{UpdateDeckCard, UpdateDeckCardError},
        DeckCard,
    },
};

/// Database port for deck building operations.
///
/// Defines all database operations for deck profiles and deck cards.
/// Implemented by PostgreSQL adapter in `outbound/sqlx/deck`.
///
/// # Authorization
///
/// This trait does NOT enforce authorization - it trusts the service layer
/// to verify ownership before calling repository methods.
pub trait DeckRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Creates a new deck profile (empty deck with metadata).
    ///
    /// Inserts deck profile into database with name, commander, copy limit, and owner.
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    /// Adds a card to a deck (or updates quantity if already exists).
    ///
    /// Creates deck_card join table entry with card_id, quantity, and category.
    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves a single deck profile by ID.
    ///
    /// Returns deck metadata without card list.
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all deck profiles for a user.
    ///
    /// Returns list of deck metadata (no card lists).
    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves all cards in a deck.
    ///
    /// Returns deck_card entries with full card data joined from cards table.
    fn get_deck_cards(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    // ========
    //  update
    // ========

    /// Updates deck profile metadata (name, commander, copy limit).
    ///
    /// Allows changing deck configuration after creation.
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    /// Updates a card's quantity or category in a deck.
    ///
    /// Modifies existing deck_card entry.
    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a deck and all its cards (cascading delete).
    ///
    /// Removes deck profile and all deck_card entries.
    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    /// Removes a card from a deck.
    ///
    /// Deletes deck_card entry for specified card.
    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}

/// Service port for deck building business logic.
///
/// Orchestrates deck operations by combining repository calls with authorization checks.
/// Ensures users can only access/modify their own decks.
///
/// # Authorization Pattern
///
/// Every operation verifies deck ownership before proceeding:
/// 1. Extract user_id from request
/// 2. Fetch deck from database
/// 3. Verify deck.owner_id == user_id
/// 4. Proceed with operation or return Unauthorized error
///
/// # Implementation
///
/// Implemented in `domain/deck/services` with authorization + repository calls.
pub trait DeckService: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Creates a new deck profile with authorization check.
    ///
    /// Validates request and creates empty deck owned by requesting user.
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    /// Adds a card to a deck with authorization check.
    ///
    /// Verifies deck ownership, then adds card or updates quantity.
    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves a deck profile with authorization check.
    ///
    /// Verifies ownership before returning deck metadata.
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all decks for the requesting user.
    ///
    /// Returns user's deck list (metadata only, no cards).
    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves complete deck (profile + cards) with authorization check.
    ///
    /// Verifies ownership, fetches profile and all cards, returns composite.
    fn get_deck(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Deck, GetDeckError>> + Send;

    // ========
    //  update
    // ========

    /// Updates deck profile with authorization check.
    ///
    /// Verifies ownership before updating name, commander, or copy limit.
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    /// Updates card quantity/category with authorization check.
    ///
    /// Verifies deck ownership before modifying deck_card entry.
    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a deck with authorization check.
    ///
    /// Verifies ownership before deleting deck and all cards.
    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    /// Removes a card from deck with authorization check.
    ///
    /// Verifies deck ownership before deleting deck_card entry.
    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}
