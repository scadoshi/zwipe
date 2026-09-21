//! Fetch all available languages.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetLanguages;

/// Trait for fetching the list of all available card languages.
#[allow(missing_docs)]
pub trait ClientGetLanguages {
    fn get_languages(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetLanguages for ZwipeClient {
    async fn get_languages(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetLanguages, None).await
    }
}
