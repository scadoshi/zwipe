//! Scryfall HTTP requests: a `RequestBuilder` that always carries the
//! User-Agent and Accept headers Scryfall expects.

use anyhow::Context;
use reqwest::{
    Client, RequestBuilder, Response,
    header::{ACCEPT, USER_AGENT},
};
use serde::Deserialize;
use zwipe_core::domain::card::scryfall_data::ScryfallData;

// == request constants ==
pub(super) const USER_AGENT_VALUE: &str = "zwipe/0.0";
pub(super) const ACCEPT_VALUE: &str = "*/*";
pub(super) const SCRYFALL_API_BASE: &str = "https://api.scryfall.com";
pub(super) const CARDS_SEARCH_ENDPOINT: &str = "/cards/search";

/// Scryfall search response wrapper.
#[derive(Deserialize, Debug)]
struct ScryfallDataSearchResponse {
    data: Vec<ScryfallData>,
}

// == helpers ==

/// A `RequestBuilder` pre-set with the headers Scryfall expects.
#[derive(Debug)]
pub(super) struct ScryfallRequest(RequestBuilder);

impl ScryfallRequest {
    /// Builds a GET request carrying the Scryfall headers.
    pub(super) fn get(client: Client, full_url: &str) -> Self {
        Self(
            client
                .get(full_url)
                .header(USER_AGENT, USER_AGENT_VALUE)
                .header(ACCEPT, ACCEPT_VALUE),
        )
    }
    /// Sends the request.
    pub(super) async fn send(self) -> Result<Response, reqwest::Error> {
        self.0.send().await
    }

    /// Adds the `q=` search parameter.
    fn with_query(self, search_str: &str) -> Self {
        ScryfallRequest(self.0.query(&[("q", search_str)]))
    }

    /// Searches for a card by name via the Scryfall search endpoint.
    #[allow(dead_code)]
    pub(super) async fn search_cards(
        client: Client,
        search_str: &str,
    ) -> anyhow::Result<Vec<ScryfallData>> {
        let url = SCRYFALL_API_BASE.to_string() + CARDS_SEARCH_ENDPOINT;
        let request = ScryfallRequest::get(client, &url);

        let get_result = request
            .with_query(search_str)
            .send()
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

/// Extension trait for building a [`ScryfallRequest`] from a reqwest `Client`.
#[allow(dead_code)]
pub(super) trait IntoScryfallRequest {
    /// Builds a request targeting the given Scryfall endpoint.
    fn into_scryfall_request(self, endpoint: &str) -> ScryfallRequest;
}

impl IntoScryfallRequest for Client {
    fn into_scryfall_request(self, endpoint: &str) -> ScryfallRequest {
        ScryfallRequest::get(self, endpoint)
    }
}
