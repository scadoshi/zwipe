//! Fetch the oracle tag catalog.

use crate::{ClientError, ZwipeClient};
use zwipe_core::{domain::card::oracle_tag::OracleTag, http::endpoints::card::GetOracleTags};

impl ZwipeClient {
    /// Fetches the full oracle tag catalog (slug, label, description, parents).
    pub async fn get_oracle_tags(&self) -> Result<Vec<OracleTag>, ClientError> {
        self.call(GetOracleTags, None).await
    }
}
