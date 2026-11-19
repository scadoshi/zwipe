//! The card search in the app: its lookup, signed in with a fresh session when one is held, so
//! the user's Universes Beyond setting applies, else signed out; and its routes, the card page
//! and the results page, which `use_app_search` drives Home's Card search panel with.

use std::time::Duration;

use components::{CardSearch, SearchHost, use_card_search};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_client::card::lookup_cards::LookupError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{Destination, Failure};

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::router::Router;

/// The app's [`SearchHost`]: the runtime's clock, the lookup with the session when one is held,
/// and the app's routes.
#[derive(Clone, Copy)]
pub(crate) struct AppSearch {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    navigator: Navigator,
}

impl std::fmt::Debug for AppSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppSearch").finish_non_exhaustive()
    }
}

impl SearchHost for AppSearch {
    async fn sleep(self, wait: Duration) {
        tokio::time::sleep(wait).await;
    }

    async fn lookup(self, lookup: HttpCardLookup) -> Result<Vec<Card>, Failure> {
        lookup_cards(self.session, self.client, &lookup).await
    }

    fn open(self, to: Destination) {
        if let Some(route) = route_of(to) {
            self.navigator.push(route);
        }
    }
}

/// A card search bar driven by the app, for one mount.
pub(crate) fn use_app_search() -> CardSearch<AppSearch> {
    use_card_search(AppSearch {
        session: use_context(),
        client: use_context(),
        navigator: use_navigator(),
    })
}

/// The route a search opens: a card page, a printing's page, or the results page.
pub(crate) fn route_of(to: Destination) -> Option<Router> {
    match to {
        Destination::Card(slug) => Some(Router::CardNamed {
            slug: slug.to_string(),
        }),
        Destination::Printing { path } => path.parse().ok(),
        Destination::Results { query } => Some(Router::Search { q: query }),
    }
}

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
