//! The card search's results page at `/search?q=<text>`, signed in or out, over the
//! [`card_results`] model: every card the lookup matches for the text, in the card page's
//! several-cards grid, each opening its card page, and a ghost of the grid while they load. A
//! text only one card matches opens that card's page in this page's place.

use components::{CardResultsGrid, CardResultsGridGhost, LoadFailed};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::card_results::{self, Action, Effect, State, Status};

use crate::inbound::cards::card_search::{lookup_cards, route_of};
use crate::inbound::shell::ScreenFrame;

/// The page's model with the handles that perform its effects. Copy, so every handler and task
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
            Effect::Lookup { ticket, lookup } => {
                spawn(async move {
                    let reply = match lookup_cards(self.session, self.client, &lookup).await {
                        Ok(cards) => Action::Loaded { ticket, cards },
                        Err(failure) => Action::Failed { ticket, failure },
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Replace(to) => {
                if let Some(route) = route_of(to) {
                    self.navigator.replace(route);
                }
            }
            Effect::Navigate(to) => {
                if let Some(route) = route_of(to) {
                    self.navigator.push(route);
                }
            }
        }
    }
}

/// `/search?q=<text>`.
#[component]
pub fn Search(q: String) -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        session: use_context(),
        client: use_context(),
        navigator: use_navigator(),
    };
    let dispatch = move |action: Action| driver.dispatch(action);

    // Looks up on mount and again whenever the route's text changes in place.
    use_effect(use_reactive!(|q| dispatch(Action::Load(q))));

    let state = driver.model.read();
    let status = state.status();
    let cards = state.cards().to_vec();
    let title = state.title();
    drop(state);

    let body = match status {
        Status::Several => rsx! {
            div { class: "screen-content content-enter web-card-list",
                CardResultsGrid { title: title.clone(), cards, on_select: move |id: Uuid| dispatch(Action::Pick(id)) }
            }
        },
        Status::Loading => rsx! {
            div { class: "screen-content web-card-list", CardResultsGridGhost {} }
        },
        Status::NoMatch => rsx! {
            div { class: "screen-content web-card-list",
                p { class: "web-card-message", {card_results::NO_MATCH} }
            }
        },
        Status::RateLimited | Status::Failed => {
            let message = if status == Status::RateLimited {
                card_results::RATE_LIMITED
            } else {
                card_results::FAILED
            };
            rsx! {
                div { class: "screen-content",
                    LoadFailed { message, on_retry: move |()| dispatch(Action::Retry) }
                }
            }
        }
    };

    rsx! {
        ScreenFrame { title, titled: status == Status::Several, {body} }
    }
}
