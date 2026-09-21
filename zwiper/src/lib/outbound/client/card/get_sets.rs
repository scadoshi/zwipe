//! Fetch all card sets.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetSets;

/// Trait for fetching the list of all card sets.
#[allow(missing_docs)]
pub trait ClientGetSets {
    fn get_sets(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetSets for ZwipeClient {
    async fn get_sets(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetSets, None).await
    }
}
