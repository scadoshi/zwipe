//! Port traits for deck building operations.
//!
//! This module defines the interfaces (ports) for deck management in hexagonal architecture.
//! Decks are collections of Magic: The Gathering cards with metadata like name, commander, and copy limits.

use std::future::Future;

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::deck_profile::DeckProfile;
use zwipe_core::domain::deck::requests::clear_deck_suppressions::ClearDeckSuppressions;
use zwipe_core::domain::deck::requests::clone_deck::CloneDeck;
use zwipe_core::domain::deck::requests::commander_maybeboard::CommanderMaybeboardCard;
use zwipe_core::domain::deck::requests::create_deck_card::CreateDeckCard;
use zwipe_core::domain::deck::requests::create_deck_profile::CreateDeckProfile;
use zwipe_core::domain::deck::requests::delete_deck::DeleteDeck;
use zwipe_core::domain::deck::requests::delete_deck_card::DeleteDeckCard;
use zwipe_core::domain::deck::requests::get_deck_profile::GetDeckProfile;
use zwipe_core::domain::deck::requests::get_deck_profiles::GetDeckProfiles;
use zwipe_core::domain::deck::requests::import_deck_cards::{
    ImportDeckCards, ImportDeckCardsResult,
};
use zwipe_core::domain::deck::requests::skip_deck_card::SkipDeckCard;
use zwipe_core::domain::deck::requests::update_deck_card::UpdateDeckCard;
use zwipe_core::domain::deck::requests::update_deck_profile::UpdateDeckProfile;
use zwipe_core::domain::deck::{Deck, DeckCard};

use crate::domain::deck::models::deck::activity::{Activity, ActivityCounts};
use crate::domain::deck::models::deck::clear_deck_suppressions::ClearDeckSuppressionsError;
use crate::domain::deck::models::deck::clone_deck::CloneDeckError;
use crate::domain::deck::models::deck::commander_maybeboard::CommanderMaybeboardError;
use crate::domain::deck::models::deck::commander_suggestions::{
    CommanderDismissal, CommanderSuggestions, CommanderSuggestionsError, MainboardCard,
    PlayStyleSummary, ProfiledDeck,
};
use crate::domain::deck::models::deck::create_deck_profile::CreateDeckProfileError;
use crate::domain::deck::models::deck::delete_deck::DeleteDeckError;
use crate::domain::deck::models::deck::get_deck::GetDeckError;
use crate::domain::deck::models::deck::get_deck_profile::GetDeckProfileError;
use crate::domain::deck::models::deck::get_deck_tokens::GetDeckTokensError;
use crate::domain::deck::models::deck::import_archidekt::ArchidektDeck;
use crate::domain::deck::models::deck::search_deck_cards::{DeckCardServe, SearchDeckCardsError};
use crate::domain::deck::models::deck::share_deck::{
    GetSharedDeckError, ShareDeckError, SharedDeck,
};
use crate::domain::deck::models::deck::skip_deck_card::SkipDeckCardError;
use crate::domain::deck::models::deck::update_deck_profile::UpdateDeckProfileError;
use crate::domain::deck::models::deck_card::create_deck_card::CreateDeckCardError;
use crate::domain::deck::models::deck_card::delete_deck_card::DeleteDeckCardError;
use crate::domain::deck::models::deck_card::get_deck_card::GetDeckCardError;
use crate::domain::deck::models::deck_card::import_deck_cards::{ImportDeckCardsError, ImportRow};
use crate::domain::deck::models::deck_card::update_deck_card::UpdateDeckCardError;

