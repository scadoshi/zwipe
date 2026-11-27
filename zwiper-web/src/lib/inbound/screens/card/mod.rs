//! The card screen at `/card/<slug>` and `/card/<slug>/<id>`, signed in or out, over the
//! [`card_lookup`] model.
//!
//! One card draws the deck page's wide card layout: the card in the middle column, its details
//! docked at the right and its stats widgets at the left; a phone stacks the card,
//! the details and the widgets. Send to and Share are the floating action dock, Create account in
//! Send to's place signed out. Several cards
//! that share a name show as the deck page's Stacks grid, each a whole card that opens its
//! printing. `?open=send-to` opens Send to once the card shows: a signed-out Send to goes to
//! Login and comes back with it. A printing picked in Printings takes the card's place and goes
//! in the route as `?printing=<id>`, so a reload or a shared link shows it.

use std::collections::HashSet;

use components::icons::{DockIcon, SEND, ShareIcon};
use components::{
    CardDetailsPanel, CardDetailsPanelSkeleton, CardResultsGrid, CardStatsWidget,
    FlippableCardImage, LoadFailed, SendTo, USER_PLUS, focus_card_search,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{Toasts, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::models::search_card::commander_eligibility;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageSize;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_ui::screens::add_cards::PRINTING_SWAPPED;
use zwipe_ui::screens::card_lookup::{
    self, Action, Ask, CardStats, Destination, Effect, Open, State, Status,
};
use zwipe_ui::telemetry::vocabulary::Screen;

use crate::inbound::cards::otag::{
    OtagDescribe, OtagExamplesHost, OtagExamplesOpen, use_otag_reveals,
};
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::send_to::{SendToHost, use_send_to};
use crate::inbound::chrome::shortcuts::{CardPageKey, ShortcutsScreen, card_page_key};
use crate::inbound::chrome::shortcuts_sheet::use_offer_shortcuts;
use crate::inbound::chrome::toast;
use crate::inbound::router::Route;
use crate::inbound::session::return_to;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;
use crate::outbound::browser;

/// `/card/<slug>`: every card with the name.
#[component]
pub fn CardNamed(slug: String, open: Option<Ask>, printing: Option<Uuid>) -> Element {
    rsx! {
        CardScreen { key: "{slug}", open: Open::Named { slug }, ask: open, printing }
    }
}

/// `/card/<slug>/<id>`: one printing.
#[component]
pub fn CardPrinting(slug: String, id: Uuid, open: Option<Ask>, printing: Option<Uuid>) -> Element {
    rsx! {
        CardScreen { key: "{id}", open: Open::Printing { slug, id }, ask: open, printing }
    }
}

/// The route a model destination opens.
fn route_of(to: Destination) -> Route {
    let ask = |send: bool| send.then_some(Ask::SendTo);
    match to {
        Destination::Named {
            slug,
            send,
            printing,
        } => Route::CardNamed {
            slug: slug.to_string(),
            open: ask(send),
            printing,
        },
        Destination::Printing {
            slug,
            id,
            send,
            printing,
        } => Route::CardPrinting {
            slug,
            id,
            open: ask(send),
            printing,
        },
        Destination::Login => Route::Login {},
        Destination::Register => Route::Register {},
    }
}

/// The card screen's model with the handles that perform its effects. Copy, so every handler and
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
            Effect::Replace(to) | Effect::ShowPrinting(to) => {
                navigator.replace(route_of(to));
            }
            Effect::Navigate(to) => {
                navigator.push(route_of(to));
            }
            Effect::RememberReturn(to) => return_to::remember(&route_of(to)),
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
                browser::copy_to_clipboard(&text);
                toast::raise(toasts, &done);
            }
            Effect::FetchStats { ticket, oracle_id } => {
                spawn(async move {
                    let http = client.peek().clone();
                    let reply = match http.fetch_card_stats(oracle_id).await {
                        Ok(stats) => Action::Stats { ticket, stats },
                        Err(_) => Action::StatsFailed { ticket },
                    };
                    self.dispatch(reply);
                });
            }
        }
    }
}

