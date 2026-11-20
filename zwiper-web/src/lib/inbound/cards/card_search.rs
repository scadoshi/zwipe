//! The card search on the web: its lookup, signed in with a fresh session when one is held, so
//! the user's Universes Beyond setting applies, else signed out; its routes; each submitted
//! search counted in the usage buffer while signed in; and the wide nav's box, which
//! [`NavSearch`] draws.

use std::time::Duration;

use components::{CardSearch, CardSearchBar, SearchHost, use_card_search};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_client::card::lookup_cards::LookupError;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{Destination, Failure};
use zwipe_ui::telemetry::SharedUsageBuffer;

use crate::inbound::chrome::media::NAV_FOLD_MAX_PX;
use crate::inbound::router::Route;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::inbound::telemetry;
use crate::outbound::timer;

/// The nav box's id.
const NAV_SEARCH_ID: &str = "nav-card-search";

/// The web app's [`SearchHost`]: the browser's clock, the lookup with the session when one is
/// held, the app's routes, and the usage buffer its searches count in.
#[derive(Clone, Copy)]
pub struct WebSearch {
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    navigator: Navigator,
    usage: Signal<SharedUsageBuffer>,
}

impl std::fmt::Debug for WebSearch {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("WebSearch").finish_non_exhaustive()
    }
}

impl SearchHost for WebSearch {
    async fn sleep(self, wait: Duration) {
        timer::sleep(wait).await;
    }

    async fn lookup(self, lookup: HttpCardLookup) -> Result<Vec<Card>, Failure> {
        lookup_cards(self.session, self.client, &lookup).await
    }

    fn open(self, to: Destination) {
        // A submitted search starts at the top on a fresh lookup; only Back finds its old place.
        if let Destination::Results { query } = &to {
            components::results_place::submitted(query);
        }
        if let Some(route) = route_of(to) {
            self.navigator.push(route);
        }
    }

    fn record_search(self) {
        if self.session.peek().is_some() {
            self.usage.peek().record_search();
        }
    }
}

/// A card search bar driven by the web app, for one mount.
fn use_web_search() -> CardSearch<WebSearch> {
    use_card_search(WebSearch {
        session: use_context(),
        client: use_context(),
        navigator: use_navigator(),
        usage: telemetry::use_usage(),
    })
}

/// The route a search opens: a card page, a printing's page, or the results page.
pub fn route_of(to: Destination) -> Option<Route> {
    match to {
        Destination::Card(slug) => Some(Route::CardNamed {
            slug: slug.to_string(),
            open: None,
        }),
        Destination::Printing { path } => path.parse().ok(),
        Destination::Results { query } => Some(Route::Search { q: query }),
    }
}

/// Where the nav folds, the card search is the header's.
fn header_media() -> String {
    format!("(max-width: {NAV_FOLD_MAX_PX}px)")
}

/// The nav's card search: the chip and the box on a wide window, the chip alone in the header
/// below the nav's fold, where a tap opens the box across the header.
#[component]
pub fn NavSearch() -> Element {
    let search = use_web_search();
    rsx! {
        CardSearchBar {
            model: search.model(),
            on_action: Callback::new(move |action| search.dispatch(action)),
            input_id: NAV_SEARCH_ID,
            header_media: header_media(),
        }
    }
}

/// Looks up `lookup` for the card search's [`Effect::Lookup`](zwipe_ui::screens::card_search::Effect::Lookup).
/// A held session is freshened first, refreshing an access token near its end; one that can't
/// be (its refresh refused, or no refresh token left) looks up signed out. A token the server
/// still refuses asks once more signed out, in the client.
///
/// # Errors
///
/// The model's [`Failure`]: rate limited with the server's `Retry-After` when readable, or any
/// other failure.
pub async fn lookup_cards(
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    lookup: &HttpCardLookup,
) -> Result<Vec<Card>, Failure> {
    let fresh = if session.peek().is_some() {
        ensure_fresh(session, client).await.ok()
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
    use uuid::Uuid;
    use zwipe_core::domain::card::slug;

    use super::*;

    #[test]
    fn a_name_opens_its_card_page_and_a_name_with_no_slug_its_printing() {
        assert_eq!(
            route_of(Destination::Card(slug::card_slug("Sol Ring").unwrap())),
            Some(Route::CardNamed {
                slug: "sol-ring".to_owned(),
                open: None
            })
        );
        let id = Uuid::new_v4();
        let path = slug::printing_path("_____", id);
        assert!(
            matches!(route_of(Destination::Printing { path: path.clone() }), Some(Route::CardPrinting { id: got, open: None, .. }) if got == id),
            "{path}"
        );
    }

    #[test]
    fn several_matches_open_the_results_page_for_the_text() {
        let route = route_of(Destination::Results {
            query: "solemn sim".to_owned(),
        })
        .unwrap();
        assert_eq!(
            route,
            Route::Search {
                q: "solemn sim".to_owned()
            }
        );
        assert_eq!(route.to_string(), "/search?q=solemn%20sim");
    }
}
