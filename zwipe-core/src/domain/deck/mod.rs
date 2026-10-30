//! Decks, their cards, tags and draw odds.

pub mod draw_odds;
pub mod models;
pub mod requests;
pub mod tag_profile;

pub use models::{
    Board, Deck, DeckCard, DeckCommandZone, DeckEntry, DeckMetrics, DeckName, DeckOtherTag,
    DeckProfile, DeckTag, DeckTagView, DeckWarning, Format, ImportMode, InvalidBoard,
    InvalidDeckOtherTag, InvalidDeckTag, InvalidDeckname, InvalidFormat, InvalidPowerLevel,
    InvalidQuantity, MAX_DECK_ORACLE_TAGS, MAX_DECK_OTHER_TAGS, MAX_DECK_TAGS, PowerLevel,
    Quantity, WarningAction, board, deck, deck_card, deck_metrics, deck_name, deck_oracle_tags,
    deck_other_tag, deck_profile, deck_tag, deck_tag_label, deck_warning, dedupe_oracle_tags,
    format, import_mode, power_level, quantity, seed_oracle_tags, seed_oracle_tags_from_catalog,
    validate_deck,
};
pub use requests::{
    ClearDeckSuppressions, CloneDeck, CommanderMaybeboardCard, CreateDeckCard, CreateDeckProfile,
    DeleteDeck, DeleteDeckCard, GetDeckProfile, GetDeckProfiles, ImportDeckCards,
    ImportDeckCardsResult, ImportLine, ImportedCard, InvalidClearDeckSuppressions,
    InvalidCloneDeck, InvalidCommanderMaybeboardCard, InvalidCreateDeckCard,
    InvalidCreateDeckProfile, InvalidDeleteDeck, InvalidDeleteDeckCard, InvalidGetDeckCard,
    InvalidGetDeckProfiles, InvalidSkipDeckCard, InvalidUpdateDeckCard, InvalidUpdateDeckProfile,
    SkipDeckCard, UnresolvedCard, UpdateDeckCard, UpdateDeckProfile, clear_deck_suppressions,
    clone_deck, commander_maybeboard, create_deck_card, create_deck_profile, delete_deck,
    delete_deck_card, get_deck_card, get_deck_profile, get_deck_profiles, import_deck_cards,
    skip_deck_card, update_deck_card, update_deck_profile,
};
