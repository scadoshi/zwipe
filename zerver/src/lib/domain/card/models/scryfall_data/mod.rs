/// Multi-part card relationships (split cards, double-faced cards, etc.).
pub mod all_parts;
/// Card face data for double-faced/transform/modal cards.
pub mod card_faces;
/// Color and color identity types and conversions.
pub mod colors;
/// Card image URIs at various resolutions.
pub mod image_uris;
/// Format legality status (Standard, Modern, Commander, etc.).
pub mod legalities;
/// Card pricing data from various sources.
pub mod prices;
/// Rarity types (Common, Uncommon, Rare, Mythic).
pub mod rarity;

pub use zwipe_core::domain::card::scryfall_data::ScryfallData;
