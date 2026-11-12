//! Fetch one page of an oracle tag's example cards, with no account.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag_examples::OracleTagSlug;
use zwipe_core::http::endpoints::card::GetOracleTagExamples;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the page of `slug`'s example cards that starts at `offset`: the most played
    /// cards carrying the tag, no tokens, at most
    /// [`EXAMPLES_PAGE_SIZE`](zwipe_core::domain::card::oracle_tag_examples::EXAMPLES_PAGE_SIZE).
    /// The route is public, for a reader with no account.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status (an offset past the examples' end is one), or the reply doesn't decode.
    pub async fn fetch_oracle_tag_examples(
        &self,
        slug: &OracleTagSlug,
        offset: u32,
    ) -> Result<Vec<Card>, ClientError> {
        let endpoint = GetOracleTagExamples {
            slug: slug.clone(),
            offset,
        };
        self.call(endpoint, None).await
    }
}
