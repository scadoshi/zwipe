//! Port traits for deck building operations.
//!
//! This module defines the interfaces (ports) for deck management in hexagonal architecture.
//! Decks are collections of Magic: The Gathering cards with metadata like name, commander, and copy limits.

use std::future::Future;

use zwipe_core::domain::card::{Card, search_card::card_filter::CardFilter};
use crate::domain::deck::models::{
    deck::{
        clone_deck::CloneDeckError,
        create_deck_profile::CreateDeckProfileError,
        delete_deck::DeleteDeckError,
        get_deck::GetDeckError,
        get_deck_profile::GetDeckProfileError,
        get_deck_tokens::GetDeckTokensError,
        import_archidekt::ArchidektCard,
        search_deck_cards::SearchDeckCardsError,
        update_deck_profile::UpdateDeckProfileError,
    },
    deck_card::{
        create_deck_card::CreateDeckCardError,
        delete_deck_card::DeleteDeckCardError,
        get_deck_card::GetDeckCardError,
        import_deck_cards::ImportDeckCardsError,
        update_deck_card::UpdateDeckCardError,
    },
};
use zwipe_core::domain::deck::{
    Deck, DeckCard, DeckName,
    deck_profile::DeckProfile,
    requests::{
        clone_deck::CloneDeck,
        create_deck_card::CreateDeckCard,
        create_deck_profile::CreateDeckProfile,
        delete_deck::DeleteDeck,
        delete_deck_card::DeleteDeckCard,
        get_deck_profile::GetDeckProfile,
        get_deck_profiles::GetDeckProfiles,
        import_deck_cards::{ImportDeckCards, ImportDeckCardsResult},
        update_deck_card::UpdateDeckCard,
        update_deck_profile::UpdateDeckProfile,
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

    // =======
    //  count
    // =======

    /// Returns the number of decks owned by a user.
    fn count_decks_by_user(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<i64, anyhow::Error>> + Send;

    /// Returns the total card quantity (sum of all copies) in a deck.
    fn count_cards_in_deck(
        &self,
        deck_id: uuid::Uuid,
    ) -> impl Future<Output = Result<i64, anyhow::Error>> + Send;

    /// Returns the sum of quantities for specific oracle_ids in a deck (deck board only).
    fn sum_quantities_for_oracle_ids(
        &self,
        deck_id: uuid::Uuid,
        oracle_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<i64, anyhow::Error>> + Send;

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

    /// Bulk upserts cards into a deck (insert or overwrite quantity).
    fn bulk_create_deck_cards(
        &self,
        request: &ImportDeckCards,
        cards: &[(uuid::Uuid, uuid::Uuid, i32, String)],
    ) -> impl Future<Output = Result<Vec<DeckCard>, ImportDeckCardsError>> + Send;

    /// Deletes every card on `board` whose oracle_id is not in `keep_oracle_ids`.
    /// Used by replace-mode imports to make a board exactly match the imported
    /// list. Callers must have verified deck ownership first.
    fn delete_deck_cards_not_in(
        &self,
        deck_id: uuid::Uuid,
        board: &str,
        keep_oracle_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    // ========
    //  clone
    // ========

    /// Transactionally copies a source deck's profile and all entries into a
    /// new deck owned by `owner_id` with the given `new_name`. Returns the
    /// new deck's id. The caller must have already verified `owner_id` owns
    /// `source_deck_id` — this method performs no authorization check.
    fn clone_deck(
        &self,
        source_deck_id: uuid::Uuid,
        new_name: &DeckName,
        owner_id: uuid::Uuid,
    ) -> impl Future<Output = Result<uuid::Uuid, CloneDeckError>> + Send;
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

    /// Deck-aware card search with authorization check: applies `filter` but
    /// excludes cards already in the deck (any board, plus commander/partner/
    /// background/signature slots), and when `filter` has no explicit
    /// `order_by` and the deck's commander has cached synergy data, orders
    /// results by synergy descending. Explicit sort always wins; absent
    /// signal degrades gracefully to the filter's own semantics.
    fn search_deck_cards(
        &self,
        request: &GetDeckProfile,
        filter: &CardFilter,
    ) -> impl Future<Output = Result<Vec<Card>, SearchDeckCardsError>> + Send;

    /// Retrieves all token cards produced by the cards in a deck.
    fn get_deck_tokens(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<Card>, GetDeckTokensError>> + Send;

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

    /// Imports cards from a plain-text decklist with authorization check.
    fn import_deck_cards(
        &self,
        request: &ImportDeckCards,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ImportDeckCardsError>> + Send;

    /// Imports an Archidekt card list into an existing deck owned by `user_id`,
    /// onto the given board — exactly like `import_deck_cards`, except cards
    /// resolve by Scryfall printing id (with a name fallback) instead of by
    /// name. With [`zwipe_core::domain::deck::ImportMode::Replace`], the board
    /// is made to exactly match the list.
    fn import_archidekt_deck(
        &self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        cards: &[ArchidektCard],
        board: zwipe_core::domain::deck::Board,
        email_verified: bool,
        mode: zwipe_core::domain::deck::ImportMode,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ImportDeckCardsError>> + Send;

    // ========
    //  clone
    // ========

    /// Clones an existing deck owned by the caller into a new deck with a
    /// caller-chosen name. Performs source ownership check and deck-count
    /// limit enforcement, then delegates the transactional copy to the
    /// repository. Returns the new deck's id.
    fn clone_deck(
        &self,
        request: &CloneDeck,
    ) -> impl Future<Output = Result<uuid::Uuid, CloneDeckError>> + Send;
}
