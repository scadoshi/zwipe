//! Fetch the Universes Beyond franchise catalog.

use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::http::endpoints::card::GetUbFranchises;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the franchises the Universes Beyond exceptions picker offers.
    ///
    /// Served rather than compiled in, so a new crossover release becomes
    /// selectable on a deploy instead of a store train. There is no offline
    /// fallback: the picker writes a preference, which needs the server
    /// anyway.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn get_ub_franchises(&self) -> Result<Vec<UbFranchiseView>, ClientError> {
        self.call(GetUbFranchises, None).await
    }
}
