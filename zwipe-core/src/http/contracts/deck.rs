//! Deck management HTTP request contracts.

use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::card::scryfall_data::colors::{Color, identity_of};
use crate::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use crate::domain::deck::deck_metrics::{DeckMetrics, deck_price};
use crate::domain::deck::format::Format;
use crate::domain::deck::{Board, DeckEntry, DeckOtherTag, ImportMode, PowerLevel};
use crate::domain::user::theme::ThemeConfig;
use crate::http::helpers::Patch;

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
#[derive(Debug, Clone, Deserialize, Serialize)]
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
    /// Deck archetype/strategy tags (`snake_case` strings). Absent or empty = none.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub tags: Option<Vec<String>>,
    /// Power level (`snake_case` bracket string). Absent = unset.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub power_level: Option<String>,
    /// Secondary, non-gameplay labels (`snake_case` strings). Absent or empty = none.
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
#[derive(Debug)]
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
    #[must_use]
    pub fn commander_id(mut self, id: Option<Uuid>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander card ID.
    #[must_use]
    pub fn partner_commander_id(mut self, id: Option<Uuid>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment card ID.
    #[must_use]
    pub fn background_id(mut self, id: Option<Uuid>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell card ID.
    #[must_use]
    pub fn signature_spell_id(mut self, id: Option<Uuid>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the deck format.
    #[must_use]
    pub fn format(mut self, format: Option<String>) -> Self {
        self.format = format;
        self
    }

    /// Sets the deck tags.
    #[must_use]
    pub fn tags(mut self, tags: Option<Vec<String>>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the power level.
    #[must_use]
    pub fn power_level(mut self, power_level: Option<String>) -> Self {
        self.power_level = power_level;
        self
    }

    /// Sets the other-tags.
    #[must_use]
    pub fn other_tags(mut self, other_tags: Option<Vec<String>>) -> Self {
        self.other_tags = other_tags;
        self
    }

    /// Sets the oracle-tags.
    #[must_use]
    pub fn oracle_tags(mut self, oracle_tags: Option<Vec<String>>) -> Self {
        self.oracle_tags = oracle_tags;
        self
    }

    /// Sets the land target.
    #[must_use]
    pub fn land_target(mut self, land_target: Option<i32>) -> Self {
        self.land_target = land_target;
        self
    }

    /// Sets the price target (budget).
    #[must_use]
    pub fn price_target(mut self, price_target: Option<f64>) -> Self {
        self.price_target = price_target;
        self
    }

    /// Sets the price target currency.
    #[must_use]
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
/// Uses [`Patch`] for nullable fields: absent means unchanged, `null`
/// means clear, a bare value means set (the Patch custom serde impls make
/// this real; every field carries `default` + `skip_serializing_if` per the
/// Patch contract). `name` is the one non-clearable field: a deck always
/// has a name, so explicit `null` there is a 422 (RFC 7396 resolution),
/// enforced by the handler.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct HttpUpdateDeckProfile {
    /// New deck name; absent = leave unchanged. Non-clearable: `null` is
    /// rejected by the handler rather than ignored.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub name: Patch<String>,
    /// Commander card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub commander_id: Patch<Uuid>,
    /// Partner commander card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub partner_commander_id: Patch<Uuid>,
    /// Background enchantment card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub background_id: Patch<Uuid>,
    /// Signature spell card ID with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub signature_spell_id: Patch<Uuid>,
    /// Format with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub format: Patch<String>,
    /// Tags with partial update semantics. `Set` replaces the full tag set
    /// (empty/`null` clears all tags); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub tags: Patch<Vec<String>>,
    /// Power level with partial update semantics. `Set(None)` clears it;
    /// absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub power_level: Patch<String>,
    /// Other-tags with partial update semantics. `Set` replaces the full set
    /// (empty/`null` clears all); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub other_tags: Patch<Vec<String>>,
    /// Oracle-tags with partial update semantics. `Set` replaces the full set
    /// (empty/`null` clears all); absent leaves them unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub oracle_tags: Patch<Vec<String>>,
    /// Land target with partial update semantics. `Set(None)` clears the
    /// override (back to the format heuristic); absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub land_target: Patch<i32>,
    /// Price target with partial update semantics. `Set(None)` clears the
    /// budget; absent leaves it unchanged.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub price_target: Patch<f64>,
    /// Price target currency with partial update semantics.
    #[serde(default, skip_serializing_if = "Patch::is_unchanged")]
    pub price_target_currency: Patch<PriceCurrency>,
}

impl HttpUpdateDeckProfile {
    /// Creates a builder with all fields defaulting to unchanged.
    pub fn builder() -> HttpUpdateDeckProfileBuilder {
        HttpUpdateDeckProfileBuilder {
            name: None,
            commander_id: Patch::Unchanged,
            partner_commander_id: Patch::Unchanged,
            background_id: Patch::Unchanged,
            signature_spell_id: Patch::Unchanged,
            format: Patch::Unchanged,
            tags: Patch::Unchanged,
            power_level: Patch::Unchanged,
            other_tags: Patch::Unchanged,
            oracle_tags: Patch::Unchanged,
            land_target: Patch::Unchanged,
            price_target: Patch::Unchanged,
            price_target_currency: Patch::Unchanged,
        }
    }
}

/// Builder for [`HttpUpdateDeckProfile`].
#[derive(Debug)]
pub struct HttpUpdateDeckProfileBuilder {
    name: Option<String>,
    commander_id: Patch<Uuid>,
    partner_commander_id: Patch<Uuid>,
    background_id: Patch<Uuid>,
    signature_spell_id: Patch<Uuid>,
    format: Patch<String>,
    tags: Patch<Vec<String>>,
    power_level: Patch<String>,
    other_tags: Patch<Vec<String>>,
    oracle_tags: Patch<Vec<String>>,
    land_target: Patch<i32>,
    price_target: Patch<f64>,
    price_target_currency: Patch<PriceCurrency>,
}

impl HttpUpdateDeckProfileBuilder {
    /// Sets the new deck name. `None` leaves the name unchanged; the builder
    /// cannot express "clear" because a name is non-clearable.
    #[must_use]
    pub fn name(mut self, name: Option<&str>) -> Self {
        self.name = name.map(ToString::to_string);
        self
    }

    /// Sets the commander update.
    #[must_use]
    pub fn commander_id(mut self, id: Patch<Uuid>) -> Self {
        self.commander_id = id;
        self
    }

    /// Sets the partner commander update.
    #[must_use]
    pub fn partner_commander_id(mut self, id: Patch<Uuid>) -> Self {
        self.partner_commander_id = id;
        self
    }

    /// Sets the background enchantment update.
    #[must_use]
    pub fn background_id(mut self, id: Patch<Uuid>) -> Self {
        self.background_id = id;
        self
    }

    /// Sets the signature spell update.
    #[must_use]
    pub fn signature_spell_id(mut self, id: Patch<Uuid>) -> Self {
        self.signature_spell_id = id;
        self
    }

    /// Sets the format update.
    #[must_use]
    pub fn format(mut self, format: Patch<String>) -> Self {
        self.format = format;
        self
    }

    /// Sets the tags update.
    #[must_use]
    pub fn tags(mut self, tags: Patch<Vec<String>>) -> Self {
        self.tags = tags;
        self
    }

    /// Sets the power level update.
    #[must_use]
    pub fn power_level(mut self, power_level: Patch<String>) -> Self {
        self.power_level = power_level;
        self
    }

    /// Sets the other-tags update.
    #[must_use]
    pub fn other_tags(mut self, other_tags: Patch<Vec<String>>) -> Self {
        self.other_tags = other_tags;
        self
    }

    /// Sets the oracle-tags update.
    #[must_use]
    pub fn oracle_tags(mut self, oracle_tags: Patch<Vec<String>>) -> Self {
        self.oracle_tags = oracle_tags;
        self
    }

    /// Sets the land target update.
    #[must_use]
    pub fn land_target(mut self, land_target: Patch<i32>) -> Self {
        self.land_target = land_target;
        self
    }

    /// Sets the price target update.
    #[must_use]
    pub fn price_target(mut self, price_target: Patch<f64>) -> Self {
        self.price_target = price_target;
        self
    }

    /// Sets the price target currency update.
    #[must_use]
    pub fn price_target_currency(mut self, price_target_currency: Patch<PriceCurrency>) -> Self {
        self.price_target_currency = price_target_currency;
        self
    }

    /// Builds the request.
    pub fn build(self) -> HttpUpdateDeckProfile {
        HttpUpdateDeckProfile {
            name: self.name.map_or(Patch::Unchanged, |n| Patch::Set(Some(n))),
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
/// Of the owner's identity it carries only the username, as [`Self::owner`]:
/// Scotty made that public on shared decks on 2026-11-11, in the link and
/// as "by <username>". No user id, email or anything else about the
/// account.
#[derive(Debug, Clone, Deserialize, Serialize, PartialEq)]
pub struct HttpSharedDeck {
    /// Deck display name.
    pub name: String,
    /// Deck format, if set.
    pub format: Option<Format>,
    /// Power level (official Commander Bracket), if set.
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
    /// Deck land target, if set; the share page shows Lands as
    /// `actual / target` like the app. `#[serde(default)]` for old payloads.
    #[serde(default)]
    pub land_target: Option<i32>,
    /// Deck price target (budget) in `price_target_currency`, if set.
    #[serde(default)]
    pub price_target: Option<f64>,
    /// The price target's own stored currency (targets never convert).
    #[serde(default)]
    pub price_target_currency: Option<PriceCurrency>,
    /// When the deck last changed, cards included. zwipe.net versions the
    /// preview image's URL with it, so an edited deck gets a fresh image.
    /// `#[serde(default)]` for payloads predating it.
    #[serde(default)]
    pub updated_at: Option<DateTime<Utc>>,
    /// The owner's theme, which the preview image is drawn in. A display
    /// setting, not an identity. `#[serde(default)]` for payloads predating it.
    #[serde(default)]
    pub theme: Option<ThemeConfig>,
    /// The owner's username, for the link's first segment and the page's
    /// "by <username>". `#[serde(default)]`: a server that predates it sends
    /// none, and the page then shows no byline and keeps its link.
    #[serde(default)]
    pub owner: Option<String>,
}

impl HttpSharedDeck {
    /// The command zone's cards in slot order: commander (or oathbreaker),
    /// partner, background, signature spell.
    #[must_use]
    pub fn zone_cards(&self) -> Vec<&Card> {
        [
            &self.commander,
            &self.partner_commander,
            &self.background,
            &self.signature_spell,
        ]
        .into_iter()
        .flatten()
        .collect()
    }

    /// The main board's entries without the command zone's cards, which the
    /// share page shows in their own strip rather than in the groups.
    #[must_use]
    pub fn main_entries(&self) -> Vec<&DeckEntry> {
        let zone: Vec<Uuid> = self
            .zone_cards()
            .into_iter()
            .filter_map(|card| card.scryfall_data.oracle_id)
            .collect();
        self.entries
            .iter()
            .filter(|e| e.deck_card.board == Board::Deck && !zone.contains(&e.deck_card.oracle_id))
            .collect()
    }

    /// The share page's card count: the main board without the command
    /// zone's cards, plus one for each filled command zone slot (the
    /// commander only in a format that has one). Saturates on a corrupt
    /// quantity rather than wrapping.
    #[must_use]
    pub fn card_count(&self) -> i64 {
        let main: i64 = self
            .main_entries()
            .iter()
            .map(|e| i64::from(*e.deck_card.quantity))
            .fold(0, i64::saturating_add);
        let slots = [
            self.format.is_some_and(|f| f.has_commander()) && self.commander.is_some(),
            self.partner_commander.is_some(),
            self.background.is_some(),
            self.signature_spell.is_some(),
        ]
        .into_iter()
        .filter(|filled| *filled)
        .count();
        i64::try_from(slots).map_or(main, |slots| main.saturating_add(slots))
    }

    /// The deck's price in US dollars, as the share page totals it: the main
    /// board plus the command zone at each card's chosen printing. A card
    /// with no USD price adds nothing.
    #[must_use]
    pub fn price_usd(&self) -> f64 {
        let zone: Vec<Card> = self.zone_cards().into_iter().cloned().collect();
        deck_price(&self.entries, &zone, PriceCurrency::Usd)
    }

    /// The deck's metrics as the share page computes them, the command zone
    /// counted but kept out of the distributions.
    #[must_use]
    pub fn metrics(&self) -> DeckMetrics {
        let zone: Vec<Card> = self.zone_cards().into_iter().cloned().collect();
        DeckMetrics::from_entries_and_command_zone(&self.entries, &zone)
    }

    /// The deck's colors in WUBRG order: the command zone's combined color
    /// identity when it holds a card, else the main board's. Empty is
    /// colorless.
    #[must_use]
    pub fn colors(&self) -> Vec<Color> {
        let zone = self.zone_cards();
        let colors: Vec<Color> = if zone.is_empty() {
            self.main_entries()
                .iter()
                .flat_map(|e| e.card.scryfall_data.color_identity.iter().copied())
                .collect()
        } else {
            zone.iter()
                .flat_map(|card| card.scryfall_data.color_identity.iter().copied())
                .collect()
        };
        identity_of(&colors)
    }
}

/// Deck-aware card search response.
///
/// The result carries one fact about itself beyond the cards: whether synergy
/// ordering was actually applied. A cold commander cache serves the full pool
/// in default order instead, and the app says so.
///
/// The flag rides in the body rather than a header because a browser cannot
/// read a custom header, and a body keeps this endpoint on the `Endpoint`
/// trait with no hand-written transport.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct HttpDeckCardSearch {
    /// The cards, in serve order.
    pub cards: Vec<Card>,
    /// False when synergy was requested but the commander's cache was still
    /// warming, so these are the full pool rather than a synergy ordering.
    pub synergy_applied: bool,
    /// Index into `cards` where related cards begin: the synergy pool ran out
    /// and the page continues with cards like the deck. Absent when every card
    /// is from the pool; older clients ignore it and keep swiping.
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub related_from: Option<usize>,
}

#[cfg(test)]
mod tests {
    use super::*;

    /// A shared deck as an older server sent it: no `updated_at`, no `theme`.
    const OLD_SHARED_DECK: &str = r#"{"name":"Krenko","format":"commander","commander":null,"partner_commander":null,"background":null,"signature_spell":null,"entries":[]}"#;

    fn shared(format: Option<Format>) -> HttpSharedDeck {
        let mut deck: HttpSharedDeck = serde_json::from_str(OLD_SHARED_DECK).unwrap();
        deck.format = format;
        deck
    }

    fn priced(name: &str, usd: Option<&str>, colors: Vec<Color>) -> Card {
        let mut card = crate::test_utils::make_card(name);
        card.scryfall_data.prices.usd = usd.map(str::to_owned);
        card.scryfall_data.color_identity = colors.into();
        card
    }

    fn entry(card: Card, qty: i32, board: Board) -> DeckEntry {
        let mut e = crate::test_utils::make_entry(&card.scryfall_data.name, qty);
        e.deck_card.scryfall_data_id = card.scryfall_data.id;
        e.deck_card.oracle_id = card.scryfall_data.oracle_id.unwrap();
        e.deck_card.board = board;
        e.card = card;
        e
    }

    #[test]
    fn the_count_and_price_match_the_share_page() {
        let mut deck = shared(Some(Format::Commander));
        let commander = priced("Krenko", Some("2.50"), vec![Color::Red]);
        deck.entries = vec![
            entry(commander.clone(), 1, Board::Deck),
            entry(
                priced("Goblin", Some("0.25"), vec![Color::Red]),
                4,
                Board::Deck,
            ),
            entry(priced("Unpriced", None, vec![]), 2, Board::Deck),
            entry(
                priced("Maybe", Some("100.00"), vec![Color::Blue]),
                1,
                Board::Maybeboard,
            ),
        ];
        deck.commander = Some(commander);
        assert_eq!(
            deck.main_entries().len(),
            2,
            "the commander and the maybeboard leave"
        );
        assert_eq!(
            deck.card_count(),
            7,
            "six main cards plus the commander's slot"
        );
        assert!(
            (deck.price_usd() - 3.5).abs() < 1e-9,
            "{}",
            deck.price_usd()
        );
        assert_eq!(
            deck.colors(),
            vec![Color::Red],
            "the command zone's identity"
        );
    }

    #[test]
    fn a_deck_without_a_command_zone_takes_its_colors_from_its_cards() {
        let mut deck = shared(Some(Format::Modern));
        deck.entries = vec![
            entry(
                priced("Bolt", Some("1.00"), vec![Color::Red]),
                4,
                Board::Deck,
            ),
            entry(
                priced("Snapcaster", Some("10.00"), vec![Color::Blue]),
                4,
                Board::Deck,
            ),
            entry(
                priced("Side", None, vec![Color::Green]),
                1,
                Board::Sideboard,
            ),
        ];
        assert_eq!(deck.card_count(), 8);
        assert_eq!(
            deck.colors(),
            vec![Color::Blue, Color::Red],
            "WUBRG order, sideboard out"
        );
        assert_eq!(deck.zone_cards(), Vec::<&Card>::new());
    }

    #[test]
    fn a_shared_deck_from_an_older_server_still_parses() {
        let deck: HttpSharedDeck = serde_json::from_str(OLD_SHARED_DECK).unwrap();
        assert_eq!(deck.name, "Krenko");
        assert_eq!(deck.updated_at, None);
        assert_eq!(deck.theme, None);
    }

    #[test]
    fn a_shared_deck_round_trips_its_version_and_theme() {
        let mut deck: HttpSharedDeck = serde_json::from_str(OLD_SHARED_DECK).unwrap();
        deck.updated_at = DateTime::from_timestamp(1_762_560_000, 0);
        deck.theme = Some(ThemeConfig {
            name: "dracula".to_owned(),
            is_dark: false,
        });
        let json = serde_json::to_string(&deck).unwrap();
        assert!(
            json.contains(r#""theme":{"name":"dracula","is_dark":false}"#),
            "{json}"
        );
        let back: HttpSharedDeck = serde_json::from_str(&json).unwrap();
        assert_eq!(back, deck);
    }

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
        // (no "Unchanged" strings, no {"Set": ...} wrappers).
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
        assert_eq!(req.commander_id, Patch::Set(None));
        assert_eq!(req.land_target, Patch::Set(Some(33)));
        assert!(req.format.is_unchanged());
        assert!(req.name.is_unchanged());

        // A null name decodes as Set(None), the handler's 422 signal.
        let req: HttpUpdateDeckProfile = serde_json::from_str(r#"{"name":null}"#).unwrap();
        assert_eq!(req.name, Patch::Set(None));
    }
}
