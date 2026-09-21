//! Fetch all oracle text words.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetOracleWords;

/// Trait for fetching the list of all normalized oracle text words.
#[allow(missing_docs)]
pub trait ClientGetOracleWords {
    fn get_oracle_words(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetOracleWords for ZwipeClient {
    async fn get_oracle_words(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetOracleWords, None).await
    }
}
