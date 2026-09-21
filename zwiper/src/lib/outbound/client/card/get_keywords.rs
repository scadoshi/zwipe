//! Fetch all keyword abilities.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetKeywords;

/// Trait for fetching the list of all keyword abilities (flying, trample, etc.).
#[allow(missing_docs)]
pub trait ClientGetKeywords {
    fn get_keywords(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetKeywords for ZwipeClient {
    async fn get_keywords(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetKeywords, None).await
    }
}
