//! The card search's results page in the app at `/search?q=<text>`, over the shared
//! [`card_results`] model: every card the lookup matches for the text, in the card page's
//! results grid, each opening its card page, and a ghost of the grid while they load. A text
//! only one card matches opens that card's page in this page's place.

use components::list_end::{ListEnd, use_list_end};
use components::results_place::{self, KeptPlace, use_results_scroll, use_submits};
use components::{CardResultsGrid, CardResultsGridGhost, LoadFailed};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_ui::screens::card_results::{self, Action, Effect, State, Status};

use crate::inbound::components::card_search::{lookup_cards, route_of};
use crate::inbound::components::screen_header::ScreenHeader;

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
    let navigator = use_navigator();
    let driver = Driver {
        model: use_signal(State::new),
        session: use_context(),
        client: use_context(),
        navigator,
    };
    let dispatch = move |action: Action| driver.dispatch(action);

    // Back from a card finds the results it kept, every page loaded, scrolled where they were;
    // anything else looks the text up.
    let mut scroll = use_signal(|| 0.0_f64);
    use_results_scroll(Callback::new(move |y: f64| scroll.set(y)));
    use_effect(use_reactive!(|q| {
        let signed_in = driver
            .session
            .peek()
            .as_ref()
            .is_some_and(|s| !s.is_expired());
        if let Some(place) = results_place::kept_for(&q, signed_in) {
            dispatch(Action::Resume(place.kept));
            results_place::restore_place(place.scroll);
        } else {
            results_place::fresh_start();
            dispatch(Action::Load(q));
        }
    }));
    // A submit of the text already on screen changes no route: it loads page 1 again, at the top.
    use_submits(Callback::new(move |text: String| {
        if text == driver.model.peek().query() {
            results_place::fresh_start();
            dispatch(Action::Load(text));
        }
    }));
    use_drop(move || {
        if let Some(kept) = driver.model.peek().kept() {
            results_place::keep(KeptPlace {
                kept,
                scroll: *scroll.peek(),
                signed_in: driver
                    .session
                    .peek()
                    .as_ref()
                    .is_some_and(|s| !s.is_expired()),
            });
        }
    });

    let state = driver.model.read();
    let status = state.status();
    let cards = state.cards().to_vec();
    let title = state.title();
    let has_more = state.has_more();
    let dealt_from = state.dealt_from();
    drop(state);

    let body = match status {
        Status::Several => rsx! {
            div { class: "screen-content content-enter card-page-list",
                NextPageOnEnd { on_end: move |()| dispatch(Action::LoadMore) }
                CardResultsGrid {
                    title: title.clone(),
                    cards,
                    on_select: move |id: Uuid| dispatch(Action::Pick(id)),
                    dealt_from,
                }
                // Scrolling to it loads the next page.
                if has_more {
                    ListEnd {}
                }
            }
        },
        Status::Loading => rsx! {
            div { class: "screen-content card-page-list", CardResultsGridGhost {} }
        },
        Status::NoMatch => rsx! {
            div { class: "screen-content card-page-list",
                p { class: "card-page-message",
                    span { class: "chip-note", {card_results::NO_MATCH} }
                }
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

    // The grid draws its own heading, so the header leaves its title out, as the web's nav does.
    let header = if status == Status::Several {
        String::new()
    } else {
        title
    };
    rsx! {
        div { class: "screen",
            ScreenHeader { title: header, back: move |()| navigator.go_back() }
            {body}
        }
    }
}

/// Calls `on_end` as the results' end scrolls near the view ([`use_list_end`]), while the grid
/// shows.
#[component]
fn NextPageOnEnd(on_end: Callback<()>) -> Element {
    use_list_end(on_end);
    rsx! {}
}
