//! The card search's results page at `/search?q=<text>`, signed in or out, over the
//! [`card_results`] model: every card the lookup matches for the text, in the card page's
//! several-cards grid, each opening its card page, and a ghost of the grid while they load. A
//! text only one card matches opens that card's page in this page's place.
//!
//! On a keyboard the deck page's cursor walks the grid as laid out
//! ([`use_card_grid_cursor`]), Enter opens the cursor's card, and the card keys act on it, else
//! on the hovered card, else on the card whose details are open: `d` opens or closes its details
//! in the deck page's Stacks drawer ([`DeckStackDrawer`]), a side drawer with no scrim that the
//! cursor keys pass through and that swaps to each card the cursor lands on; `i` its image, `p`
//! its Printings, Shift+P its Send to and `v` its other face. Shift+H lists them.

use std::collections::HashMap;

use components::deck_cursor::use_card_grid_cursor;
use components::hint::ContentHintDialog;
use components::keys::use_page_keys_through;
use components::list_end::{ListEnd, use_list_end};
use components::{
    CardResultsGrid, CardResultsGridGhost, DeckRowHost, DeckStackDrawer, LoadFailed, MeldSource,
    MeldToggle, StackDetails, focus_card_search, next_card_face, use_stack_details,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::Reveal;
use zwipe_ui::hint::search_results as hint_copy;
use zwipe_ui::screens::card_results::{self, Action, Effect, State, Status};

use crate::inbound::cards::card_row::{CardOverlays, CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::card_search::{lookup_cards, route_of};
use crate::inbound::cards::row_keys::{ROW_KEYS, key_card_js};
use crate::inbound::cards::send_to::{SendToHost, use_send_to};
use crate::inbound::cards::stack_view::use_stack_details_sync;
use crate::inbound::chrome::shortcuts::{
    STACK_DETAILS_CLASS, SearchResultsKey, ShortcutsScreen, search_results_key,
};
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
    // The open card's details, in the deck page's Stacks drawer.
    let reveal = use_signal(Reveal::default);
    let details_open = use_signal(|| false);
    let details = use_stack_details(reveal, details_open);
    let listed = use_memo(move || {
        driver
            .model
            .read()
            .cards()
            .iter()
            .map(|c| c.scryfall_data.id)
            .collect::<Vec<Uuid>>()
    });
    use_stack_details_sync(reveal, details_open, listed);
    // The image preview and the Printings, to look at only.
    let overlays = use_card_overlays();
    let CardOverlays {
        mut preview,
        mut printing,
    } = overlays;
    let mut hint_open = use_signal(|| false);
    let navigator = driver.navigator;
    let on_key = use_callback(
        move |(pressed, card): (SearchResultsKey, Card)| match pressed {
            SearchResultsKey::Details => details.click(card.scryfall_data.id),
            SearchResultsKey::Printings => printing.set(Some(card)),
            SearchResultsKey::Image => {
                let face = faces
                    .peek()
                    .get(&card.scryfall_data.id)
                    .copied()
                    .unwrap_or(0);
                preview.set(Some((card.scryfall_data, face)));
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
    let has_more = state.has_more();
    drop(state);
    let rows: Vec<RowInfo> = cards
        .iter()
        .map(|card| RowInfo {
            card: card.clone(),
            qty: 1,
            kind: RowKind::Main { mvp: false },
        })
        .collect();
    let host = DeckRowHost {
        on_image: Some(Callback::new(move |shown: (ScryfallData, usize)| {
            preview.set(Some(shown));
        })),
        on_printing: None,
        on_view_printing: Some(Callback::new(move |card: Card| printing.set(Some(card)))),
        describe_tag: None,
        on_examples: None,
    };

    let body = match status {
        Status::Several => rsx! {
            div { class: "screen-content content-enter web-card-list",
                SearchResultsKeys {
                    model: driver.model,
                    faces,
                    details,
                    on_key,
                    on_end: move |()| dispatch(Action::LoadMore),
                }
                CardResultsGrid {
                    title: title.clone(),
                    cards,
                    faces: faces(),
                    on_select: move |id: Uuid| dispatch(Action::Pick(id)),
                }
                // Reaching it, by scroll or End, loads the next page.
                if has_more {
                    ListEnd {}
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
        // The open card's details, the deck page's Stacks drawer: no scrim, and the cursor keys
        // pass through it.
        DeckStackDrawer {
            rows,
            boards: HashMap::new(),
            details,
            faces,
            actions: None,
            host,
        }
        CardOverlaysHost { overlays }
        ContentHintDialog { open: hint_open, content: hint_copy::content() }
        if signed_in() {
            SendToHost { send_to }
        }
    }
}

/// The results' keys while the grid shows: the cursor over it, which the details drawer
/// follows, the card keys and `/`, offered on Shift+H. The card keys reach the page through the
/// drawer, as on the deck page. A card key acts on the cursor's card in `model`'s results after a
/// key, else on the hovered one, else on the card whose details are open. `v` turns the card in
/// `faces`, or toggles a meld half's Meld; every other card key goes to `on_key`. Reaching the
/// list's end, by scroll or End, calls `on_end`.
#[component]
fn SearchResultsKeys(
    model: Signal<State>,
    faces: Signal<HashMap<Uuid, usize>>,
    details: StackDetails,
    on_key: Callback<(SearchResultsKey, Card)>,
    on_end: Callback<()>,
) -> Element {
    use_card_grid_cursor(Some(details));
    use_offer_shortcuts(ShortcutsScreen::SearchResults);
    // The list's end, scrolled to or reached with End, loads the next page.
    use_list_end(on_end);
    let melds = try_use_context::<MeldSource>();
    use_page_keys_through(
        STACK_DETAILS_CLASS,
        &ROW_KEYS,
        Callback::new(move |key: String| {
            let Some(pressed) = search_results_key(&key) else {
                return;
            };
            if pressed == SearchResultsKey::Search {
                focus_card_search();
                return;
            }
            spawn(async move {
                let cards = model.peek().cards().to_vec();
                let open = details
                    .reveal
                    .peek()
                    .open()
                    .filter(|_| *details.details_open.peek())
                    .and_then(|id| cards.iter().find(|c| c.scryfall_data.id == id).cloned());
                let card = keyed_card(&cards).await.or(open);
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

#[cfg(test)]
mod tests {
    /// The page's source, for the wiring it shares with the deck page.
    const SOURCE: &str = include_str!("search.rs");

    #[test]
    fn d_opens_the_deck_page_s_stacks_drawer_that_the_cursor_keys_pass_through() {
        assert!(
            SOURCE.contains("DeckStackDrawer {"),
            "the Stacks drawer, not a copy"
        );
        assert!(
            SOURCE.contains("use_card_grid_cursor(Some(details));"),
            "the drawer follows the cursor"
        );
        assert!(
            SOURCE.contains("STACK_DETAILS_CLASS,\n        &ROW_KEYS,"),
            "the card keys reach the page through the drawer, as on the deck page"
        );
        assert!(
            SOURCE.contains("SearchResultsKey::Details => details.click(card.scryfall_data.id),"),
            "d opens the card's details, or closes them where they are open"
        );
    }

    #[test]
    fn i_opens_the_image_view_the_deck_page_s_i_opens() {
        assert!(SOURCE.contains("CardOverlaysHost { overlays }"));
        assert!(SOURCE.contains("preview.set(Some((card.scryfall_data, face)));"));
    }
}
