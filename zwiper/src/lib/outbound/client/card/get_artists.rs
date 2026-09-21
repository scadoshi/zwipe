//! Fetch all unique artist names.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetArtists;

/// Trait for fetching the list of all unique card artists.
#[allow(missing_docs)]
pub trait ClientGetArtists {
    fn get_artists(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetArtists for ZwipeClient {
    async fn get_artists(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetArtists, None).await
    }
}
