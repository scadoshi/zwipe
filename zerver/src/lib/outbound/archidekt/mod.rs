//! Archidekt outbound adapter.
//!
//! Fetches a public deck from Archidekt's open JSON API
//! (`GET https://archidekt.com/api/decks/{id}/`) and reduces it to the card
//! list the deck service can import, the deck's format, and the cards in
//! its commander category. Archidekt embeds the Scryfall printing id
//! on every card (`card.uid`), so resolution downstream is a direct id lookup:
//! no fuzzy name matching, and the exact printing is preserved.
//!
//! Archidekt's API is undocumented (open beta); this adapter deliberately
//! deserializes only the handful of fields we need and tolerates the rest.

use std::collections::HashSet;
use std::time::Duration;

use serde::{Deserialize, Deserializer};
use thiserror::Error;
use uuid::Uuid;
use zwipe_core::domain::deck::Format;

use crate::domain::deck::models::deck::import_archidekt::{ArchidektCard, ArchidektDeck};

/// Product token for the User-Agent; the contact URL is appended per-client
/// from config so it tracks the deployment's public domain.
const USER_AGENT_PRODUCT: &str = "ZwipeTCG/1.0";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);
/// Most a deck response may be before it is dropped unread. A 250-card deck
/// with Archidekt's full per-card payload is well under 2 MiB.
const MAX_RESPONSE_BYTES: usize = 5 * 1024 * 1024;

/// Errors fetching or parsing an Archidekt deck.
#[derive(Debug, Error)]
pub enum ArchidektError {
    /// The deck wasn't found (404): private, unlisted, or wrong id.
    #[error("deck not found on archidekt")]
    NotFound,
    /// Archidekt returned a non-success status other than 404.
    #[error("archidekt returned status {0}")]
    Upstream(u16),
    /// Network failure or response body parse failure.
    #[error("archidekt request failed: {0}")]
    Network(#[from] reqwest::Error),
    /// The response body exceeds `MAX_RESPONSE_BYTES`.
    #[error("archidekt response too large")]
    TooLarge,
}

/// Thin client over Archidekt's public deck API.
///
/// Holds a `reqwest::Client` (a connection pool); cheap to construct per call,
/// but can be shared if it ever lands in `AppState`.
#[derive(Debug, Clone)]
pub struct ArchidektClient {
    client: reqwest::Client,
    user_agent: String,
}

impl ArchidektClient {
    /// Creates a new Archidekt client whose User-Agent advertises `contact_url`
    /// (the deployment's public web base URL, from config).
    pub fn new(contact_url: &str) -> Self {
        Self {
            client: reqwest::Client::new(),
            user_agent: format!("{USER_AGENT_PRODUCT} (+{contact_url})"),
        }
    }

    /// Extracts a numeric Archidekt deck id from a deck URL or a bare id.
    ///
    /// Accepts `https://archidekt.com/decks/13769484/shorikai`,
    /// `archidekt.com/decks/13769484`, or just `13769484`.
    pub fn extract_deck_id(input: &str) -> Option<i64> {
        let input = input.trim();
        if let Ok(id) = input.parse::<i64>() {
            return Some(id);
        }
        let after = input.split("/decks/").nth(1)?;
        let digits: String = after.chars().take_while(char::is_ascii_digit).collect();
        digits.parse::<i64>().ok()
    }

