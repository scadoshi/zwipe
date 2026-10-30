//! Fetch the featured flavor card: one shared pick per UTC hour.
//!
//! Unauthed: the home screen shows it pre- and post-login, and the server
//! serves everyone the same card from its in-memory slot (see the featured
//! flavor plan).

use zwipe_core::domain::card::Card;
use zwipe_core::http::endpoints::card::FeaturedFlavor;

use crate::{ClientError, ZwipeClient};

impl ZwipeClient {
    /// Fetches the hour's featured flavor card.
    ///
    /// # Errors
    ///
    /// Returns a [`ClientError`] when the request fails to complete, the server answers with an
    /// error status, or the reply doesn't decode.
    pub async fn featured_flavor(&self) -> Result<Card, ClientError> {
        self.call(FeaturedFlavor, None).await
    }
}
