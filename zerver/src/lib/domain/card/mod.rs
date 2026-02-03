//! Magic: The Gathering card data and search.
//!
//! This module manages the comprehensive card database sourced from Scryfall API,
//! providing search, retrieval, and synchronization capabilities for MTG cards.
//!
//! # Card Data Source
//!
//! All card data comes from the [Scryfall API](https://scryfall.com/docs/api),
//! which provides:
//! - Complete card database (~100 fields per card)
//! - Regular updates with new sets
//! - High-quality card images
//! - Comprehensive rulings and legality data
//!
//! # Core Operations
//!
//! ## Card Sync
//!
//! - **Bulk Download**: Fetch all cards from Scryfall bulk data endpoint
//! - **Delta Updates**: Only update changed cards based on timestamps
//! - **Batch Upsert**: Efficient database insertion respecting PostgreSQL limits
//! - **Metrics Tracking**: Record sync performance and errors
//!
//! ## Card Search
//!
//! Comprehensive filtering by:
//! - **Text**: Name, oracle text, flavor text, type line
//! - **Combat**: Power, toughness (exact or range)
//! - **Mana**: CMC, color identity
//! - **Metadata**: Rarity, set, artist, language
//! - **Flags**: Commander-legal, tokens, digital-only, promo
//! - **Pagination**: Limit, offset, ordering
//!
//! ## Card Retrieval
//!
//! - Get single card by ID
//! - Get multiple cards (batch)
//! - Get card with profile (user-specific data)
//! - List distinct values (artists, sets, languages, types)
//!
//! # Data Model
//!
//! - **ScryfallData**: Complete card data from Scryfall (~100 fields)
//! - **CardProfile**: User-specific card metadata (favorites, notes - future)
//! - **Card**: Composite of ScryfallData + CardProfile
//!
//! # Example
//!
//! ```rust,ignore
//! use zwipe::domain::card::models::search_card::CardFilter;
//!
//! // Search for blue creatures with flying
//! let filter = CardFilter::builder()
//!     .color_identity(vec!["U"])
//!     .oracle_text_contains("flying")
//!     .type_line_contains("Creature")
//!     .limit(20)
//!     .build();
//!
//! let cards = card_service.search_cards(filter).await?;
//!
//! // Get specific card
//! let card = card_service.get_card(card_id).await?;
//! println!("{} - {}", card.name, card.mana_cost);
//! ```

/// Card models and value objects (ScryfallData, CardProfile, Card, filters).
pub mod models;

/// Port traits (interfaces) for card operations.
#[cfg(feature = "zerver")]
pub mod ports;

/// Service implementations for card business logic.
#[cfg(feature = "zerver")]
pub mod services;