    /// Fetches a public Archidekt deck and returns its cards, format and
    /// commanders.
    ///
    /// # Errors
    ///
    /// Returns [`ArchidektError::NotFound`] on a 404, `Upstream` on any other
    /// failing status or a body that does not parse, `TooLarge` past
    /// `MAX_RESPONSE_BYTES`, and `Network` when the request fails.
    pub async fn fetch_deck(&self, deck_id: i64) -> Result<ArchidektDeck, ArchidektError> {
        let url = format!("https://archidekt.com/api/decks/{deck_id}/");
        let response = self
            .client
            .get(&url)
            .header(reqwest::header::USER_AGENT, &self.user_agent)
            .timeout(REQUEST_TIMEOUT)
            .send()
            .await?;

        let status = response.status();
        if !status.is_success() {
            return Err(match status {
                reqwest::StatusCode::NOT_FOUND => ArchidektError::NotFound,
                other => ArchidektError::Upstream(other.as_u16()),
            });
        }

        if response
            .content_length()
            .is_some_and(|len| usize::try_from(len).map_or(true, |len| len > MAX_RESPONSE_BYTES))
        {
            return Err(ArchidektError::TooLarge);
        }
        let body = response.bytes().await?;
        if body.len() > MAX_RESPONSE_BYTES {
            return Err(ArchidektError::TooLarge);
        }
        let raw: RawDeck = serde_json::from_slice(&body).map_err(|e| {
            tracing::warn!(error = %e, "archidekt response did not parse");
            ArchidektError::Upstream(status.as_u16())
        })?;
        Ok(raw.into_deck())
    }
}

/// The format for Archidekt's `deckFormat` id, matched by the format's
/// display name. Archidekt's slugs predate Scryfall's renames, so its Brawl
/// (20) is Brawl here, not Historic Brawl. `None` for formats with no match
/// here (7 Custom, 8 Frontier, 11 1v1 Commander, 25 Canadian Highlander,
/// 26 Competitive Brawl, 27 Tiny Leaders Reborn) and unknown ids. Ids 11
/// through 27 come from Archidekt's frontend bundle.
fn format_from_archidekt(id: i64) -> Option<Format> {
    Some(match id {
        1 => Format::Standard,
        2 => Format::Modern,
        3 => Format::Commander,
        4 => Format::Legacy,
        5 => Format::Vintage,
        6 => Format::Pauper,
        9 => Format::Future,
        10 => Format::Penny,
        12 => Format::Duel,
        13 => Format::StandardBrawl,
        14 => Format::Oathbreaker,
        15 => Format::Pioneer,
        16 => Format::Historic,
        17 => Format::PauperCommander,
        18 => Format::Alchemy,
        19 => Format::Explorer,
        20 => Format::Brawl,
        21 => Format::Gladiator,
        22 => Format::Premodern,
        23 => Format::Predh,
        24 => Format::Timeless,
        _ => return None,
    })
}

/// The category name Archidekt marks as the deck's commanders.
const COMMANDER_CATEGORY: &str = "Commander";

// --- raw Archidekt response shapes (only what we read) ----------------------

#[derive(Debug, Deserialize)]
struct RawDeck {
    #[serde(rename = "deckFormat", default)]
    deck_format: Option<i64>,
    #[serde(default, deserialize_with = "null_as_default")]
    categories: Vec<RawCategory>,
    #[serde(default, deserialize_with = "null_as_default")]
    cards: Vec<RawCard>,
}

#[derive(Debug, Deserialize)]
struct RawCategory {
    name: String,
    #[serde(rename = "includedInDeck", default = "default_true")]
    included_in_deck: bool,
    #[serde(rename = "isPremier", default)]
    is_premier: bool,
}

#[derive(Debug, Deserialize)]
struct RawCard {
    #[serde(default = "default_quantity")]
    quantity: i32,
    // Archidekt sends `null` (not just an absent key) for a card stripped of all
    // its category tags. `#[serde(default)]` alone only covers a missing field,
    // so null must be mapped to an empty Vec explicitly or the whole deck fails.
    #[serde(default, deserialize_with = "null_as_default")]
    categories: Vec<String>,
    card: RawCardInner,
}

#[derive(Debug, Deserialize)]
struct RawCardInner {
    // A custom card or a stripped entry can carry a null/absent uid; treat it as
    // empty so it parses to a nil id and surfaces as unresolved (see into_cards),
    // rather than failing the whole-deck parse.
    #[serde(default, deserialize_with = "null_as_default")]
    uid: String,
    // Custom cards have `oracleCard: null` (their data lives in the deck's
    // top-level `customCards`); Option keeps the parse alive and the card falls
    // through to unresolved via its empty name.
    #[serde(rename = "oracleCard", default)]
    oracle_card: Option<RawOracleCard>,
}

#[derive(Debug, Deserialize)]
struct RawOracleCard {
    #[serde(default)]
    name: String,
}

fn default_true() -> bool {
    true
}

fn default_quantity() -> i32 {
    1
}

/// Deserializes a present-but-`null` field as `T::default()` instead of erroring.
///
/// `#[serde(default)]` only applies to absent keys; Archidekt sometimes sends an
/// explicit `null` for list fields (e.g. a card's `categories`), which would
/// otherwise fail the whole-deck parse.
fn null_as_default<'de, D, T>(deserializer: D) -> Result<T, D::Error>
where
    D: Deserializer<'de>,
    T: Default + Deserialize<'de>,
{
    Ok(Option::deserialize(deserializer)?.unwrap_or_default())
}

impl RawDeck {
    fn into_deck(self) -> ArchidektDeck {
        // The commander category is the one Archidekt marks premier, or failing
        // that the one named exactly "Commander", never a look-alike such as
        // "Backup Commander".
        let premier = self
            .categories
            .iter()
            .find(|c| c.is_premier)
            .or_else(|| {
                self.categories
                    .iter()
                    .find(|c| c.name.eq_ignore_ascii_case(COMMANDER_CATEGORY))
            })
            .map(|c| c.name.clone());
        let commanders = premier
            .map(|name| {
                self.cards
                    .iter()
                    .filter(|c| c.quantity > 0 && c.categories.contains(&name))
                    .map(RawCard::to_card)
                    .collect()
            })
            .unwrap_or_default();
        ArchidektDeck {
            format: self.deck_format.and_then(format_from_archidekt),
            commanders,
            cards: self.into_cards(),
        }
    }

