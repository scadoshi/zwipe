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

use std::collections::HashSet;
use std::time::Duration;

use serde::Deserialize;
use thiserror::Error;
use uuid::Uuid;

use crate::domain::deck::models::deck::import_archidekt::ArchidektCard;

const USER_AGENT: &str = "ZwipeTCG/1.0 (+https://zwipe.net)";
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
}

impl Default for ArchidektClient {
    fn default() -> Self {
        Self::new()
    }
}

impl ArchidektClient {
    /// Creates a new Archidekt client.
    pub fn new() -> Self {
        Self {
            client: reqwest::Client::new(),
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
            .header(reqwest::header::USER_AGENT, USER_AGENT)
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
    #[serde(default)]
    categories: Vec<RawCategory>,
    #[serde(default)]
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
    #[serde(default)]
    categories: Vec<String>,
    card: RawCardInner,
}

#[derive(Debug, Deserialize)]
struct RawCardInner {
    uid: String,
    #[serde(rename = "oracleCard")]
    oracle_card: RawOracleCard,
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
            .filter(|c| !c.categories.iter().any(|cat| excluded.contains(cat.as_str())))
            .map(|c| ArchidektCard {
                // Unparseable ids become nil so they surface as unresolved
                // rather than silently vanishing.
                scryfall_id: Uuid::parse_str(&c.card.uid).unwrap_or_else(|_| Uuid::nil()),
                name: c.card.oracle_card.name.clone(),
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
        assert_eq!(ArchidektClient::extract_deck_id("https://moxfield.com/decks/abc"), None);
    }
}
