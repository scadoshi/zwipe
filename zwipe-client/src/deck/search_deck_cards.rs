//! Deck-aware card search.
//!
//! Same `CardQuery` body as the plain search, but scoped to a deck: the
//! server excludes cards already in the deck (any board, plus profile slots)
//! and defaults to synergy ordering when no explicit sort is set.
//!
//! The response is read in either shape. Today the server answers a bare
//! `Vec<Card>` and reports synergy through the `x-synergy-applied` header;
//! the envelope carries the same fact in the body. Reading both is what lets
//! the server switch once the version floor rules out older clients. See
//! `context/plans/synergy_flag_into_body.md`.

use crate::{ClientError, ZwipeClient};
use reqwest::StatusCode;
use serde::Deserialize;
use tracing::info;
use uuid::Uuid;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::{contracts::deck::HttpDeckCardSearch, paths::search_deck_cards_route},
};

/// Either response shape. An array cannot match the struct and an object
/// cannot match the `Vec`, so the untagged match is unambiguous.
#[derive(Deserialize)]
#[serde(untagged)]
enum SearchPayload {
    Envelope(HttpDeckCardSearch),
    Bare(Vec<Card>),
}

impl ZwipeClient {
    /// Deck-aware card search.
    ///
    /// Returns `(cards, synergy_warming)`: `synergy_warming` is true when
    /// synergy was requested but the commander's cache was still warming, so
    /// the server served the full pool.
    pub async fn search_deck_cards(
        &self,
        deck_id: Uuid,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<(Vec<Card>, bool), ClientError> {
        let mut url = self.base_url.clone();
        url.set_path(&search_deck_cards_route(deck_id));

        info!("POST {} filter: {:?}", url, card_filter);

        let response = self
            .client
            .post(url)
            .json(card_filter)
            .bearer_auth(&*session.access_token.value)
            .send()
            .await?;

        match response.status() {
            StatusCode::OK => {
                // Read the header before `json()` consumes the response. It is
                // the fallback for a bare array; the envelope carries its own.
                let header_warming = header_says_warming(response.headers());
                let bytes = response.bytes().await?;
                Ok(match serde_json::from_slice::<SearchPayload>(&bytes)? {
                    // The body wins: it describes the cards it arrived with,
                    // and a browser cannot read the header at all.
                    SearchPayload::Envelope(body) => (body.cards, !body.synergy_applied),
                    SearchPayload::Bare(cards) => (cards, header_warming),
                })
            }
            status => {
                let message = response.text().await?;
                Err((status, message).into())
            }
        }
    }
}

/// `x-synergy-applied: false` means synergy was requested but the commander's
/// cache is still warming. An absent header means the server applied synergy
/// or was never asked for it, which is not a warming state either way.
fn header_says_warming(headers: &reqwest::header::HeaderMap) -> bool {
    headers
        .get("x-synergy-applied")
        .and_then(|v| v.to_str().ok())
        .is_some_and(|v| v == "false")
}

#[cfg(test)]
mod tests {
    #![allow(clippy::unwrap_used)]
    use super::*;

    /// The shape every shipped client sends today, and the one the server
    /// still answers with. It must keep decoding.
    #[test]
    fn a_bare_array_still_decodes() {
        let payload: SearchPayload = serde_json::from_str("[]").unwrap();
        assert!(matches!(payload, SearchPayload::Bare(cards) if cards.is_empty()));
    }

    /// The shape step 3 of the plan switches the server to.
    #[test]
    fn an_envelope_decodes_with_its_flag() {
        let json = r#"{"cards":[],"synergy_applied":false}"#;
        let payload: SearchPayload = serde_json::from_str(json).unwrap();
        let SearchPayload::Envelope(body) = payload else {
            unreachable!("an object must not decode as a bare array")
        };
        assert!(body.cards.is_empty());
        assert!(!body.synergy_applied, "the warming case must survive");
    }

    /// Neither variant may swallow the other's shape.
    #[test]
    fn the_two_shapes_do_not_collide() {
        assert!(matches!(
            serde_json::from_str::<SearchPayload>(r#"{"cards":[],"synergy_applied":true}"#),
            Ok(SearchPayload::Envelope(_))
        ));
        assert!(matches!(
            serde_json::from_str::<SearchPayload>("[]"),
            Ok(SearchPayload::Bare(_))
        ));
        // Anything else is still an error rather than a silent empty result.
        assert!(serde_json::from_str::<SearchPayload>(r#"{"cards":[]}"#).is_err());
    }
}
