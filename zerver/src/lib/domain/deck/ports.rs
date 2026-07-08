//! Port traits for deck building operations.
//!
//! This module defines the interfaces (ports) for deck management in hexagonal architecture.
//! Decks are collections of Magic: The Gathering cards with metadata like name, commander, and copy limits.

use crate::domain::BoxFuture;
use std::future::Future;

use crate::domain::deck::models::{
    deck::{
        clear_deck_suppressions::ClearDeckSuppressionsError, clone_deck::CloneDeckError,
        create_deck_profile::CreateDeckProfileError, delete_deck::DeleteDeckError,
        get_deck::GetDeckError, get_deck_profile::GetDeckProfileError,
        get_deck_tokens::GetDeckTokensError, import_archidekt::ArchidektCard,
        search_deck_cards::SearchDeckCardsError,
        share_deck::{GetSharedDeckError, ShareDeckError, SharedDeck},
        skip_deck_card::SkipDeckCardError,
        update_deck_profile::UpdateDeckProfileError,
    },
    deck_card::{
        create_deck_card::CreateDeckCardError, delete_deck_card::DeleteDeckCardError,
        get_deck_card::GetDeckCardError, import_deck_cards::ImportDeckCardsError,
        update_deck_card::UpdateDeckCardError,
    },
};
use zwipe_core::domain::card::{Card, search_card::card_filter::CardQuery};
use zwipe_core::domain::deck::{
    Deck, DeckCard, DeckName,
    deck_profile::DeckProfile,
    requests::{
        clear_deck_suppressions::ClearDeckSuppressions,
        clone_deck::CloneDeck,
        create_deck_card::CreateDeckCard,
        create_deck_profile::CreateDeckProfile,
        delete_deck::DeleteDeck,
        delete_deck_card::DeleteDeckCard,
        get_deck_profile::GetDeckProfile,
        get_deck_profiles::GetDeckProfiles,
        import_deck_cards::{ImportDeckCards, ImportDeckCardsResult},
        skip_deck_card::SkipDeckCard,
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
    /// list. Callers must have verified deck ownership first. Bulk deletes do
    /// NOT suppress — importing a new list isn't a per-card rejection.
    fn delete_deck_cards_not_in(
        &self,
        deck_id: uuid::Uuid,
        board: &str,
        keep_oracle_ids: &[uuid::Uuid],
    ) -> impl Future<Output = Result<(), anyhow::Error>> + Send;

    /// Deletes a deck's entire suppression set (skips + removals), returning
    /// the number of rows removed. Ownership-checked.
    fn clear_deck_suppressions(
        &self,
        request: &ClearDeckSuppressions,
    ) -> impl Future<Output = Result<u64, ClearDeckSuppressionsError>> + Send;

    /// Upserts a single skip suppression for a deck. Ownership-checked.
    fn skip_deck_card(
        &self,
        request: &SkipDeckCard,
    ) -> impl Future<Output = Result<(), SkipDeckCardError>> + Send;

    /// Deletes a single skip-sourced suppression (undo). Removal suppressions
    /// are untouched. Ownership-checked.
    fn unskip_deck_card(
        &self,
        request: &SkipDeckCard,
    ) -> impl Future<Output = Result<(), SkipDeckCardError>> + Send;

    // ========
    //  clone
    // ========

    /// Transactionally copies a source deck's profile and all entries into a
    /// new deck owned by `owner_id` with the given `new_name`. Returns the
    /// new deck's id. The caller must have already verified `owner_id` owns
    /// `source_deck_id` — this method performs no authorization check.
    /// `share_token` is deliberately not copied: clones start private.
    fn clone_deck(
        &self,
        source_deck_id: uuid::Uuid,
        new_name: &DeckName,
        owner_id: uuid::Uuid,
    ) -> impl Future<Output = Result<uuid::Uuid, CloneDeckError>> + Send;

    // =======
    //  share
    // =======

    /// Generates (or regenerates) the deck's share token and returns it.
    /// The caller must have already verified ownership.
    fn set_share_token(
        &self,
        deck_id: uuid::Uuid,
    ) -> impl Future<Output = Result<uuid::Uuid, ShareDeckError>> + Send;

    /// Nulls the deck's share token (revokes the public link). The caller
    /// must have already verified ownership.
    fn clear_share_token(
        &self,
        deck_id: uuid::Uuid,
    ) -> impl Future<Output = Result<(), ShareDeckError>> + Send;

    /// Resolves a share token to `(deck_id, owner_user_id)`, or `None` when
    /// no deck carries this token (never shared, or sharing was stopped).
    fn get_deck_id_by_share_token(
        &self,
        token: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<(uuid::Uuid, uuid::Uuid)>, anyhow::Error>> + Send;
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
    ///
    /// Returns `(cards, synergy_warming)` where `synergy_warming` is true when
    /// the filter requested synergy but the commander's cache wasn't available
    /// (cold), so the search fell back to the full pool.
    fn search_deck_cards(
        &self,
        request: &GetDeckProfile,
        filter: &CardQuery,
    ) -> impl Future<Output = Result<(Vec<Card>, bool), SearchDeckCardsError>> + Send;

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

    /// Clears a deck's suppression set (skipped/removed cards come back into
    /// the swipe pool) with authorization check. Returns rows removed.
    fn clear_deck_suppressions(
        &self,
        request: &ClearDeckSuppressions,
    ) -> impl Future<Output = Result<u64, ClearDeckSuppressionsError>> + Send;

    /// Suppresses a single card for a deck (durable skip) with authorization
    /// check.
    fn skip_deck_card(
        &self,
        request: &SkipDeckCard,
    ) -> impl Future<Output = Result<(), SkipDeckCardError>> + Send;

    /// Removes a single skip suppression (undo) with authorization check.
    fn unskip_deck_card(
        &self,
        request: &SkipDeckCard,
    ) -> impl Future<Output = Result<(), SkipDeckCardError>> + Send;

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

    // =======
    //  share
    // =======

    /// Generates (or regenerates) the deck's share token with authorization
    /// check. Returns the new token; any previous link dies with its token.
    fn share_deck(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<uuid::Uuid, ShareDeckError>> + Send;

    /// Revokes the deck's share token with authorization check.
    fn unshare_deck(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<(), ShareDeckError>> + Send;

    /// Resolves a share token to the full deck aggregate plus command zone
    /// cards. **Unauthenticated**: possession of the token is the authority.
    fn get_shared_deck(
        &self,
        token: uuid::Uuid,
    ) -> impl Future<Output = Result<SharedDeck, GetSharedDeckError>> + Send;
}

/// Object-safe wrapper used by `AppState` so the concrete service type stays
/// out of the generic parameter list. Auto-implemented for any `DeckService`.
pub trait ErasedDeckService: Send + Sync + 'static {
    /// See [`DeckService::create_deck_profile`].
    fn create_deck_profile<'a>(
        &'a self,
        request: &'a CreateDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, CreateDeckProfileError>>;

    /// See [`DeckService::create_deck_card`].
    fn create_deck_card<'a>(
        &'a self,
        request: &'a CreateDeckCard,
    ) -> BoxFuture<'a, Result<DeckCard, CreateDeckCardError>>;

    /// See [`DeckService::get_deck_profile`].
    fn get_deck_profile<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, GetDeckProfileError>>;

    /// See [`DeckService::get_deck_profiles`].
    fn get_deck_profiles<'a>(
        &'a self,
        request: &'a GetDeckProfiles,
    ) -> BoxFuture<'a, Result<Vec<DeckProfile>, GetDeckProfileError>>;

    /// See [`DeckService::get_deck`].
    fn get_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<Deck, GetDeckError>>;

    /// See [`DeckService::search_deck_cards`].
    fn search_deck_cards<'a>(
        &'a self,
        request: &'a GetDeckProfile,
        filter: &'a CardQuery,
    ) -> BoxFuture<'a, Result<(Vec<Card>, bool), SearchDeckCardsError>>;

    /// See [`DeckService::get_deck_tokens`].
    fn get_deck_tokens<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetDeckTokensError>>;

    /// See [`DeckService::update_deck_profile`].
    fn update_deck_profile<'a>(
        &'a self,
        request: &'a UpdateDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, UpdateDeckProfileError>>;

    /// See [`DeckService::update_deck_card`].
    fn update_deck_card<'a>(
        &'a self,
        request: &'a UpdateDeckCard,
    ) -> BoxFuture<'a, Result<DeckCard, UpdateDeckCardError>>;

    /// See [`DeckService::delete_deck`].
    fn delete_deck<'a>(
        &'a self,
        request: &'a DeleteDeck,
    ) -> BoxFuture<'a, Result<(), DeleteDeckError>>;

    /// See [`DeckService::delete_deck_card`].
    fn delete_deck_card<'a>(
        &'a self,
        request: &'a DeleteDeckCard,
    ) -> BoxFuture<'a, Result<(), DeleteDeckCardError>>;

    /// See [`DeckService::clear_deck_suppressions`].
    fn clear_deck_suppressions<'a>(
        &'a self,
        request: &'a ClearDeckSuppressions,
    ) -> BoxFuture<'a, Result<u64, ClearDeckSuppressionsError>>;

    /// See [`DeckService::skip_deck_card`].
    fn skip_deck_card<'a>(
        &'a self,
        request: &'a SkipDeckCard,
    ) -> BoxFuture<'a, Result<(), SkipDeckCardError>>;

    /// See [`DeckService::unskip_deck_card`].
    fn unskip_deck_card<'a>(
        &'a self,
        request: &'a SkipDeckCard,
    ) -> BoxFuture<'a, Result<(), SkipDeckCardError>>;

    /// See [`DeckService::import_deck_cards`].
    fn import_deck_cards<'a>(
        &'a self,
        request: &'a ImportDeckCards,
    ) -> BoxFuture<'a, Result<ImportDeckCardsResult, ImportDeckCardsError>>;

    /// See [`DeckService::import_archidekt_deck`].
    fn import_archidekt_deck<'a>(
        &'a self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        cards: &'a [ArchidektCard],
        board: zwipe_core::domain::deck::Board,
        email_verified: bool,
        mode: zwipe_core::domain::deck::ImportMode,
    ) -> BoxFuture<'a, Result<ImportDeckCardsResult, ImportDeckCardsError>>;

    /// See [`DeckService::clone_deck`].
    fn clone_deck<'a>(
        &'a self,
        request: &'a CloneDeck,
    ) -> BoxFuture<'a, Result<uuid::Uuid, CloneDeckError>>;

    /// See [`DeckService::share_deck`].
    fn share_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<uuid::Uuid, ShareDeckError>>;

    /// See [`DeckService::unshare_deck`].
    fn unshare_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<(), ShareDeckError>>;

    /// See [`DeckService::get_shared_deck`].
    fn get_shared_deck(
        &self,
        token: uuid::Uuid,
    ) -> BoxFuture<'_, Result<SharedDeck, GetSharedDeckError>>;
}