/// Database port for deck building operations.
pub trait DeckRepository: Clone + Send + Sync + 'static {
    // == create ==

    /// Creates a new deck profile. The user's row lock, the deck-count check
    /// and the insert run in one transaction, so concurrent creates can't
    /// pass `deck_limit` together. `deck_limit` is resolved by the domain; the
    /// request's `email_verified` picks the error variant.
    fn create_deck_profile(
        &self,
        request: &CreateDeckProfile,
        deck_limit: i64,
    ) -> impl Future<Output = Result<DeckProfile, CreateDeckProfileError>> + Send;

    /// Adds a card to a deck. Ownership + row lock, the card-limit check, and
    /// the insert run in one transaction (`FOR UPDATE` serializes concurrent
    /// adds, closing the count-then-insert TOCTOU). `card_limit` is resolved by
    /// the domain; the request's `email_verified` picks the error variant.
    fn create_deck_card(
        &self,
        request: &CreateDeckCard,
        card_limit: i64,
    ) -> impl Future<Output = Result<DeckCard, CreateDeckCardError>> + Send;

    // == count ==

    /// Returns the total card quantity (sum of all copies) in a deck.
    fn count_cards_in_deck(
        &self,
        deck_id: uuid::Uuid,
    ) -> impl Future<Output = Result<i64, anyhow::Error>> + Send;

    // == get ==

    /// Retrieves a single deck profile by ID.
    fn fetch_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all deck profiles for a user.
    fn fetch_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves all cards in a deck.
    fn fetch_deck_cards(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<DeckCard>, GetDeckCardError>> + Send;

    /// The distinct `card_roles` of a deck's mainboard MVPs, for the serve's
    /// `W_STEER` term. Empty when the deck has no MVPs (or none of them carry
    /// roles), which leaves serve ordering untouched. Unvested MVPs count:
    /// vesting guards the *global* signal, while steering your own deck is
    /// immediate by design.
    fn fetch_mvp_card_roles(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<String>, GetDeckCardError>> + Send;

    // == update ==

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

    // == delete ==

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

    /// Applies one import batch atomically: ownership + row lock, card-limit
    /// enforcement, bulk upsert, and (replace mode) per-board reconcile all run
    /// in a single transaction, so a failure anywhere leaves the deck exactly
    /// as it was. The `FOR UPDATE` deck lock serializes concurrent imports of
    /// the same deck, closing the limit-check TOCTOU. `card_limit` is resolved
    /// by the domain (policy stays there); `email_verified` picks
    /// `LimitReached` vs `UnverifiedLimitReached`. Bulk deletes do NOT
    /// suppress; importing a new list isn't a per-card rejection. A row whose
    /// printing the import named moves a card already in the deck to that
    /// printing; a default-printing row leaves the deck's printing alone.
    fn apply_import_batch(
        &self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        mode: zwipe_core::domain::deck::ImportMode,
        batch: &[ImportRow],
        card_limit: i64,
        email_verified: bool,
    ) -> impl Future<Output = Result<Vec<DeckCard>, ImportDeckCardsError>> + Send;

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

    /// Lists a user's commander maybeboard oracle ids, newest save first.
    fn fetch_commander_maybeboard_oracle_ids(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<uuid::Uuid>, CommanderMaybeboardError>> + Send;

    /// Adds a commander maybeboard entry (idempotent; duplicate = no-op
    /// success). The `latest_cards` existence check, the insert, and the cap
    /// check all run in one transaction under the user row lock (`FOR UPDATE`
    /// serializes concurrent adds, closing the count-then-insert TOCTOU).
    /// `cap` is resolved by the domain; policy stays there.
    fn add_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
        cap: i64,
    ) -> impl Future<Output = Result<(), CommanderMaybeboardError>> + Send;

    /// Removes a commander maybeboard entry (idempotent).
    fn remove_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
    ) -> impl Future<Output = Result<(), CommanderMaybeboardError>> + Send;

    /// Deletes a user's entire commander maybeboard, returning the number of
    /// rows removed. Idempotent.
    fn clear_commander_maybeboard(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<u64, CommanderMaybeboardError>> + Send;

    // == suggested commanders ==

    /// Every deck the user owns, in any format, that has a commander or a
    /// mainboard card, most recently updated first, each with its commander's
    /// oracle id when it has one, its colors and its mainboard oracle ids.
    fn fetch_profiled_decks(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<ProfiledDeck>, CommanderSuggestionsError>> + Send;

    /// Every mainboard card of every deck the user owns, in any format, with
    /// the printing the deck runs, its name and its type line. Rows come
    /// grouped by deck, most recently updated deck first, the same deck order
    /// as [`Self::fetch_profiled_decks`].
    fn fetch_mainboard_cards(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<MainboardCard>, CommanderSuggestionsError>> + Send;

    /// The user's daily swipes and searches from the database's UTC date
    /// back over `ACTIVITY_DAYS`, their swipe outcomes across every deck,
    /// kept and passed cards per card role and the most skipped cards never
    /// kept or saved, both from decks with a commander.
    fn fetch_activity_counts(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<ActivityCounts, CommanderSuggestionsError>> + Send;

    /// Records a dismissal, restarting its clock when one exists.
    fn dismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> impl Future<Output = Result<(), CommanderSuggestionsError>> + Send;

    /// Deletes one dismissal (idempotent).
    fn undismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> impl Future<Output = Result<(), CommanderSuggestionsError>> + Send;

    /// Deletes every dismissal of the user, returning the number removed.
    fn clear_commander_dismissals(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<u64, CommanderSuggestionsError>> + Send;

    // == clone ==

    /// Transactionally copies a source deck's profile and all entries into a
    /// new deck owned by the requesting user with the given `new_name`.
    /// Returns the new deck's id. The deck-count check against `deck_limit`
    /// runs in the same transaction under the user's row lock, as in
    /// `create_deck_profile`. The caller must have already verified the user
    /// owns `source_deck_id`: this method performs no authorization check.
    /// `share_token` is deliberately not copied: clones start private.
    fn clone_deck(
        &self,
        request: &CloneDeck,
        deck_limit: i64,
    ) -> impl Future<Output = Result<uuid::Uuid, CloneDeckError>> + Send;

    // == share ==

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
    fn fetch_deck_id_by_share_token(
        &self,
        token: uuid::Uuid,
    ) -> impl Future<Output = Result<Option<(uuid::Uuid, uuid::Uuid)>, anyhow::Error>> + Send;
}

/// Service port for deck building business logic.
pub trait DeckService: Clone + Send + Sync + 'static {
    // == create ==

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

    // == get ==

    /// Retrieves a deck profile with authorization check.
    fn fetch_deck_profile(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<DeckProfile, GetDeckProfileError>> + Send;

    /// Retrieves all decks for the requesting user.
    fn fetch_deck_profiles(
        &self,
        request: &GetDeckProfiles,
    ) -> impl Future<Output = Result<Vec<DeckProfile>, GetDeckProfileError>> + Send;

    /// Retrieves complete deck with authorization check.
    fn fetch_deck(
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
    ) -> impl Future<Output = Result<DeckCardServe, SearchDeckCardsError>> + Send;

    /// Retrieves all token cards produced by the cards in a deck.
    fn fetch_deck_tokens(
        &self,
        request: &GetDeckProfile,
    ) -> impl Future<Output = Result<Vec<Card>, GetDeckTokensError>> + Send;

    // == update ==

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

    // == delete ==

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

    /// Returns the user's commander maybeboard hydrated to full cards
    /// (preferred printing via `latest_cards`), newest save first.
    fn fetch_commander_maybeboard(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Vec<Card>, CommanderMaybeboardError>> + Send;

    /// Adds a commander maybeboard entry (idempotent, capped).
    fn add_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
    ) -> impl Future<Output = Result<(), CommanderMaybeboardError>> + Send;

    /// Removes a commander maybeboard entry (idempotent).
    fn remove_commander_maybeboard_card(
        &self,
        request: &CommanderMaybeboardCard,
    ) -> impl Future<Output = Result<(), CommanderMaybeboardError>> + Send;

    /// Clears the user's entire commander maybeboard (idempotent), returning
    /// rows removed.
    fn clear_commander_maybeboard(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<u64, CommanderMaybeboardError>> + Send;

    /// Ranks commanders for the user from the decks they have built, at most
    /// `limit`, each with the reasons behind it. Decks in every format count.
    /// A user with no deck that has a commander or a mainboard card gets an
    /// empty list and `deck_count` 0.
    fn commander_suggestions(
        &self,
        user_id: uuid::Uuid,
        limit: u32,
    ) -> impl Future<Output = Result<CommanderSuggestions, CommanderSuggestionsError>> + Send;

    /// The user's deck count and color map, from the same decks
    /// [`DeckService::commander_suggestions`] reads, without tag profiles or
    /// scoring. A user with no such deck gets `deck_count` 0 and a map of
    /// zeros.
    fn play_style_summary(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<PlayStyleSummary, CommanderSuggestionsError>> + Send;

    /// The user's activity for the Playstyle screen: the repository's
    /// counts, the roles narrowed by `activity::rank_roles`, and each hard
    /// pass with its most recent normal printing. A pass whose card the
    /// fetch misses is dropped.
    fn activity(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<Activity, CommanderSuggestionsError>> + Send;

    /// Hides a commander from the user's suggestions for a week.
    fn dismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> impl Future<Output = Result<(), CommanderSuggestionsError>> + Send;

    /// Lifts one dismissal (idempotent).
    fn undismiss_commander(
        &self,
        request: &CommanderDismissal,
    ) -> impl Future<Output = Result<(), CommanderSuggestionsError>> + Send;

    /// Lifts every dismissal of the user, returning the number lifted.
    fn clear_commander_dismissals(
        &self,
        user_id: uuid::Uuid,
    ) -> impl Future<Output = Result<u64, CommanderSuggestionsError>> + Send;

    /// Imports cards from a plain-text decklist with authorization check.
    fn import_deck_cards(
        &self,
        request: &ImportDeckCards,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ImportDeckCardsError>> + Send;

    /// Imports an Archidekt deck's cards into an existing deck owned by
    /// `user_id`, onto the given board, exactly like `import_deck_cards`,
    /// except cards resolve by Scryfall printing id (with a name fallback)
    /// instead of by name. With [`zwipe_core::domain::deck::ImportMode::Replace`],
    /// the board is made to exactly match the list. Once the cards are in, the
    /// deck's format and commander slots that are still empty take Archidekt's
    /// (see [`ProfileFill::plan`](crate::domain::deck::models::deck::import_archidekt::ProfileFill::plan)),
    /// and the result says what was set.
    fn import_archidekt_deck(
        &self,
        user_id: uuid::Uuid,
        deck_id: uuid::Uuid,
        deck: &ArchidektDeck,
        board: zwipe_core::domain::deck::Board,
        email_verified: bool,
        mode: zwipe_core::domain::deck::ImportMode,
    ) -> impl Future<Output = Result<ImportDeckCardsResult, ImportDeckCardsError>> + Send;

    // == clone ==

    /// Clones an existing deck owned by the caller into a new deck with a
    /// caller-chosen name. Performs source ownership check and deck-count
    /// limit enforcement, then delegates the transactional copy to the
    /// repository. Returns the new deck's id.
    fn clone_deck(
        &self,
        request: &CloneDeck,
    ) -> impl Future<Output = Result<uuid::Uuid, CloneDeckError>> + Send;

    // == share ==

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
    fn fetch_shared_deck(
        &self,
        token: uuid::Uuid,
    ) -> impl Future<Output = Result<SharedDeck, GetSharedDeckError>> + Send;
}
