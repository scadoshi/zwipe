//! Leaving a deck screen whose deck the server no longer has.

use dioxus::router::Navigator;
use zwipe_client::ClientError;

use crate::inbound::router::Router;

/// Hands back a deck request's value, or `None` when it failed.
///
/// A not-found reply means the deck is gone, so the deck list replaces the
/// screen and Back lands wherever the user was before the deck. The request
/// runner has already reported the failure and raised its toast.
pub(crate) fn settle<T>(navigator: Navigator, reply: Result<T, ClientError>) -> Option<T> {
    match reply {
        Ok(value) => Some(value),
        Err(ClientError::NotFound(_)) => {
            navigator.replace(Router::DeckList {});
            None
        }
        Err(_) => None,
    }
}
