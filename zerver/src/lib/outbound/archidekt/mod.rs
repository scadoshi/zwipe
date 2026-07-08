//! Archidekt outbound adapter.
//!
//! Fetches a public deck from Archidekt's open JSON API
//! (`GET https://archidekt.com/api/decks/{id}/`) and reduces it to the card
//! list the deck service can import. Archidekt embeds the Scryfall printing id
//! on every card (`card.uid`), so resolution downstream is a direct id lookup —
//! no fuzzy name matching, and the exact printing is preserved.
//!
//! Archidekt's API is undocumented (open beta); this adapter deliberately
//! deserializes only the handful of fields we need and tolerates the rest.

use crate::domain::deck::models::deck::import_archidekt::ArchidektCard;
use serde::{Deserialize, Deserializer};
use std::{collections::HashSet, time::Duration};
use thiserror::Error;
use uuid::Uuid;

/// Product token for the User-Agent; the contact URL is appended per-client
/// from config so it tracks the deployment's public domain.
const USER_AGENT_PRODUCT: &str = "ZwipeTCG/1.0";
const REQUEST_TIMEOUT: Duration = Duration::from_secs(10);

/// Errors fetching or parsing an Archidekt deck.
#[derive(Debug, Error)]
pub enum ArchidektError {
    /// The deck wasn't found (404) — private, unlisted, or wrong id.
    #[error("deck not found on archidekt")]
    NotFound,
    /// Archidekt returned a non-success status other than 404.
    #[error("archidekt returned status {0}")]
    Upstream(u16),
    /// Network failure or response body parse failure.
    #[error("archidekt request failed: {0}")]
    Network(#[from] reqwest::Error),
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

    /// Fetches a public Archidekt deck and returns its card list.
    pub async fn fetch_deck(&self, deck_id: i64) -> Result<Vec<ArchidektCard>, ArchidektError> {
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

        let raw: RawDeck = response.json().await?;
        Ok(raw.into_cards())
    }
}

// --- raw Archidekt response shapes (only what we read) ----------------------

#[derive(Debug, Deserialize)]
struct RawDeck {
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
            .filter(|c| {
                !c.categories
                    .iter()
                    .any(|cat| excluded.contains(cat.as_str()))
            })
            .map(|c| ArchidektCard {
                // Unparseable ids become nil so they surface as unresolved
                // rather than silently vanishing.
                scryfall_id: Uuid::parse_str(&c.card.uid).unwrap_or_else(|_| Uuid::nil()),
                name: c
                    .card
                    .oracle_card
                    .as_ref()
                    .map(|o| o.name.clone())
                    .unwrap_or_default(),
                quantity: c.quantity,
            })
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn extract_id_from_full_url() {
        assert_eq!(
            ArchidektClient::extract_deck_id("https://archidekt.com/decks/13769484/shorikai"),
            Some(13769484)
        );
    }

    #[test]
    fn extract_id_from_url_without_slug() {
        assert_eq!(
            ArchidektClient::extract_deck_id("archidekt.com/decks/11493358"),
            Some(11493358)
        );
    }

    #[test]
    fn extract_id_from_bare_id() {
        assert_eq!(ArchidektClient::extract_deck_id("11493358"), Some(11493358));
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
    #[allow(clippy::expect_used)]
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
    #[allow(clippy::expect_used)]
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
        assert!(custom.name.is_empty());
        let names: Vec<&str> = cards.iter().map(|c| c.name.as_str()).collect();
        assert_eq!(names, vec!["", "Mystic Remora"]);
    }

    /// Top-level `cards`/`categories` sent as `null` must also degrade to empty
    /// rather than erroring.
    #[test]
    #[allow(clippy::expect_used)]
    fn null_top_level_lists_parse_to_empty() {
        let raw: RawDeck =
            serde_json::from_str(r#"{"categories": null, "cards": null}"#).expect("must parse");
        assert!(raw.into_cards().is_empty());
    }
}
