//! Deck management HTTP request contracts.

use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::{
    domain::{
        card::{Card, search_card::card_filter::price_currency::PriceCurrency},
        deck::{DeckEntry, DeckOtherTag, ImportMode, PowerLevel, format::Format},
    },
    http::helpers::Opdate,
};

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
    /// Power level (snake_case bracket string). Absent = unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_level: Option<String>,
    /// Secondary, non-gameplay labels (snake_case strings). Absent or empty = none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub other_tags: Option<Vec<String>>,
    /// Granular oracle-tag slugs the deck declares. Absent or empty = none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub oracle_tags: Option<Vec<String>>,
    /// User-set land target. Absent = use the format-derived heuristic.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub land_target: Option<i32>,
    /// User-set deck price target (budget). Absent = no budget.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_target: Option<f64>,
    /// Currency for the price target. Absent = USD.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub price_target_currency: Option<PriceCurrency>,
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
            power_level: None,
            other_tags: None,
            oracle_tags: None,
            land_target: None,
            price_target: None,
            price_target_currency: None,
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
    power_level: Option<String>,
    other_tags: Option<Vec<String>>,
    oracle_tags: Option<Vec<String>>,
    land_target: Option<i32>,
    price_target: Option<f64>,
    price_target_currency: Option<PriceCurrency>,
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

    /// Sets the power level.
    pub fn power_level(mut self, power_level: Option<String>) -> Self {
        self.power_level = power_level;
        self
    }

    /// Sets the other-tags.
    pub fn other_tags(mut self, other_tags: Option<Vec<String>>) -> Self {
        self.other_tags = other_tags;
        self
    }

    /// Sets the oracle-tags.
    pub fn oracle_tags(mut self, oracle_tags: Option<Vec<String>>) -> Self {
        self.oracle_tags = oracle_tags;
        self
    }

    /// Sets the land target.
    pub fn land_target(mut self, land_target: Option<i32>) -> Self {
        self.land_target = land_target;
        self
    }

    /// Sets the price target (budget).
    pub fn price_target(mut self, price_target: Option<f64>) -> Self {
        self.price_target = price_target;
        self
    }

    /// Sets the price target currency.
    pub fn price_target_currency(mut self, price_target_currency: Option<PriceCurrency>) -> Self {
        self.price_target_currency = price_target_currency;
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
            power_level: self.power_level,
            other_tags: self.other_tags,
            oracle_tags: self.oracle_tags,
            land_target: self.land_target,
            price_target: self.price_target,
            price_target_currency: self.price_target_currency,
        }
    }
}

/// Deck metadata update request body with partial update semantics.
///
/// Uses [`Opdate`] for nullable fields — absent means unchanged, `null`
/// means clear, a bare value means set (the Opdate custom serde impls make
/// this real; every field carries `default` + `skip_serializing_if` per the
/// Opdate contract). `name` is the one non-clearable field: a deck always
/// has a name, so explicit `null` there is a 422 (RFC 7396 resolution),
/// enforced by the handler.
#[derive(Debug, Serialize, Deserialize)]
pub struct HttpUpdateDeckProfile {
    /// New deck name; absent = leave unchanged. Non-clearable: `null` is
    /// rejected by the handler rather than ignored.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub name: Opdate<String>,
    /// Commander card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub commander_id: Opdate<Uuid>,
    /// Partner commander card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub partner_commander_id: Opdate<Uuid>,
    /// Background enchantment card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub background_id: Opdate<Uuid>,
    /// Signature spell card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub signature_spell_id: Opdate<Uuid>,
    /// Format with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub format: Opdate<String>,
    /// Tags with partial update semantics. `Set` replaces the full tag set
    /// (empty/`null` clears all tags); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub tags: Opdate<Vec<String>>,
    /// Power level with partial update semantics. `Set(None)` clears it;
    /// absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub power_level: Opdate<String>,
    /// Other-tags with partial update semantics. `Set` replaces the full set
    /// (empty/`null` clears all); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub other_tags: Opdate<Vec<String>>,
    /// Oracle-tags with partial update semantics. `Set` replaces the full set
    /// (empty/`null` clears all); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub oracle_tags: Opdate<Vec<String>>,
    /// Land target with partial update semantics. `Set(None)` clears the
    /// override (back to the format heuristic); absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub land_target: Opdate<i32>,
    /// Price target with partial update semantics. `Set(None)` clears the
    /// budget; absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub price_target: Opdate<f64>,
    /// Price target currency with partial update semantics.
    #[serde(default, skip_serializing_if = "Opdate::is_unchanged")]
    pub price_target_currency: Opdate<PriceCurrency>,
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
            power_level: Opdate::Unchanged,
            other_tags: Opdate::Unchanged,
            oracle_tags: Opdate::Unchanged,
            land_target: Opdate::Unchanged,
            price_target: Opdate::Unchanged,
            price_target_currency: Opdate::Unchanged,
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
    power_level: Opdate<String>,
    other_tags: Opdate<Vec<String>>,
    oracle_tags: Opdate<Vec<String>>,
    land_target: Opdate<i32>,
    price_target: Opdate<f64>,
    price_target_currency: Opdate<PriceCurrency>,
}

