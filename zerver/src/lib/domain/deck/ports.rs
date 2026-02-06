//! Port traits for deck building operations.
//!
//! This module defines the interfaces (ports) for deck management in hexagonal architecture.
//! Decks are collections of Magic: The Gathering cards with metadata like name, commander, and copy limits.

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
pub trait DeckRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Creates a new deck profile.
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    /// Adds a card to a deck.
    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves a single deck profile by ID.
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all deck profiles for a user.
    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves all cards in a deck.
    fn get_deck_cards(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    // ========
    //  update
    // ========

    /// Updates deck profile metadata.
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    /// Updates a card's quantity or category in a deck.
    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a deck and all its cards.
    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    /// Removes a card from a deck.
    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}

/// Service port for deck building business logic.
pub trait DeckService: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========

    /// Creates a new deck profile with authorization check.
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    /// Adds a card to a deck with authorization check.
    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // =====
    //  get
    // =====

    /// Retrieves a deck profile with authorization check.
    fn get_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all decks for the requesting user.
    fn get_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves complete deck with authorization check.
    fn get_deck(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Deck, GetDeckError>> + Send;

    // ========
    //  update
    // ========

    /// Updates deck profile with authorization check.
    fn update_deck_profile(
        &self,
        request: &UpdateDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, UpdateDeckProfileError>> + Send;

    /// Updates card quantity/category with authorization check.
    fn update_deck_card(
        &self,
        request: &UpdateDeckCard,
    ) -> impl Future<Output = Result<DeckCard, UpdateDeckCardError>> + Send;

    // ========
    //  delete
    // ========

    /// Deletes a deck with authorization check.
    fn delete_deck(
        &self,
        request: &DeleteDeck,
    ) -> impl Future<Output = Result<(), DeleteDeckError>> + Send;

    /// Removes a card from deck with authorization check.
    fn delete_deck_card(
        &self,
        request: &DeleteDeckCard,
    ) -> impl Future<Output = Result<(), DeleteDeckCardError>> + Send;
}
