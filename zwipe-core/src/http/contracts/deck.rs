//! Deck management HTTP request contracts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::deck::ImportMode;
use crate::http::helpers::Opdate;

/// Request to import an Archidekt deck's cards into an existing deck.
///
/// The server extracts the numeric deck id from the URL, fetches the deck via
/// Archidekt's public JSON API, resolves each printing against the card
/// database by Scryfall id, and imports the cards into the caller's deck (the
/// target deck id comes from the URL path) exactly like the plain-text
/// importer. Responds with an `ImportDeckCardsResult`.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpImportArchidektDeck {
    /// Archidekt deck URL (e.g. `https://archidekt.com/decks/13769484/shorikai`)
    /// or a bare numeric deck id.
    pub url: String,
    /// Board to place the imported cards on. Values: `"deck"`, `"maybeboard"`,
    /// `"sideboard"`. Defaults to `"deck"` if absent.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub board: Option<String>,
    /// Add on top of the target board (default), or replace it (cards on it
    /// that aren't in the Archidekt list are removed).
    /// Values: `"add"`, `"replace"`.
    #[serde(default)]
    pub mode: ImportMode,
}

/// Deck creation request body.
#[derive(Debug, Deserialize, Serialize)]
pub struct HttpCreateDeckProfile {
    /// Deck display name.
    pub name: String,
    /// Optional commander card ID.
    pub commander_id: Option<Uuid>,
    /// Optional partner commander card ID.
    pub partner_commander_id: Option<Uuid>,
    /// Optional background enchantment card ID.
    pub background_id: Option<Uuid>,
    /// Optional signature spell card ID.
    pub signature_spell_id: Option<Uuid>,
    /// Optional deck format.
    pub format: Option<String>,
    /// Deck archetype/strategy tags (snake_case strings). Absent or empty = none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
}

impl HttpCreateDeckProfile {
    /// Creates a builder with the required name field.
    pub fn builder(name: &str) -> HttpCreateDeckProfileBuilder {
        HttpCreateDeckProfileBuilder {
            name: name.to_string(),
            commander_id: None,
            partner_commander_id: None,
            background_id: None,
            signature_spell_id: None,
            format: None,
            tags: None,
        }
    }
}

/// Builder for [`HttpCreateDeckProfile`].
pub struct HttpCreateDeckProfileBuilder {
    name: String,
    commander_id: Option<Uuid>,
    partner_commander_id: Option<Uuid>,
    background_id: Option<Uuid>,
    signature_spell_id: Option<Uuid>,
    format: Option<String>,
    tags: Option<Vec<String>>,
}

impl HttpCreateDeckProfileBuilder {
    /// Sets the commander card ID.
    pub fn commander_id(mut self, id: Option<Uuid>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander card ID.
    pub fn partner_commander_id(mut self, id: Option<Uuid>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment card ID.
    pub fn background_id(mut self, id: Option<Uuid>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell card ID.
    pub fn signature_spell_id(mut self, id: Option<Uuid>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the deck format.
    pub fn format(mut self, format: Option<String>) -> Self {
        self.format = format;
        self
    }

    /// Sets the deck tags.
    pub fn tags(mut self, tags: Option<Vec<String>>) -> Self {
        self.tags = tags;
        self
    }

    /// Builds the request.
    pub fn build(self) -> HttpCreateDeckProfile {
        HttpCreateDeckProfile {
            name: self.name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format: self.format,
            tags: self.tags,
        }
    }
}

/// Deck metadata update request body with partial update semantics.
///
/// Uses [`Opdate`] for nullable fields: absent means unchanged, `null` means set to `None`.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdateDeckProfile {
    /// New deck name, or `None` to leave unchanged.
    pub name: Option<String>,
    /// Commander card ID with partial update semantics.
    pub commander_id: Opdate<Uuid>,
    /// Partner commander card ID with partial update semantics.
    pub partner_commander_id: Opdate<Uuid>,
    /// Background enchantment card ID with partial update semantics.
    pub background_id: Opdate<Uuid>,
    /// Signature spell card ID with partial update semantics.
    pub signature_spell_id: Opdate<Uuid>,
    /// Format with partial update semantics.
    pub format: Opdate<String>,
    /// Tags with partial update semantics. `Set` replaces the full tag set
    /// (empty/`null` clears all tags); absent leaves them unchanged.
    pub tags: Opdate<Vec<String>>,
}

impl HttpUpdateDeckProfile {
    /// Creates a builder with all fields defaulting to unchanged.
    pub fn builder() -> HttpUpdateDeckProfileBuilder {
        HttpUpdateDeckProfileBuilder {
            name: None,
            commander_id: Opdate::Unchanged,
            partner_commander_id: Opdate::Unchanged,
            background_id: Opdate::Unchanged,
            signature_spell_id: Opdate::Unchanged,
            format: Opdate::Unchanged,
            tags: Opdate::Unchanged,
        }
    }
}

/// Builder for [`HttpUpdateDeckProfile`].
pub struct HttpUpdateDeckProfileBuilder {
    name: Option<String>,
    commander_id: Opdate<Uuid>,
    partner_commander_id: Opdate<Uuid>,
    background_id: Opdate<Uuid>,
    signature_spell_id: Opdate<Uuid>,
    format: Opdate<String>,
    tags: Opdate<Vec<String>>,
}

impl HttpUpdateDeckProfileBuilder {
    /// Sets the new deck name.
    pub fn name(mut self, name: Option<&str>) -> Self {
        self.name = name.map(|s| s.to_string());
        self
    }

    /// Sets the commander update.
    pub fn commander_id(mut self, id: Opdate<Uuid>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander update.
    pub fn partner_commander_id(mut self, id: Opdate<Uuid>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment update.
    pub fn background_id(mut self, id: Opdate<Uuid>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell update.
    pub fn signature_spell_id(mut self, id: Opdate<Uuid>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the format update.
    pub fn format(mut self, format: Opdate<String>) -> Self {
        self.format = format;
        self
    }

    /// Sets the tags update.
    pub fn tags(mut self, tags: Opdate<Vec<String>>) -> Self {
        self.tags = tags;
        self
    }

    /// Builds the request.
    pub fn build(self) -> HttpUpdateDeckProfile {
        HttpUpdateDeckProfile {
            name: self.name,
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format: self.format,
            tags: self.tags,
        }
    }
}

/// Deck clone request body.
///
/// The source deck id comes from the URL path; this body supplies the
/// new name for the clone. The caller is identified by the JWT.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpCloneDeck {
    /// Name for the new deck.
    pub new_name: String,
}

/// Deck clone response body.
///
/// Returns only the new deck id; the client navigates to the deck view
/// which loads the full aggregate via its own resources.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpClonedDeck {
    /// Id of the newly created clone.
    pub deck_id: Uuid,
}
