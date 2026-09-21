//! Fetch the featured flavor card: one shared pick per UTC hour.
//!
//! Unauthed: the home screen shows it pre- and post-login, and the server
//! serves everyone the same card from its in-memory slot (see the featured
//! flavor plan).

use crate::outbound::client::{ClientError, ZwipeClient};
use std::future::Future;
use zwipe_core::{domain::card::Card, http::endpoints::card::FeaturedFlavor};

/// Trait for fetching the hour's featured flavor card.
#[allow(missing_docs)]
pub trait ClientFeaturedFlavor {
    fn featured_flavor(&self) -> impl Future<Output = Result<Card, ClientError>> + Send;
}

impl ClientFeaturedFlavor for ZwipeClient {
    async fn featured_flavor(&self) -> Result<Card, ClientError> {
        self.call(FeaturedFlavor, None).await
    }
}
