//! The card search's results page at `/search?q=<text>`, signed in or out, over the
//! [`card_results`] model: every card the lookup matches for the text, in the card page's
//! several-cards grid, each opening its card page, and a ghost of the grid while they load. A
//! text only one card matches opens that card's page in this page's place.
//!
//! On a keyboard the deck page's cursor walks the grid as laid out
//! ([`use_card_grid_cursor`]), Enter opens the cursor's card, and the card keys act on it, else
//! on the hovered card: `d` its details, `p` its Printings, Shift+P its Send to and `v` its other
//! face. Shift+H lists them.

use std::collections::HashMap;

use components::deck_cursor::use_card_grid_cursor;
use components::hint::ContentHintDialog;
use components::keys::use_page_keys_through;
use components::{
    CardResultsGrid, CardResultsGridGhost, LoadFailed, MeldSource, MeldToggle, focus_card_search,
    next_card_face,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_ui::hint::search_results as hint_copy;
use zwipe_ui::screens::card_results::{self, Action, Effect, State, Status};

use crate::inbound::cards::card_info::{
    CardDetailsSheet, DETAILS_DRAWER_CLASS, THROUGH_DRAWER_KEYS,
};
use crate::inbound::cards::card_search::{lookup_cards, route_of};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::row_keys::key_card_js;
use crate::inbound::cards::send_to::{SendToHost, use_send_to};
use crate::inbound::chrome::shortcuts::{SearchResultsKey, ShortcutsScreen, search_results_key};
use crate::inbound::chrome::shortcuts_sheet::use_offer_shortcuts;
use crate::inbound::router::Route;
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
    let session = driver.session;
    let signed_in = move || session.read().as_ref().is_some_and(|s| !s.is_expired());
    let send_to = use_send_to();
    // The face each result shows, which `v` and the details drawer's Flip turn.
    let faces: Signal<HashMap<Uuid, usize>> = use_signal(HashMap::new);
    // The card whose details or Printings are open, from a key.
    let mut details_card: Signal<Option<Card>> = use_signal(|| None);
    let mut details_open = use_signal(|| false);
    let mut printings_card: Signal<Option<Card>> = use_signal(|| None);
    let mut printings_open = use_signal(|| false);
    let mut hint_open = use_signal(|| false);
    let navigator = driver.navigator;
    let on_key = use_callback(
        move |(pressed, card): (SearchResultsKey, Card)| match pressed {
            SearchResultsKey::Details => {
                details_card.set(Some(card));
                details_open.set(true);
            }
            SearchResultsKey::Printings => {
                printings_card.set(Some(card));
                printings_open.set(true);
            }
            SearchResultsKey::SendTo => {
                if signed_in() {
                    send_to.open_for(card);
                } else {
                    navigator.push(Route::Login {});
                }
            }
            SearchResultsKey::Flip | SearchResultsKey::Search => {}
        },
    );

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
                SearchResultsKeys {
                    model: driver.model,
                    faces,
                    details_open,
                    details_card,
                    on_key,
                }
                CardResultsGrid {
                    title: title.clone(),
                    cards,
                    faces: faces(),
                    on_select: move |id: Uuid| dispatch(Action::Pick(id)),
                }
            }
        },
        Status::Loading => rsx! {
            div { class: "screen-content web-card-list", CardResultsGridGhost {} }
        },
        Status::NoMatch => rsx! {
            div { class: "screen-content web-card-list",
                p { class: "web-card-message",
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

    rsx! {
        ScreenFrame {
            title,
            titled: status == Status::Several,
            on_hint: move |()| hint_open.set(true),
            {body}
        }
        if let Some(card) = details_card() {
            CardDetailsSheet {
                open: details_open,
                face: faces.read().get(&card.scryfall_data.id).copied().unwrap_or(0),
                on_face_change: {
                    let id = card.scryfall_data.id;
                    let mut faces = faces;
                    move |next: usize| {
                        faces.write().insert(id, next);
                    }
                },
                on_printings: {
                    let card = card.clone();
                    move |()| {
                        printings_card.set(Some(card.clone()));
                        printings_open.set(true);
                    }
                },
                card,
            }
        }
        if let Some(card) = printings_card() {
            PrintingSheet {
                face: faces.read().get(&card.scryfall_data.id).copied().unwrap_or(0),
                card,
                open: printings_open,
            }
        }
        ContentHintDialog { open: hint_open, content: hint_copy::content() }
        if signed_in() {
            SendToHost { send_to }
        }
    }
}

/// The results' keys while the grid shows: the cursor over it, the card keys and `/`, offered on
/// Shift+H. A card key acts on `details_card` while its details drawer is open (only `v` reaches
/// the page through it), else on the cursor's card in `model`'s results after a key, else on the
/// hovered one. `v` turns the card in `faces`, or toggles a meld half's Meld; every other card key
/// goes to `on_key`.
#[component]
fn SearchResultsKeys(
    model: Signal<State>,
    faces: Signal<HashMap<Uuid, usize>>,
    details_open: Signal<bool>,
    details_card: Signal<Option<Card>>,
    on_key: Callback<(SearchResultsKey, Card)>,
) -> Element {
    use_card_grid_cursor();
    use_offer_shortcuts(ShortcutsScreen::SearchResults);
    let melds = try_use_context::<MeldSource>();
    use_page_keys_through(
        DETAILS_DRAWER_CLASS,
        &THROUGH_DRAWER_KEYS,
        Callback::new(move |key: String| {
            let Some(pressed) = search_results_key(&key) else {
                return;
            };
            if pressed == SearchResultsKey::Search {
                focus_card_search();
                return;
            }
            spawn(async move {
                let open = (*details_open.peek())
                    .then(|| details_card.peek().clone())
                    .flatten();
                let card = if let Some(card) = open {
                    Some(card)
                } else {
                    let cards = model.peek().cards().to_vec();
                    keyed_card(&cards).await
                };
                let Some(card) = card else {
                    return;
                };
                if pressed != SearchResultsKey::Flip {
                    on_key.call((pressed, card));
                    return;
                }
                let id = card.scryfall_data.id;
                let face = faces.peek().get(&id).copied().unwrap_or(0);
                if let Some(next) = next_card_face(&card, face) {
                    let mut faces = faces;
                    faces.write().insert(id, next);
                } else if card.scryfall_data.meld().is_some() {
                    MeldToggle::of(&card.scryfall_data, melds).toggle();
                }
            });
        }),
    );
    rsx! {}
}

/// The card the keys act on: the cursor's after a key, else the hovered one, from `cards`.
async fn keyed_card(cards: &[Card]) -> Option<Card> {
    let found = document::eval(&key_card_js()).recv::<String>().await.ok()?;
    let (_, id) = found.split_once(':')?;
    let id = Uuid::try_parse(id).ok()?;
    cards.iter().find(|c| c.scryfall_data.id == id).cloned()
}
