//! Archidekt deck import operation — server-side types.
//!
//! The Archidekt outbound adapter reduces a fetched deck to a list of
//! [`ArchidektCard`]s, which the deck service imports into an existing deck
//! exactly like a plain-text import — same boards, same add/replace modes,
//! same result shape. Card *data* is never taken from Archidekt — only
//! identity (Scryfall printing id), quantity, and name.

#[cfg(feature = "zerver")]
use uuid::Uuid;

/// A single card entry from an Archidekt deck.
#[cfg(feature = "zerver")]
#[derive(Debug, Clone)]
pub struct ArchidektCard {
    /// Scryfall printing id (`card.uid`), resolved against `scryfall_data.id`.
    /// `Uuid::nil()` if Archidekt gave an unparseable id — surfaces as unresolved.
    pub scryfall_id: Uuid,
    /// Card name (Archidekt's oracle name), used for the name fallback and the
    /// unresolved report.
    pub name: String,
    /// Number of copies.
    pub quantity: i32,
}
