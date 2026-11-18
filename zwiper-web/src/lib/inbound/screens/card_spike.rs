//! SPIKE, never merged: the card screen from the deck page's own grammar, for Scotty to judge
//! the look. On a wide window the card sits in the page's middle column and its details dock at
//! the right, as on Add and Remove; Send to and Share are the floating action dock at every
//! width. A phone shows the details under the card. `look` is ignored.

use components::icons::{DockIcon, SEND, ShareIcon};
use components::{
    CardDetailsPanel, CardStack, FlippableCardImage, LoadFailed, Panel, PickerSearch, StackCard,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::models::search_card::commander_eligibility;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageSize;
use zwipe_core::domain::card::slug::card_slug;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_ui::screens::card_lookup::{self, Action, Destination, Effect, Open, State, Status};

use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// One of the card page's counters: its number, its label and, for a rate, how many swipes it
/// counts.
struct Counter {
    num: &'static str,
    label: &'static str,
    sub: Option<&'static str>,
}

/// Every card's counters, in the order they show, with sample numbers: the real ones are still
/// being worked out.
const COUNTERS: [Counter; 7] = [
    Counter {
        num: "312",
        label: "In decks",
        sub: None,
    },
    Counter {
        num: "41",
        label: "MVP in decks",
        sub: None,
    },
    Counter {
        num: "72%",
        label: "Add rate",
        sub: Some("1,240 added"),
    },
    Counter {
        num: "18%",
        label: "Skip rate",
        sub: Some("310 skipped"),
    },
    Counter {
        num: "4%",
        label: "Remove rate",
        sub: Some("69 removed"),
    },
    Counter {
        num: "6%",
        label: "Maybe rate",
        sub: Some("103 maybeboarded"),
    },
    Counter {
        num: "1,722",
        label: "Swipes",
        sub: None,
    },
];

/// The counters a card that can lead a Commander deck adds after the others, with sample numbers.
const COMMANDER_COUNTERS: [Counter; 2] = [
    Counter {
        num: "87",
        label: "Decks led",
        sub: None,
    },
    Counter {
        num: "9%",
        label: "Pick rate",
        sub: Some("860 picked"),
    },
];

/// The spike's only rules: the card's size in the middle column, the details under the card on a
/// phone (the deck page hides them behind the eye there), and the list's width.
const SPIKE_CSS: &str = r"
.cs-search { width: 100%; max-width: 30rem; }
.cs-image { width: 100%; max-width: 26rem; }
.cs-image img { width: 100%; height: auto; }
.cs-list { max-width: 40rem; margin: 0 auto; padding: var(--space-3) var(--space-4); box-sizing: border-box; width: 100%; display: flex; flex-direction: column; gap: var(--space-2); }
.cs-results { width: 100%; grid-template-columns: repeat(auto-fit, minmax(12.5rem, 15rem)); justify-content: center; }
.cs-metrics { width: 100%; max-width: 20rem; display: flex; flex-direction: column; gap: var(--space-2); }
.cs-metrics .stats-strip { grid-template-columns: repeat(2, minmax(0, 1fr)); }
.cs-metrics .stat-sub { font-size: 0.65rem; opacity: 0.55; text-align: center; }
@media (min-width: 900px) {
  .cs-metrics { position: fixed; top: var(--web-dock-panel-top); left: 1rem; z-index: 40; width: var(--web-dock-panel-width); max-width: none; }
  .cs-list { max-width: 60rem; }
}
.cs-message { text-align: center; padding: var(--space-5) 0; }
@media (max-width: 899px) {
  .cs-card { display: flex; flex-direction: column; gap: var(--space-4); padding: var(--space-3) var(--space-4); }
  .cs-card .web-cards-swipe-main { display: flex; flex-direction: column; align-items: center; gap: var(--space-3); }
  .cs-card .cs-image { max-width: 20rem; }
  .cs-card .web-swipe-details { display: block; width: 100%; max-width: 20rem; }
  .cs-card .web-swipe-details > .card-details-panel { margin: 0; width: 100%; box-sizing: border-box; }
}
";

/// SPIKE: `/card/:slug?:look`.
#[component]
pub fn CardSpike(slug: String, look: String) -> Element {
    rsx! { SpikeScreen { key: "{slug}", open: Open::Named { slug }, look } }
}

/// SPIKE: `/card/:slug/:id?:look`.
#[component]
pub fn CardSpikePrinting(slug: String, id: Uuid, look: String) -> Element {
    rsx! { SpikeScreen { key: "{id}", open: Open::Printing { slug, id }, look } }
}

#[component]
fn SpikeScreen(open: Open, look: String) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let navigator = use_navigator();
    let toast = components::use_toast();
    let mut model = use_signal(State::new);
    let mut printings_open = use_signal(|| false);
    // The card page's folds shut by the user; the stats fold starts open.
    let signed_in = move || session.read().as_ref().is_some_and(|s| !s.is_expired());

    let route_of = {
        let look = look;
        move |to: Destination| match to {
            Destination::Named { slug, .. } => Route::CardSpike {
                slug: slug.to_string(),
                look: look.clone(),
            },
            Destination::Printing { slug, id, .. } => Route::CardSpikePrinting {
                slug,
                id,
                look: look.clone(),
            },
            Destination::Login => Route::Login {},
        }
    };

    let dispatch = use_callback(move |action: Action| {
        let effects = model.write().update(action);
        for effect in effects {
            match effect {
                Effect::FetchNamed { ticket, slug } => {
                    spawn(async move {
                        let http = client.peek().clone();
                        let reply = match http.fetch_named_cards(&slug).await {
                            Ok(cards) => Action::Named { ticket, cards },
                            Err(_) => Action::Failed { ticket },
                        };
                        let effects = model.write().update(reply);
                        drop(effects);
                    });
                }
                Effect::FetchCard { ticket, id } => {
                    spawn(async move {
                        let http = client.peek().clone();
                        let reply = match http.fetch_card(id).await {
                            Ok(card) => Action::Printing {
                                ticket,
                                card: Box::new(card),
                            },
                            Err(ClientError::NotFound(_)) => Action::Missing { ticket },
                            Err(_) => Action::Failed { ticket },
                        };
                        let effects = model.write().update(reply);
                        drop(effects);
                    });
                }
                Effect::Replace(to) => {
                    navigator.replace(route_of(to));
                }
                Effect::Navigate(to) => {
                    navigator.push(route_of(to));
                }
                Effect::RememberReturn(_) => {}
                Effect::OpenSendTo(_) => {
                    toast.info(
                        "Send to opens here".to_string(),
                        components::ToastOptions::default(),
                    );
                }
                Effect::Copy { text, done } => {
                    document::eval(&format!("navigator.clipboard?.writeText({text:?})"));
                    toast.info(done.text, components::ToastOptions::default());
                }
            }
        }
    });

    // Loads on mount and again whenever the route opens another card in place.
    use_effect(use_reactive!(|(open,)| {
        dispatch.call(Action::Load {
            open,
            send: false,
            signed_in: signed_in(),
        });
    }));

    let query = use_signal(String::new);
    let search = rsx! {
        div {
            class: "cs-search",
            onkeydown: move |evt| {
                if evt.key() == Key::Enter
                    && let Some(slug) = card_slug(&query.peek())
                {
                    navigator.push(Route::CardSpike {
                        slug: slug.to_string(),
                        look: String::new(),
                    });
                }
            },
            PickerSearch { id: "card-spike-search", placeholder: "Card name", query }
        }
    };

    let state = model.read();
    let status = state.status();
    let card = state.card().cloned();
    let cards = state.cards().to_vec();
    let message = state.message();
    drop(state);
    let frame_title = card
        .as_ref()
        .map_or_else(|| "Card".to_owned(), |c| c.scryfall_data.name.clone());

    let body = match (status, card) {
        (Status::One, Some(card)) => {
            let sd = card.scryfall_data.clone();
            let leads = commander_eligibility::is_valid_commander(&card, &Format::Commander);
            let counters = COUNTERS
                .iter()
                .chain(COMMANDER_COUNTERS.iter().filter(|_| leads));
            rsx! {
                div { class: "screen-content content-enter web-cards-swipe cs-card",
                    div { class: "web-cards-swipe-main",
                        {search}
                        div { class: "cs-image",
                            FlippableCardImage { sd, size: ImageSize::Large }
                        }
                    }
                    div { class: "web-swipe-details",
                        CardDetailsPanel {
                            card: card.clone(),
                            send_to: false,
                            on_printings: move |()| printings_open.set(true),
                        }
                    }
                    // The card's numbers as Activity's counters, two to a row: the left column on a
                    // wide window, under the details on a phone. Static markup, since the kit's
                    // counter rolls digits while a value loads.
                    div { class: "cs-metrics",
                        div { class: "home-hero activity-counts",
                            Panel {
                                div { class: "stats-figures",
                                    section { class: "stats-strip",
                                        for counter in counters {
                                            div { key: "{counter.label}", class: "stat",
                                                span { class: "stat-num", "{counter.num}" }
                                                span { class: "stat-label", "{counter.label}" }
                                                if let Some(sub) = counter.sub {
                                                    span { class: "stat-sub", "{sub}" }
                                                }
                                            }
                                        }
                                    }
                                    div { class: "stats-source",
                                        span { class: "stat-chip", "Sample numbers" }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "action-dock action-dock-right",
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Send to",
                        onclick: move |_| dispatch.call(Action::SendTo { signed_in: signed_in() }),
                        DockIcon { path: SEND }
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Share",
                        onclick: move |_| dispatch.call(Action::Share),
                        ShareIcon {}
                    }
                }
                PrintingSheet { card, open: printings_open }
            }
        }
        (Status::Several, _) => rsx! {
            div { class: "screen-content content-enter cs-list",
                {search}
                // Each result is a one-card stack in the deck page's Stacks grid: the whole card
                // shows, its name on its own frame, and a tap opens it.
                div { class: "deck-stacks cs-results",
                    for listed in cards {
                        div { key: "{listed.scryfall_data.id}", class: "cs-result",
                            CardStack {
                                cards: vec![StackCard { card: listed, qty: 1, face: 0, board: None, starred: false }],
                                on_select: move |id: Uuid| dispatch.call(Action::Pick(id)),
                            }
                        }
                    }
                }
            }
        },
        (Status::NotFound, _) => rsx! {
            div { class: "screen-content cs-list",
                {search}
                p { class: "cs-message", {message.unwrap_or(card_lookup::NOT_FOUND)} }
            }
        },
        (Status::Failed, _) => rsx! {
            div { class: "screen-content",
                LoadFailed { message: "Couldn't load this card", on_retry: move |()| dispatch.call(Action::Retry) }
            }
        },
        _ => rsx! {
            div { class: "screen-content web-cards-swipe cs-card",
                div { class: "web-cards-swipe-main", {search} }
                div { class: "web-swipe-details", components::CardDetailsPanelSkeleton {} }
            }
        },
    };

    rsx! {
        ScreenFrame { title: frame_title,
            style { {SPIKE_CSS} }
            {body}
        }
    }
}
