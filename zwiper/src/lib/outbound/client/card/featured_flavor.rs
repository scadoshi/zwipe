//! Fetch the featured flavor card: one shared pick per UTC hour.
//!
//! Unauthed: the home screen shows it pre- and post-login, and the server
//! serves everyone the same card from its in-memory slot (see the featured
//! flavor plan).

use crate::outbound::client::{ClientError, ZwipeClient};
use zwipe_core::{domain::card::Card, http::endpoints::card::FeaturedFlavor};

impl ZwipeClient {
    /// Fetches the hour's featured flavor card.
    pub async fn featured_flavor(&self) -> Result<Card, ClientError> {
        self.call(FeaturedFlavor, None).await
    }
}