    fn into_cards(self) -> Vec<ArchidektCard> {
        // Categories flagged includedInDeck=false are maybeboard/sideboard-like
        // (e.g. "Attraction") and are dropped from the import.
        let excluded: HashSet<&str> = self
            .categories
            .iter()
            .filter(|c| !c.included_in_deck)
            .map(|c| c.name.as_str())
            .collect();

        self.cards
            .iter()
            // A quantity of zero or less is noise, not a card: skipping it
            // keeps the deck_cards quantity check from failing the import.
            .filter(|c| c.quantity > 0)
            .filter(|c| {
                !c.categories
                    .iter()
                    .any(|cat| excluded.contains(cat.as_str()))
            })
            .map(RawCard::to_card)
            .collect()
    }
}

impl RawCard {
    fn to_card(&self) -> ArchidektCard {
        ArchidektCard {
            // Unparseable ids become nil so they surface as unresolved
            // rather than silently vanishing.
            scryfall_id: Uuid::parse_str(&self.card.uid).unwrap_or_else(|_| Uuid::nil()),
            name: self
                .card
                .oracle_card
                .as_ref()
                .map(|o| o.name.clone())
                .unwrap_or_default(),
            quantity: self.quantity,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_id_from_full_url() {
        assert_eq!(
            ArchidektClient::extract_deck_id("https://archidekt.com/decks/13769484/shorikai"),
            Some(13_769_484)
        );
    }

    #[test]
    fn extract_id_from_url_without_slug() {
        assert_eq!(
            ArchidektClient::extract_deck_id("archidekt.com/decks/11493358"),
            Some(11_493_358)
        );
    }

    #[test]
    fn extract_id_from_bare_id() {
        assert_eq!(
            ArchidektClient::extract_deck_id("11493358"),
            Some(11_493_358)
        );
    }

    #[test]
    fn extract_id_rejects_garbage() {
        assert_eq!(
            ArchidektClient::extract_deck_id("https://moxfield.com/decks/abc"),
            None
        );
    }

    /// A card with an explicit `"categories": null` (Archidekt emits this for a
    /// card stripped of all category tags) must not fail the whole-deck parse.
    /// Regression for deck 21966480 ("Everybody gets a lightning bolt").
    #[test]
    fn null_card_categories_parses_to_empty() {
        let json = r#"{
            "categories": [{"name": "Maybeboard", "includedInDeck": false}],
            "cards": [
                {
                    "quantity": 1,
                    "categories": null,
                    "card": {"uid": "b12e5430-0e80-47dd-80ac-85728b656a24",
                             "oracleCard": {"name": "Volcanic Island"}}
                },
                {
                    "quantity": 1,
                    "categories": ["Draw"],
                    "card": {"uid": "bf76c6a4-d6e8-4d50-b65f-020252f7b659",
                             "oracleCard": {"name": "Mystic Remora"}}
                }
            ]
        }"#;

        let raw: RawDeck =
            serde_json::from_str(json).expect("deck with null categories must parse");
        let names: Vec<String> = raw.into_cards().into_iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["Volcanic Island", "Mystic Remora"]);
    }

