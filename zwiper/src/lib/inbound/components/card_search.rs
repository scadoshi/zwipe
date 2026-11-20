//! The card search in the app: its lookup, signed in with a fresh session when one is held, so
//! the user's Universes Beyond setting applies, else signed out; its routes, the card page and
//! the results page; each submitted search counted in the usage buffer while signed in; and
//! `HeaderSearch`, the chip in every signed-in screen's header that
//! opens the box across it.

use std::time::Duration;

use components::{CardSearch, CardSearchBar, HEADER_ALWAYS, SearchHost, use_card_search};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_client::card::lookup_cards::LookupError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{Destination, Failure};

use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::router::Router;

/// The app's [`SearchHost`]: the runtime's clock, the lookup with the session when one is held,
/// the app's routes, and the usage buffer its searches count in.
#[derive(Clone, Copy)]
pub(crate) struct AppSearch {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    navigator: Navigator,
    usage_buffer: Signal<UsageBuffer>,
}

impl std::fmt::Debug for AppSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("AppSearch").finish_non_exhaustive()
    }
}

impl SearchHost for AppSearch {
    async fn sleep(self, wait: Duration) {
        crate::outbound::timer::sleep(wait).await;
    }

    async fn lookup(self, lookup: HttpCardLookup) -> Result<Vec<Card>, Failure> {
        lookup_cards(self.session, self.client, &lookup).await
    }

    fn open(self, to: Destination) {
        // A submitted search starts at the top on a fresh lookup; only Back finds its old place.
        if let Destination::Results { query } = &to {
            components::results_place::forget(query);
        }
        if let Some(route) = route_of(to) {
            self.navigator.push(route);
        }
    }

    fn record_search(self) {
        if self.session.peek().is_some() {
            self.usage_buffer.peek().record_search();
        }
    }
}

/// The header's card search: the chip in the header, a tap on which opens the box across it.
#[component]
pub(crate) fn HeaderSearch() -> Element {
    let search = use_app_search();
    let input_id = use_hook(|| header_search_id(Uuid::new_v4()));
    rsx! {
        CardSearchBar {
            model: search.model(),
            on_action: Callback::new(move |action| search.dispatch(action)),
            input_id,
            header_media: HEADER_ALWAYS,
        }
    }
}

/// The header search's box id for one mount, so a header drawn while another's is still
/// mounted never shares its box, or its chip's label, with it.
fn header_search_id(mount: Uuid) -> String {
    format!("header-card-search-{}", mount.simple())
}

/// A card search bar driven by the app, for one mount.
fn use_app_search() -> CardSearch<AppSearch> {
    use_card_search(AppSearch {
        session: use_context(),
        client: use_context(),
        navigator: use_navigator(),
        usage_buffer: use_context(),
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn each_header_search_names_its_box_for_its_own_mount() {
        let first = header_search_id(Uuid::from_u128(1));
        let second = header_search_id(Uuid::from_u128(2));
        assert_ne!(first, second, "two mounts never share a box id");
        assert!(first.starts_with("header-card-search-"), "{first}");
    }
}