/// The card screen for `open`, opening Send to once the card shows when `ask` says so and
/// showing `printing` of the card in its place.
#[component]
fn CardScreen(open: Open, ask: Option<Ask>, printing: Option<Uuid>) -> Element {
    telemetry::use_telemetry_screen(Screen::Card);
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let navigator = use_navigator();
    let toasts = use_toast();
    let send_to = use_send_to();
    let model = use_signal(State::new);
    let mut printings_open = use_signal(|| false);
    // The stats folds the user shut; every one starts open.
    let stats_folded = use_signal(HashSet::<String>::new);
    // The role tags' descriptions and their example cards.
    let examples = use_otag_reveals();
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let signed_in = move || session.read().as_ref().is_some_and(|s| !s.is_expired());

    let driver = Driver {
        model,
        client,
        navigator,
        toasts,
        send_to,
    };
    let dispatch = move |action: Action| driver.dispatch(action);

    // Loads on mount and again whenever the route opens another card in place.
    use_effect(use_reactive!(|(open, ask, printing)| {
        dispatch(Action::Load {
            open,
            send: ask == Some(Ask::SendTo),
            signed_in: signed_in(),
            printing,
        });
    }));

    let state = model.read();
    let status = state.status();
    let card = state.card().cloned();
    let cards = state.cards().to_vec();
    let message = state.message();
    let stats = state.stats();
    let title = state.screen_title();
    drop(state);
    // Several cards draw their own heading; one card's name is on the card and its details.
    let titled = status == Status::Several;

    let body = match (status, card) {
        (Status::One, Some(card)) => {
            let sd = card.scryfall_data.clone();
            let leads = commander_eligibility::is_valid_commander(&card, &Format::Commander);
            rsx! {
                div { class: "screen-content content-enter web-cards-swipe web-card-page",
                    CardPageKeys {
                        on_printings: move |()| printings_open.set(true),
                        on_send_to: move |()| dispatch(Action::SendTo { signed_in: signed_in() }),
                    }
                    div { class: "web-cards-swipe-main",
                        div { class: "web-card-image",
                            FlippableCardImage { sd, size: ImageSize::Large }
                        }
                    }
                    div { class: "web-swipe-details",
                        CardDetailsPanel {
                            card: card.clone(),
                            send_to: false,
                            describe_tag,
                            on_examples,
                            on_printings: move |()| printings_open.set(true),
                        }
                    }
                    div { class: "web-card-stats",
                        match stats {
                            CardStats::Failed => rsx! {
                                LoadFailed {
                                    message: "Couldn't load the card's stats",
                                    on_retry: move |()| dispatch(Action::RetryStats),
                                }
                            },
                            CardStats::Loading | CardStats::Loaded(_) => {
                                let loaded = match stats {
                                    CardStats::Loaded(counts) => Some(counts),
                                    CardStats::Loading | CardStats::Failed => None,
                                };
                                rsx! {
                                    for widget in card_lookup::stats::widgets(loaded.as_ref(), leads) {
                                        CardStatsWidget { key: "{widget.title}", widget, folded: stats_folded }
                                    }
                                }
                            }
                        }
                    }
                }
                div { class: "action-dock action-dock-right",
                    if signed_in() {
                        button {
                            class: "action-dock-btn",
                            "aria-label": "Send to",
                            onclick: move |_| dispatch(Action::SendTo { signed_in: true }),
                            DockIcon { path: SEND }
                        }
                    } else {
                        button {
                            class: "action-dock-btn",
                            "aria-label": "Create account",
                            title: "Create account",
                            onclick: move |_| dispatch(Action::CreateAccount),
                            DockIcon { path: USER_PLUS }
                        }
                    }
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Share",
                        onclick: move |_| dispatch(Action::Share),
                        ShareIcon {}
                    }
                }
                PrintingSheet {
                    card,
                    open: printings_open,
                    on_save: move |picked: Card| dispatch(Action::PickPrinting(Box::new(picked))),
                    saved_message: PRINTING_SWAPPED,
                }
            }
        }
        (Status::Several, _) => rsx! {
            div { class: "screen-content content-enter web-card-list",
                CardResultsGrid { title: title.clone(), cards, on_select: move |id: Uuid| dispatch(Action::Pick(id)) }
            }
        },
        (Status::NotFound, _) => rsx! {
            div { class: "screen-content web-card-list",
                p { class: "web-card-message",
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
            div { class: "screen-content web-cards-swipe web-card-page",
                div { class: "web-swipe-details", CardDetailsPanelSkeleton {} }
            }
        },
    };

    rsx! {
        ScreenFrame { title, titled, {body} }
        OtagExamplesHost { examples }
        if signed_in() {
            SendToHost { send_to }
        }
    }
}

/// The keys of a card page showing one card, while it is mounted: `p` opens its Printings,
/// Shift+P its Send to and `/` puts the cursor in the card search, and Shift+H lists them.
#[component]
fn CardPageKeys(on_printings: EventHandler<()>, on_send_to: EventHandler<()>) -> Element {
    use_offer_shortcuts(ShortcutsScreen::CardPage);
    components::keys::use_page_keys(Callback::new(move |key: String| {
        match card_page_key(&key) {
            Some(CardPageKey::Printings) => on_printings.call(()),
            Some(CardPageKey::SendTo) => on_send_to.call(()),
            Some(CardPageKey::Search) => focus_card_search(),
            None => {}
        }
    }));
    rsx! {}
}

#[cfg(test)]
mod tests {
    use std::str::FromStr;

    use super::*;

    fn slug(name: &str) -> zwipe_core::domain::card::slug::CardSlug {
        zwipe_core::domain::card::slug::card_slug(name).unwrap()
    }

    #[test]
    fn a_name_without_an_ask_is_a_bare_card_link() {
        let route = route_of(Destination::Named {
            slug: slug("Sol Ring"),
            send: false,
            printing: None,
        });
        // The router prints an empty query as a bare `?`, as it does for every route.
        let printed = route.to_string();
        assert_eq!(
            printed.strip_suffix('?').unwrap_or(&printed),
            "/card/sol-ring"
        );
        assert_eq!(Route::from_str("/card/sol-ring").ok(), Some(route));
    }

    #[test]
    fn an_asked_for_send_to_rides_the_link_and_reads_back() {
        let id = Uuid::new_v4();
        for (route, path) in [
            (
                route_of(Destination::Named {
                    slug: slug("Sol Ring"),
                    send: true,
                    printing: None,
                }),
                "/card/sol-ring?open=send-to".to_owned(),
            ),
            (
                route_of(Destination::Printing {
                    slug: "sol-ring".to_owned(),
                    id,
                    send: true,
                    printing: None,
                }),
                format!("/card/sol-ring/{id}?open=send-to"),
            ),
        ] {
            // The router follows a query value with `&` when another may come after it.
            let printed = route.to_string();
            assert_eq!(printed.strip_suffix('&').unwrap_or(&printed), path);
            assert_eq!(Route::from_str(&path).ok(), Some(route), "{path}");
        }
    }

    #[test]
    fn an_unknown_open_value_opens_the_card_and_asks_for_nothing() {
        for path in ["/card/sol-ring?open=nope", "/card/sol-ring?open="] {
            assert_eq!(
                Route::from_str(path).ok(),
                Some(Route::CardNamed {
                    slug: "sol-ring".to_owned(),
                    open: None,
                    printing: None
                }),
                "{path}"
            );
        }
    }

    #[test]
    fn a_picked_printing_rides_the_link_and_reads_back() {
        let id = Uuid::new_v4();
        let route = route_of(Destination::Named {
            slug: slug("Sol Ring"),
            send: false,
            printing: Some(id),
        });
        let path = format!("/card/sol-ring?printing={id}");
        assert_eq!(route.to_string(), path);
        assert_eq!(Route::from_str(&path).ok(), Some(route));
        let both = format!("/card/sol-ring?open=send-to&printing={id}");
        assert_eq!(
            Route::from_str(&both).ok(),
            Some(Route::CardNamed {
                slug: "sol-ring".to_owned(),
                open: Some(Ask::SendTo),
                printing: Some(id),
            })
        );
    }

    #[test]
    fn a_printing_that_is_no_id_opens_the_card() {
        assert_eq!(
            Route::from_str("/card/sol-ring?printing=nope").ok(),
            Some(Route::CardNamed {
                slug: "sol-ring".to_owned(),
                open: None,
                printing: None,
            })
        );
    }

    #[test]
    fn send_to_signed_out_goes_to_login() {
        assert_eq!(route_of(Destination::Login), Route::Login {});
    }
}
