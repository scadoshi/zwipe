//! zwipe.net's card search: the nav's chip and box above the fold, the chip alone in the header
//! below it, where a tap opens the box across the header. A lookup goes out signed out, and a
//! pick opens the card, or the results, on app.zwipe.net.

use std::time::Duration;

use components::{CardSearch, CardSearchBar, SearchHost, use_card_search};
use dioxus::prelude::*;
use zwipe_client::card::lookup_cards::LookupError;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{Destination, Failure};

use crate::components::sleep_ms;
use crate::{APP_BASE, api};

/// The site's [`SearchHost`]: the browser's clock, the lookup signed out, and links to the web
/// app.
#[derive(Debug, Clone, Copy)]
struct SiteSearch;

impl SearchHost for SiteSearch {
    async fn sleep(self, wait: Duration) {
        sleep_ms(u32::try_from(wait.as_millis()).unwrap_or(u32::MAX)).await;
    }

    async fn lookup(self, lookup: HttpCardLookup) -> Result<Vec<Card>, Failure> {
        api::client()
            .lookup_cards(&lookup, None)
            .await
            .map_err(|error| match error {
                LookupError::RateLimited { retry_after } => Failure::RateLimited { retry_after },
                LookupError::Client(_) => Failure::Other,
            })
    }

    fn open(self, to: Destination) {
        document::eval(&open_js(&app_url(&to)));
    }
}

/// `to`'s address on the web app.
fn app_url(to: &Destination) -> String {
    format!("{APP_BASE}{}", to.path())
}

/// Sends the page to `url`, written as a JSON string so no character in it leaves the literal.
fn open_js(url: &str) -> String {
    let literal = serde_json::to_string(url).unwrap_or_else(|_| "\"/\"".to_owned());
    format!("window.location.assign({literal});")
}

fn use_site_search() -> CardSearch<SiteSearch> {
    use_card_search(SiteSearch)
}

/// Where the nav folds, the card search is the header's.
const HEADER_MEDIA: &str = "(max-width: 60rem)";

/// The class of the site nav's icon buttons, the store icons, which the glass wears too.
pub(crate) const GLASS_CLASS: &str = "nav-cta nav-cta-icon";

/// The nav's card search, its glass one of the nav's icon buttons.
#[component]
pub(crate) fn NavCardSearch() -> Element {
    let search = use_site_search();
    rsx! {
        CardSearchBar {
            model: search.model(),
            on_action: Callback::new(move |action| search.dispatch(action)),
            input_id: "site-nav-card-search",
            header_media: HEADER_MEDIA,
            glass_class: GLASS_CLASS,
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::slug;

    use super::*;

    #[test]
    fn a_pick_opens_the_card_on_the_web_app() {
        let to = Destination::Card(slug::card_slug("Sol Ring").unwrap());
        assert_eq!(app_url(&to), format!("{APP_BASE}/card/sol-ring"));
    }

    #[test]
    fn several_matches_open_the_web_app_s_results() {
        let to = Destination::Results {
            query: "solemn".to_owned(),
        };
        assert_eq!(app_url(&to), format!("{APP_BASE}/search?q=solemn"));
    }

    #[test]
    fn the_address_stays_inside_its_string() {
        assert_eq!(
            open_js("https://app.zwipe.net/search?q=a\"b"),
            r#"window.location.assign("https://app.zwipe.net/search?q=a\"b");"#
        );
    }
}
