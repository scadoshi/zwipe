//! Home's card search bar over the [`card_search`] model: typing looks cards up by name, the
//! results show as chips under the box in the page's flow, and a chip opens that card's page.

use components::ClearWrap;
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::card_search::{self, Action, Destination, Effect, State, Status};

use crate::inbound::cards::card_search::lookup_cards;
use crate::inbound::router::Route;
use crate::outbound::timer;

/// The box's id.
const SEARCH_ID: &str = "home-card-search";

/// The route a picked card opens.
fn route_of(to: Destination) -> Option<Route> {
    match to {
        Destination::Card(slug) => Some(Route::CardNamed {
            slug: slug.to_string(),
            open: None,
        }),
        Destination::Printing { path } => path.parse().ok(),
    }
}

/// The bar's model with the handles that perform its effects. Copy, so every handler and task
/// holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    navigator: Navigator,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one of the model's effects.
    fn perform(self, effect: Effect) {
        match effect {
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    timer::sleep(wait).await;
                    self.dispatch(Action::Debounced(ticket));
                });
            }
            Effect::Lookup { ticket, lookup } => {
                spawn(async move {
                    let reply = match lookup_cards(self.session, self.client, &lookup).await {
                        Ok(cards) => Action::Loaded { ticket, cards },
                        Err(failure) => Action::Failed { ticket, failure },
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Cooldown { ticket, wait } => {
                spawn(async move {
                    timer::sleep(wait).await;
                    self.dispatch(Action::CooledDown(ticket));
                });
            }
            Effect::Navigate(to) => {
                if let Some(route) = route_of(to) {
                    self.navigator.push(route);
                }
            }
        }
    }
}

/// The search bar: the labeled box, then a line or the result chips under it.
#[component]
pub(super) fn CardSearchBar() -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        session: use_context(),
        client: use_context(),
        navigator: use_navigator(),
    };
    let state = driver.model.read();
    let query = state.query().to_owned();
    let status = state.status();
    let results: Vec<(Uuid, String)> = state
        .results()
        .iter()
        .map(|c| (c.scryfall_data.id, c.scryfall_data.name.clone()))
        .collect();
    drop(state);
    let line = match status {
        Status::Idle | Status::Results => None,
        Status::Loading => Some(card_search::SEARCHING),
        Status::NoMatch => Some(card_search::NO_MATCH),
        Status::RateLimited => Some(card_search::RATE_LIMITED),
        Status::Failed => Some(card_search::FAILED),
    };
    rsx! {
        div { class: "quick-add web-home-search",
            div { class: "chip-row",
                label { class: "stat-chip stat-chip-p1 chip-row-label", r#for: SEARCH_ID, {card_search::LABEL} }
                ClearWrap {
                    filled: !query.is_empty(),
                    on_clear: move |()| driver.dispatch(Action::Clear),
                    label: "Clear the search".to_string(),
                    input {
                        class: "input input-compact",
                        id: SEARCH_ID,
                        r#type: "search",
                        placeholder: card_search::PLACEHOLDER,
                        value: "{query}",
                        autocapitalize: "none",
                        autocorrect: "off",
                        spellcheck: "false",
                        oninput: move |event| driver.dispatch(Action::Type(event.value())),
                    }
                }
            }
            if let Some(line) = line {
                div { class: "card-filter-results quick-add-results",
                    span { class: "chip-unselected", {line} }
                    if status == Status::Failed {
                        button {
                            r#type: "button",
                            class: "chip-unselected card-filter-result",
                            onclick: move |_| driver.dispatch(Action::Retry),
                            "Retry"
                        }
                    }
                }
            } else if !results.is_empty() {
                div { class: "card-filter-results quick-add-results",
                    for (i, (id, name)) in results.into_iter().enumerate() {
                        button {
                            key: "{id}",
                            r#type: "button",
                            class: "chip-unselected card-filter-result",
                            style: "animation-delay: {i.saturating_mul(40)}ms;",
                            onclick: move |_| driver.dispatch(Action::Pick(id)),
                            "{name}"
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_name_opens_its_card_page_and_a_name_with_no_slug_its_printing() {
        let slug = zwipe_core::domain::card::slug::card_slug("Sol Ring").unwrap();
        assert_eq!(
            route_of(Destination::Card(slug)),
            Some(Route::CardNamed {
                slug: "sol-ring".to_owned(),
                open: None
            })
        );
        let id = Uuid::new_v4();
        let path = zwipe_core::domain::card::slug::printing_path("_____", id);
        assert!(
            matches!(route_of(Destination::Printing { path: path.clone() }), Some(Route::CardPrinting { id: got, open: None, .. }) if got == id),
            "{path}"
        );
    }
}