    /// A custom card has `oracleCard: null` (its data lives in the deck's
    /// top-level `customCards`). It must not fail the whole-deck parse; it parses
    /// with an empty name + nil id so it surfaces as unresolved downstream.
    #[test]
    fn null_oracle_card_and_uid_parse_to_unresolved() {
        let json = r#"{
            "cards": [
                {
                    "quantity": 1,
                    "categories": ["Commander"],
                    "card": {"uid": null, "oracleCard": null}
                },
                {
                    "quantity": 1,
                    "categories": ["Draw"],
                    "card": {"uid": "bf76c6a4-d6e8-4d50-b65f-020252f7b659",
                             "oracleCard": {"name": "Mystic Remora"}}
                }
            ]
        }"#;

        let raw: RawDeck = serde_json::from_str(json).expect("custom card must not fail the parse");
        let cards = raw.into_cards();
        let custom = cards.first().expect("first card present");
        assert_eq!(custom.scryfall_id, Uuid::nil());
        assert_eq!(custom.name, "");
        let names: Vec<&str> = cards.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["", "Mystic Remora"]);
    }

    /// Top-level `cards`/`categories` sent as `null` must also degrade to empty
    /// rather than erroring.
    #[test]
    fn null_top_level_lists_parse_to_empty() {
        let raw: RawDeck =
            serde_json::from_str(r#"{"categories": null, "cards": null}"#).expect("must parse");
        assert!(raw.into_cards().is_empty());
    }

    /// A card with a quantity of zero or less is skipped, not passed on to
    /// fail the whole import.
    #[test]
    fn non_positive_quantities_are_skipped() {
        let json = r#"{
            "cards": [
                {"quantity": 0, "card": {"uid": "b12e5430-0e80-47dd-80ac-85728b656a24",
                                         "oracleCard": {"name": "Volcanic Island"}}},
                {"quantity": -2, "card": {"uid": "0c7a4bd2-9b4e-4a33-8c3f-4a3b9a2d7f10",
                                          "oracleCard": {"name": "Sol Ring"}}},
                {"quantity": 1, "card": {"uid": "bf76c6a4-d6e8-4d50-b65f-020252f7b659",
                                         "oracleCard": {"name": "Mystic Remora"}}}
            ]
        }"#;

        let raw: RawDeck = serde_json::from_str(json).expect("must parse");
        let names: Vec<String> = raw.into_cards().into_iter().map(|c| c.name).collect();
        assert_eq!(names, vec!["Mystic Remora"]);
    }

    fn fixture(json: &str) -> ArchidektDeck {
        serde_json::from_str::<RawDeck>(json)
            .expect("the fixture parses")
            .into_deck()
    }

    fn names(cards: &[ArchidektCard]) -> Vec<&str> {
        cards.iter().map(|c| c.name.as_str()).collect()
    }

    /// Format 3 is Commander, and the premier category's card is the
    /// commander. It stays in the card list too; the import skips it there.
    #[test]
    fn format_three_with_a_commander_category() {
        let deck = fixture(include_str!("fixtures/commander_format3.json"));
        assert_eq!(deck.format, Some(Format::Commander));
        assert_eq!(names(&deck.commanders), ["Atraxa, Praetors' Voice"]);
        assert_eq!(names(&deck.cards), ["Atraxa, Praetors' Voice", "Sol Ring"]);
    }

    /// Two cards in the premier category are both commanders, in Archidekt's
    /// order.
    #[test]
    fn partners_are_both_commanders() {
        let deck = fixture(include_str!("fixtures/partners.json"));
        assert_eq!(
            names(&deck.commanders),
            ["Tymna the Weaver", "Thrasios, Triton Hero"]
        );
    }

    /// 1v1 Commander (11) has no format here, so the format stays unset while
    /// the commander still reads.
    #[test]
    fn an_unmapped_format_reads_as_none() {
        let deck = fixture(include_str!("fixtures/unmapped_format.json"));
        assert_eq!(deck.format, None);
        assert_eq!(names(&deck.commanders), ["Kenrith, the Returned King"]);
    }

    /// With no premier flag, the category named exactly "Commander" holds the
    /// commander, and "Backup Commander" is not it.
    #[test]
    fn a_backup_commander_is_not_the_commander() {
        let deck = fixture(include_str!("fixtures/backup_commander.json"));
        assert_eq!(names(&deck.commanders), ["Edgar Markov"]);
        assert!(!names(&deck.cards).contains(&"Sorin, Vengeful Bloodlord"));
    }

    /// The premier category wins over one merely named "Commander".
    #[test]
    fn the_premier_category_wins_over_the_name() {
        let deck = fixture(
            r#"{
                "deckFormat": 3,
                "categories": [
                    {"name": "Commander", "isPremier": false},
                    {"name": "Leaders", "isPremier": true}
                ],
                "cards": [
                    {"quantity": 1, "categories": ["Commander"],
                     "card": {"uid": "8d94b8ec-ecda-43c8-a60e-1ba33e6a54a4", "oracleCard": {"name": "Edgar Markov"}}},
                    {"quantity": 1, "categories": ["Leaders"],
                     "card": {"uid": "d0d33d52-3d28-4635-b985-51e126289259", "oracleCard": {"name": "Atraxa, Praetors' Voice"}}}
                ]
            }"#,
        );
        assert_eq!(names(&deck.commanders), ["Atraxa, Praetors' Voice"]);
    }

    /// A deck with no format, a null one, or no commander category reads as
    /// neither.
    #[test]
    fn no_format_and_no_commander_read_as_none() {
        for json in [
            r#"{"cards": []}"#,
            r#"{"deckFormat": null, "categories": [{"name": "Ramp"}]}"#,
        ] {
            let deck = fixture(json);
            assert_eq!(deck.format, None, "{json}");
            assert!(deck.commanders.is_empty(), "{json}");
        }
    }

    /// Every id the table maps reads as its format, Brawl's 20 as Brawl, and
    /// the ones with no match here as none.
    #[test]
    fn the_format_table_matches_the_spec() {
        let mapped = [
            (1, Format::Standard),
            (2, Format::Modern),
            (3, Format::Commander),
            (4, Format::Legacy),
            (5, Format::Vintage),
            (6, Format::Pauper),
            (9, Format::Future),
            (10, Format::Penny),
            (12, Format::Duel),
            (13, Format::StandardBrawl),
            (14, Format::Oathbreaker),
            (15, Format::Pioneer),
            (16, Format::Historic),
            (17, Format::PauperCommander),
            (18, Format::Alchemy),
            (19, Format::Explorer),
            (20, Format::Brawl),
            (21, Format::Gladiator),
            (22, Format::Premodern),
            (23, Format::Predh),
            (24, Format::Timeless),
        ];
        for (id, format) in mapped {
            assert_eq!(format_from_archidekt(id), Some(format), "id {id}");
        }
        for id in [0, 7, 8, 11, 25, 26, 27, 28, -1] {
            assert_eq!(format_from_archidekt(id), None, "id {id}");
        }
    }
}
