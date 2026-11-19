//! Deal a page of the merged commander ranking.

use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::suggestions::{HttpCommanderDeal, HttpCommanderDealPage};
use zwipe_core::http::endpoints::suggestions::DealCommanders;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Deals one page of the merged commander ranking: every commander `deal`'s filter matches
    /// that can lead a deck in its format, best fit for the user first, from its offset.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn deal_commanders(
        &self,
        deal: HttpCommanderDeal,
        session: &Session,
    ) -> Result<HttpCommanderDealPage, ClientError> {
        self.call(DealCommanders(deal), Some(session)).await
    }
}
