//! Fetch all card types.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::http::endpoints::card::GetCardTypes;

/// Trait for fetching the list of all card types (creature, instant, etc.).
#[allow(missing_docs)]
pub trait ClientGetCardTypes {
    fn get_card_types(&self) -> impl Future<Output = Result<Vec<String>, ClientError>> + Send;
}

impl ClientGetCardTypes for ZwipeClient {
    async fn get_card_types(&self) -> Result<Vec<String>, ClientError> {
        self.call(GetCardTypes, None).await
    }
}
