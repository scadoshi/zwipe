//! Fetch the oracle tag catalog.

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{domain::card::oracle_tag::OracleTag, http::endpoints::card::GetOracleTags};

/// Trait for fetching the full oracle tag catalog (slug, label, description, parents).
#[allow(missing_docs)]
pub trait ClientGetOracleTags {
    fn get_oracle_tags(&self) -> impl Future<Output = Result<Vec<OracleTag>, ClientError>> + Send;
}

impl ClientGetOracleTags for ZwipeClient {
    async fn get_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError> {
        self.call(GetOracleTags, None).await
    }
}