impl<T> ErasedDeckService for T
where
    T: DeckService,
{
    fn create_deck_profile<'a>(
        &'a self,
        request: &'a CreateDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, CreateDeckProfileError>> {
        Box::pin(DeckService::create_deck_profile(self, request))
    }

    fn create_deck_card<'a>(
        &'a self,
        request: &'a CreateDeckCard,
    ) -> BoxFuture<'a, Result<DeckCard, CreateDeckCardError>> {
        Box::pin(DeckService::create_deck_card(self, request))
    }

    fn get_deck_profile<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, GetDeckProfileError>> {
        Box::pin(DeckService::get_deck_profile(self, request))
    }

    fn get_deck_profiles<'a>(
        &'a self,
        request: &'a GetDeckProfiles,
    ) -> BoxFuture<'a, Result<Vec<DeckProfile>, GetDeckProfileError>> {
        Box::pin(DeckService::get_deck_profiles(self, request))
    }

    fn get_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<Deck, GetDeckError>> {
        Box::pin(DeckService::get_deck(self, request))
    }

    fn search_deck_cards<'a>(
        &'a self,
        request: &'a GetDeckProfile,
        filter: &'a CardQuery,
    ) -> BoxFuture<'a, Result<(Vec<Card>, bool), SearchDeckCardsError>> {
        Box::pin(DeckService::search_deck_cards(self, request, filter))
    }

    fn get_deck_tokens<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<Vec<Card>, GetDeckTokensError>> {
        Box::pin(DeckService::get_deck_tokens(self, request))
    }

    fn update_deck_profile<'a>(
        &'a self,
        request: &'a UpdateDeckProfile,
    ) -> BoxFuture<'a, Result<DeckProfile, UpdateDeckProfileError>> {
        Box::pin(DeckService::update_deck_profile(self, request))
    }

    fn update_deck_card<'a>(
        &'a self,
        request: &'a UpdateDeckCard,
    ) -> BoxFuture<'a, Result<DeckCard, UpdateDeckCardError>> {
        Box::pin(DeckService::update_deck_card(self, request))
    }

    fn delete_deck<'a>(
        &'a self,
        request: &'a DeleteDeck,
    ) -> BoxFuture<'a, Result<(), DeleteDeckError>> {
        Box::pin(DeckService::delete_deck(self, request))
    }

    fn delete_deck_card<'a>(
        &'a self,
        request: &'a DeleteDeckCard,
    ) -> BoxFuture<'a, Result<(), DeleteDeckCardError>> {
        Box::pin(DeckService::delete_deck_card(self, request))
    }

    fn clear_deck_suppressions<'a>(
        &'a self,
        request: &'a ClearDeckSuppressions,
    ) -> BoxFuture<'a, Result<u64, ClearDeckSuppressionsError>> {
        Box::pin(DeckService::clear_deck_suppressions(self, request))
    }

    fn skip_deck_card<'a>(
        &'a self,
        request: &'a SkipDeckCard,
    ) -> BoxFuture<'a, Result<(), SkipDeckCardError>> {
        Box::pin(DeckService::skip_deck_card(self, request))
    }

    fn unskip_deck_card<'a>(
        &'a self,
        request: &'a SkipDeckCard,
    ) -> BoxFuture<'a, Result<(), SkipDeckCardError>> {
        Box::pin(DeckService::unskip_deck_card(self, request))
    }

    fn import_deck_cards<'a>(
        &'a self,
        request: &'a ImportDeckCards,
    ) -> BoxFuture<'a, Result<ImportDeckCardsResult, ImportDeckCardsError>> {
        Box::pin(DeckService::import_deck_cards(self, request))
    }

    fn import_archidekt_deck<'a>(
        &'a self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        cards: &'a [ArchidektCard],
        board: zwipe_core::domain::deck::Board,
        email_verified: bool,
        mode: zwipe_core::domain::deck::ImportMode,
    ) -> BoxFuture<'a, Result<ImportDeckCardsResult, ImportDeckCardsError>> {
        Box::pin(DeckService::import_archidekt_deck(
            self,
            user_id,
            deck_id,
            cards,
            board,
            email_verified,
            mode,
        ))
    }

    fn clone_deck<'a>(
        &'a self,
        request: &'a CloneDeck,
    ) -> BoxFuture<'a, Result<uuid::Uuid, CloneDeckError>> {
        Box::pin(DeckService::clone_deck(self, request))
    }

    fn share_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<uuid::Uuid, ShareDeckError>> {
        Box::pin(DeckService::share_deck(self, request))
    }

    fn unshare_deck<'a>(
        &'a self,
        request: &'a GetDeckProfile,
    ) -> BoxFuture<'a, Result<(), ShareDeckError>> {
        Box::pin(DeckService::unshare_deck(self, request))
    }

    fn get_shared_deck(
        &self,
        token: uuid::Uuid,
    ) -> BoxFuture<'_, Result<SharedDeck, GetSharedDeckError>> {
        Box::pin(DeckService::get_shared_deck(self, token))
    }
}
