//! The card search's lookup: signed in with a fresh session when one is held, so the user's
//! Universes Beyond setting applies, else signed out.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_client::card::lookup_cards::LookupError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::Failure;

use crate::inbound::components::auth::ensure_session::EnsureFresh;

/// Looks up `lookup` for the card search's [`Effect::Lookup`](zwipe_ui::screens::card_search::Effect::Lookup).
/// A held session is freshened first, refreshing an access token near its end; one that can't
/// be (its refresh refused, or the refresh token expired) looks up signed out. A token the
/// server still refuses asks once more signed out, in the client.
///
/// # Errors
///
/// The model's [`Failure`]: rate limited with the server's `Retry-After`, or any other failure.
pub async fn lookup_cards(
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    lookup: &HttpCardLookup,
) -> Result<Vec<Card>, Failure> {
    let fresh = if session.peek().is_some() {
        session.ensure_fresh(client).await.ok()
    } else {
        None
    };
    let http = client.peek().clone();
    http.lookup_cards(lookup, fresh.as_ref())
        .await
        .map_err(|error| failure(&error))
}

/// The model's reading of a lookup error.
fn failure(error: &LookupError) -> Failure {
    match error {
        LookupError::RateLimited { retry_after } => Failure::RateLimited {
            retry_after: *retry_after,
        },
        LookupError::Client(_) => Failure::Other,
    }
}
