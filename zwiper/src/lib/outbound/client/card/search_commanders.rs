//! Commander search: the dedicated select-serving path.
//!
//! Same `CardQuery` body as the plain search, but the server orders by
//! decks-helmed popularity, banded + wildcarded per user per day, with
//! token/emblem printings excluded. An explicit sort in the filter still
//! wins. (context/archive/commander_select_ordering.md)

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{
    domain::{
        auth::models::session::Session,
        card::{Card, search_card::card_filter::CardQuery},
    },
    http::endpoints::card::SearchCommanders,
};

/// Trait for searching commander candidates.
#[allow(missing_docs)]
pub trait ClientSearchCommanders {
    fn search_commanders(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> impl Future<Output = Result<Vec<Card>, ClientError>> + Send;
}

impl ClientSearchCommanders for ZwipeClient {
    async fn search_commanders(
        &self,
        card_filter: &CardQuery,
        session: &Session,
    ) -> Result<Vec<Card>, ClientError> {
        let body = serde_json::to_value(card_filter)?;
        self.call(SearchCommanders(body), Some(session)).await
    }
}
