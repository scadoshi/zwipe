//! Magic: The Gathering card data and search.
//!
//! This module manages the card database sourced from Scryfall API,
//! providing search, retrieval, and synchronization capabilities for MTG cards.
//!
//! # Card Data Source
//!
//! All card data comes from the [Scryfall API](https://scryfall.com/docs/api).

/// Card models and value objects (ScryfallData, CardProfile, Card, filters).
pub mod models;
pub use models::{helpers, search_card, synergy, zervice_metrics};

/// Card request types (get, create, search operations).
pub mod requests;
pub use requests::{
    create_card, get_artists, get_card, get_card_profile, get_card_types, get_keywords,
    get_languages, get_oracle_tags, get_oracle_words, get_scryfall_data, get_sets,
};

/// Port traits (interfaces) for card operations.
pub mod ports;

/// Service implementations for card business logic.
pub mod services;
