//! SQLx type conversions for Scryfall data models.
//!
//! Two encoding strategies are used depending on the PostgreSQL column type:
//! - **JSONB via serde**: nested/complex types (`Prices`, `Legalities`, `ImageUris`,
//!   `AllParts`, `CardFaces`) serialize through `serde_json` and delegate to SQLx's
//!   `JsonValue` codec.
//! - **TEXT / TEXT[]**: flat enum types (`Colors`, `Rarity`, `Rarities`) encode as
//!   short-name strings directly into PostgreSQL `text` or `text[]` columns.

/// Maps `AllParts` and `RelatedCard` to JSONB.
pub mod all_parts;
/// Maps `CardFaces` and `CardFace` to JSONB.
pub mod card_faces;
/// Maps `Colors` to PostgreSQL `TEXT[]` using short names.
pub mod colors;
/// Maps `ImageUris` to JSONB.
pub mod image_uris;
/// Maps `Legalities` to JSONB.
pub mod legalities;
/// Maps `Prices` to JSONB.
pub mod prices;
/// Maps `Rarity` to `TEXT` and `Rarities` to `TEXT[]`.
pub mod rarity;