impl HttpUpdateDeckProfileBuilder {
    /// Sets the new deck name. `None` leaves the name unchanged; the builder
    /// cannot express "clear" because a name is non-clearable.
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

    /// Sets the power level update.
    pub fn power_level(mut self, power_level: Opdate<String>) -> Self {
        self.power_level = power_level;
        self
    }

    /// Sets the other-tags update.
    pub fn other_tags(mut self, other_tags: Opdate<Vec<String>>) -> Self {
        self.other_tags = other_tags;
        self
    }

    /// Sets the oracle-tags update.
    pub fn oracle_tags(mut self, oracle_tags: Opdate<Vec<String>>) -> Self {
        self.oracle_tags = oracle_tags;
        self
    }

    /// Sets the land target update.
    pub fn land_target(mut self, land_target: Opdate<i32>) -> Self {
        self.land_target = land_target;
        self
    }

    /// Sets the price target update.
    pub fn price_target(mut self, price_target: Opdate<f64>) -> Self {
        self.price_target = price_target;
        self
    }

    /// Sets the price target currency update.
    pub fn price_target_currency(mut self, price_target_currency: Opdate<PriceCurrency>) -> Self {
        self.price_target_currency = price_target_currency;
        self
    }

    /// Builds the request.
    pub fn build(self) -> HttpUpdateDeckProfile {
        HttpUpdateDeckProfile {
            name: self
                .name
                .map_or(Opdate::Unchanged, |n| Opdate::Set(Some(n))),
            commander_id: self.commander_id,
            partner_commander_id: self.partner_commander_id,
            background_id: self.background_id,
            signature_spell_id: self.signature_spell_id,
            format: self.format,
            tags: self.tags,
            power_level: self.power_level,
            other_tags: self.other_tags,
            oracle_tags: self.oracle_tags,
            land_target: self.land_target,
            price_target: self.price_target,
            price_target_currency: self.price_target_currency,
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

/// Clear-suppressions response body.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpClearedSuppressions {
    /// Number of suppression rows removed.
    pub cleared: u64,
}

/// Skip-card request body (POST a single durable suppression).
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpSkipDeckCard {
    /// Oracle id of the card to suppress for this deck.
    pub oracle_id: Uuid,
}

/// Share-deck response body (POST `/api/deck/{deck_id}/share`).
///
/// The token is the capability: the client builds the public URL from it.
/// Re-sharing regenerates the token, so old links die.
#[derive(Debug, Clone, Deserialize, Serialize)]
pub struct HttpDeckShareToken {
    /// The deck's live share token.
    pub share_token: Uuid,
}

/// Public shared-deck response body (GET `/api/share/deck/{token}`).
///
/// Deliberately carries **no user identity** — no username, user id, or
/// email. The page shows a deck, not an account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HttpSharedDeck {
    /// Deck display name.
    pub name: String,
    /// Deck format, if set.
    pub format: Option<Format>,
    /// Power level (WotC bracket), if set.
    #[serde(default)]
    pub power_level: Option<PowerLevel>,
    /// Deck archetype/strategy tags, as server-delivered **slugs** (`Vec<String>`)
    /// so a server-added tag shows on a shared deck without a client release; labels
    /// via `deck_tag_label`. `#[serde(default)]`.
    #[serde(default)]
    pub tags: Vec<String>,
    /// Secondary, non-gameplay labels (Budget, Jank, …). Lossy-deserialized.
    #[serde(default, deserialize_with = "crate::serde_helpers::lossy_vec")]
    pub other_tags: Vec<DeckOtherTag>,
    /// Granular oracle-tag slugs the deck declares.
    #[serde(default)]
    pub oracle_tags: Vec<String>,
    /// Commander card, if set.
    pub commander: Option<Card>,
    /// Partner commander card, if set.
    pub partner_commander: Option<Card>,
    /// Background enchantment card, if set.
    pub background: Option<Card>,
    /// Signature spell card, if set.
    pub signature_spell: Option<Card>,
    /// Card entries, same payload shape as the private deck GET.
    pub entries: Vec<DeckEntry>,
    /// Token cards the deck's cards produce, derived server-side (same as the
    /// app's token list). `#[serde(default)]` so payloads predating it parse.
    #[serde(default)]
    pub tokens: Vec<Card>,
    /// Deck land target, if set — the share page shows Lands as
    /// `actual / target` like the app. `#[serde(default)]` for old payloads.
    #[serde(default)]
    pub land_target: Option<i32>,
    /// Deck price target (budget) in `price_target_currency`, if set.
    #[serde(default)]
    pub price_target: Option<f64>,
    /// The price target's own stored currency (targets never convert).
    #[serde(default)]
    pub price_target_currency: Option<PriceCurrency>,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn create_profile_defaults_land_and_price_target_to_none_when_omitted() {
        // A client predating these fields sends none of them; it must still parse.
        let json = r#"{"name":"My Deck"}"#;
        let req: HttpCreateDeckProfile = serde_json::from_str(json).unwrap();
        assert_eq!(req.land_target, None);
        assert_eq!(req.price_target, None);
        assert_eq!(req.price_target_currency, None);
    }

    #[test]
    fn update_profile_defaults_land_target_to_unchanged_when_omitted() {
        // A body touching other fields but not land_target must deserialize
        // land_target (and friends) to Unchanged rather than erroring on a
        // missing field.
        let json = r#"{
            "name": "Renamed",
            "format": "commander",
            "tags": ["aggro"]
        }"#;
        let req: HttpUpdateDeckProfile = serde_json::from_str(json).unwrap();
        assert!(req.land_target.is_unchanged());
        assert!(req.price_target.is_unchanged());
        assert!(req.price_target_currency.is_unchanged());
    }

    #[test]
    fn update_profile_clean_wire() {
        // Clean dialect (1.7.5+): a name-only rename carries exactly one key
        // — no "Unchanged" strings, no {"Set": ...} wrappers.
        let body = HttpUpdateDeckProfile::builder()
            .name(Some("j08-b1"))
            .build();
        assert_eq!(
            serde_json::to_string(&body).unwrap(),
            r#"{"name":"j08-b1"}"#
        );

        // Inbound: null = clear, bare value = set, absent = unchanged.
        let json = r#"{"commander_id":null,"land_target":33}"#;
        let req: HttpUpdateDeckProfile = serde_json::from_str(json).unwrap();
        assert_eq!(req.commander_id, Opdate::Set(None));
        assert_eq!(req.land_target, Opdate::Set(Some(33)));
        assert!(req.format.is_unchanged());
        assert!(req.name.is_unchanged());

        // A null name decodes as Set(None) — the handler's 422 signal.
        let req: HttpUpdateDeckProfile = serde_json::from_str(r#"{"name":null}"#).unwrap();
        assert_eq!(req.name, Opdate::Set(None));
    }
}
