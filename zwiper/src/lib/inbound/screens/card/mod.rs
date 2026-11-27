//! The card page in the app at `/card/<slug>` and `/card/<slug>/<id>`, over the shared
//! `card_lookup` model: the web's card page at phone width. One card shows alone, filling the
//! screen, with Send to, Share and the eye in the action dock; the eye opens its details drawer,
//! as on the swipe screens, with Printings and its stats folded at the end.
//! Several cards that share a name show in the results grid, each opening its printing. Share
//! copies the card's link. A printing picked in Printings takes the card's place, for Send to
//! and Share too.

pub mod search;

use components::hint::ContentHintDialog;
use components::icons::{DockIcon, EyeIcon, SEND, ShareIcon};
use components::{
    CardDetailsPanelSkeleton, CardResultsGrid, FlippableCardImage, LoadFailed, SendTo, Toasts,
    use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageSize;
use zwipe_ui::hint::card_page as hint_copy;
use zwipe_ui::screens::add_cards;
use zwipe_ui::screens::card_lookup::{self, Action, Destination, Effect, Open, State, Status};

use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::screen;
use crate::inbound::components::ui_toast;
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::card_info::CardDetailsSheet;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// `/card/<slug>`: every card with the name.
#[component]
pub fn CardNamed(slug: String) -> Element {
    rsx! {
        CardScreen { key: "{slug}", open: Open::Named { slug } }
    }
}

/// `/card/<slug>/<id>`: one printing.
#[component]
pub fn CardPrinting(slug: String, id: Uuid) -> Element {
    rsx! {
        CardScreen { key: "{id}", open: Open::Printing { slug, id } }
    }
}

/// The route a model destination opens. The app is always signed in, so Login and Register
/// are its own screens only for completeness.
fn route_of(to: Destination) -> Router {
    match to {
        Destination::Named { slug, .. } => Router::CardNamed {
            slug: slug.to_string(),
        },
        Destination::Printing { slug, id, .. } => Router::CardPrinting { slug, id },
        Destination::Login => Router::Login {},
        Destination::Register => Router::Register {},
    }
}

/// The page's model with the handles that perform its effects. Copy, so every handler and
/// task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    client: Signal<ZwipeClient>,
    navigator: Navigator,
    toasts: Toasts,
    send_to: SendTo,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns, a fetch's reply included.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one of the model's effects.
    fn perform(self, effect: Effect) {
        let Self {
            client,
            navigator,
            toasts,
            send_to,
            ..
        } = self;
        match effect {
            Effect::FetchNamed { ticket, slug } => {
                spawn(async move {
                    let http = client.peek().clone();
                    let reply = match http.fetch_named_cards(&slug).await {
                        Ok(cards) => Action::Named { ticket, cards },
                        Err(_) => Action::Failed { ticket },
                    };
                    self.dispatch(reply);
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
                    self.dispatch(reply);
                });
            }
            Effect::Replace(to) => {
                navigator.replace(route_of(to));
            }
            Effect::Navigate(to) => {
                navigator.push(route_of(to));
            }
            // A sign-in never comes between, as the app's screens are signed in, and the app's
            // card routes carry no printing: the pick lives in the model.
            Effect::RememberReturn(_) | Effect::ShowPrinting(_) => {}
            Effect::OpenSendTo { card, picked } => send_to.open_with(*card, picked),
            Effect::FetchPrinting { ticket, id } => {
                spawn(async move {
                    let http = client.peek().clone();
                    let reply = match http.fetch_card(id).await {
                        Ok(card) => Action::PrintingPicked {
                            ticket,
                            card: Box::new(card),
                        },
                        Err(_) => Action::PrintingUnknown { ticket },
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Copy { text, done } => {
                let js = format!(
                    "navigator.clipboard.writeText({})",
                    serde_json::to_string(&text).unwrap_or_default()
                );
                document::eval(&js);
                ui_toast::raise(toasts, done);
            }
        }
    }
}

/// The card page for `open`.
#[component]
fn CardScreen(open: Open) -> Element {
    let navigator = use_navigator();
    let driver = Driver {
        model: use_signal(State::new),
        client: use_context(),
        navigator,
        toasts: use_toast(),
        send_to: use_context(),
    };
    let dispatch = move |action: Action| driver.dispatch(action);
    let mut printings_open = use_signal(|| false);
    // The card's details drawer, which the eye in the dock opens, as on the swipe screens.
    let mut show_details = use_signal(|| false);
    let hint_open = use_signal(|| false);
    // The role tags' example cards, which a tag in the drawer opens.
    let mut otag_examples_open = use_signal(|| false);
    let mut otag_examples_slug = use_signal(String::new);
    let open_examples = use_callback(move |slug: String| {
        otag_examples_slug.set(slug);
        otag_examples_open.set(true);
    });

    // Loads on mount and again whenever the route opens another card in place.
    use_effect(use_reactive!(|open| {
        dispatch(Action::Load {
            open,
            send: false,
            signed_in: true,
            printing: None,
        });
    }));

    let state = driver.model.read();
    let status = state.status();
    let card = state.card().cloned();
    let cards = state.cards().to_vec();
    let message = state.message();
    let title = state.screen_title();
    drop(state);

    let body = match (status, card) {
        (Status::One, Some(card)) => {
            let sd = card.scryfall_data.clone();
            rsx! {
                div { class: "screen-content content-enter card-page",
                    div { class: "card-page-image",
                        FlippableCardImage { sd, size: ImageSize::Large }
                    }
                }
                div { class: "action-dock action-dock-right",
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Send to",
                        onclick: move |_| dispatch(Action::SendTo { signed_in: true }),
                        DockIcon { path: SEND }
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Share",
                        onclick: move |_| dispatch(Action::Share),
                        ShareIcon {}
                    }
                    span { class: "action-dock-divider" }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Card details",
                        onclick: move |_| show_details.toggle(),
                        EyeIcon {}
                    }
                }
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printings_open.set(true),
                    on_examples: open_examples,
                }
                PrintingSheet {
                    card,
                    open: printings_open,
                    host_screen: screen::CARD,
                    on_save: move |picked: Card| dispatch(Action::PickPrinting(Box::new(picked))),
                    saved_message: add_cards::PRINTING_SWAPPED,
                }
            }
        }
        (Status::Several, _) => rsx! {
            div { class: "screen-content content-enter card-page-list",
                CardResultsGrid {
                    title: title.clone(),
                    cards,
                    on_select: move |id: Uuid| dispatch(Action::Pick(id)),
                }
            }
        },
        (Status::NotFound, _) => rsx! {
            div { class: "screen-content card-page-list",
                p { class: "card-page-message",
                    span { class: "chip-note", {message.unwrap_or(card_lookup::NOT_FOUND)} }
                }
            }
        },
        (Status::Failed, _) => rsx! {
            div { class: "screen-content",
                LoadFailed {
                    message: "Couldn't load this card",
                    on_retry: move |()| dispatch(Action::Retry),
                }
            }
        },
        (Status::One | Status::Loading, _) => rsx! {
            div { class: "screen-content card-page",
                CardDetailsPanelSkeleton {}
            }
        },
    };

    // Several cards draw their own heading, so the header leaves its title out, as the web's
    // nav does.
    let header = if status == Status::Several {
        String::new()
    } else {
        title
    };
    rsx! {
        div { class: "screen",
            ScreenHeader {
                title: header,
                back: move |()| navigator.go_back(),
                hint: (status == Status::One).then_some(hint_open),
            }
            {body}
        }
        ContentHintDialog { open: hint_open, content: hint_copy::content(true) }
        if otag_examples_open() {
            OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
        }
    }
}
