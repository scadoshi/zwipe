//! HTTP request builder using MTG-themed naming conventions.
//!
//! - **Planeswalker** = HTTP client wrapper with Scryfall API headers
//! - **untap** = create a new request builder
//! - **cast** = send the request
//! - **tutor** = search for a card

use crate::domain::card::models::scryfall_data::ScryfallData;
use anyhow::Context;
use reqwest::{
    header::{ACCEPT, USER_AGENT},
    Client, RequestBuilder, Response,
};
use serde::Deserialize;

// ==============================
//  equip onto scryfall requests
// ==============================
pub(super) const USER_AGENT_VALUE: &str = "zwipe/0.0";
pub(super) const ACCEPT_VALUE: &str = "*/*";
pub(super) const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
pub(super) const CARDS_SEARCH_ENDPOINT: &str = "/cards/search";

/// Scryfall search response wrapper.
#[derive(Deserialize, Debug)]
struct ScryfallDataSearchResponse {
    data: Vec<ScryfallData>,
}

// =========
//  helpers
// =========

/// Wraps a `RequestBuilder` with Scryfall API headers (User-Agent, Accept).
#[derive(Debug)]
pub(super) struct Planeswalker(RequestBuilder);

impl Planeswalker {
    /// Creates a new GET request builder with Scryfall headers.
    pub(super) fn untap(client: Client, full_url: &str) -> Self {
        Self(
            client
                .get(full_url)
                .header(USER_AGENT, USER_AGENT_VALUE)
                .header(ACCEPT, ACCEPT_VALUE),
        )
    }
    /// Sends the request.
    pub(super) async fn cast(self) -> Result<Response, reqwest::Error> {
        self.0.send().await
    }

    fn tutor_for(self, search_str: &str) -> Self {
        Planeswalker(self.0.query(&[("q", search_str)]))
    }

    /// Searches for a card by name via the Scryfall search endpoint.
    #[allow(dead_code)]
    pub(super) async fn tutor(
        client: Client,
        search_str: &str,
    ) -> anyhow::Result<Vec<ScryfallData>> {
        let url = SCRYFALL_API_BASE.to_string() + CARDS_SEARCH_ENDPOINT;
        let urza = Planeswalker::untap(client, &url);

        let get_result = urza
            .tutor_for(search_str)
            .cast()
            .await
            .context("failed to get on cards search endpoint")?;
        let get_json = get_result
            .json()
            .await
            .context("failed to parse json from get result")?;
        // tracing::debug!("card response was {:#?}", get_json);
        let card_search_response: ScryfallDataSearchResponse =
            serde_json::from_value(get_json).context("failed to parse CardSearchResponse")?;
        Ok(card_search_response.data)
    }
}

/// Extension trait for creating a `Planeswalker` from a reqwest `Client`.
#[allow(dead_code)]
pub(super) trait CreatePlaneswalker {
    /// Builds a Planeswalker targeting the given Scryfall endpoint.
    fn into_planeswalker(self, endpoint: &str) -> Planeswalker;
}

impl CreatePlaneswalker for Client {
    fn into_planeswalker(self, endpoint: &str) -> Planeswalker {
        Planeswalker::untap(self, endpoint)
    }
}
