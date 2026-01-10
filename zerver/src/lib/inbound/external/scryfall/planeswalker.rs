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

/// scryfall returns this
/// when you search for a card
#[derive(Deserialize, Debug)]
struct ScryfallDataSearchResponse {
    data: Vec<ScryfallData>,
    // has_more: bool,
    // object: String,
    // total_cards: i32,
}

// =========
//  helpers
// =========

// for building scryfall based requests
// yes these use magic the gathering terms

#[derive(Debug)]
/// ensures we have a `RequestBuilder` appropriate
/// for making calls on the Scryfall API
pub(super) struct Planeswalker(RequestBuilder);

impl Planeswalker {
    /// main constructor
    pub(super) fn untap(client: Client, full_url: &str) -> Self {
        Self(
            client
                .get(full_url)
                .header(USER_AGENT, USER_AGENT_VALUE)
                .header(ACCEPT, ACCEPT_VALUE),
        )
    }
    /// for sending requests
    pub(super) async fn cast(self) -> Result<Response, reqwest::Error> {
        self.0.send().await
    }

    /// adding the "q" parameter with value input
    /// (meant for the tutor function)
    fn tutor_for(self, search_str: &str) -> Self {
        Planeswalker(self.0.query(&[("q", search_str)]))
    }

    #[allow(dead_code)]
    /// for searching for a single card
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

/// for getting a `RequestBuilder` ready with scryfall api url + endpoint
#[allow(dead_code)]
pub(super) trait CreatePlaneswalker {
    fn into_planeswalker(self, endpoint: &str) -> Planeswalker;
}

impl CreatePlaneswalker for Client {
    fn into_planeswalker(self, endpoint: &str) -> Planeswalker {
        Planeswalker::untap(self, endpoint)
    }
}
