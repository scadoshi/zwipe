//! Oracle-tag example cards: a deck-free swipe browse.
//!
//! Reached by tapping a row in the oracle-tag dictionary: serves real cards that
//! carry the tag so you can see what it catches, most-iconic first (EDHREC rank).
//! Read-only: there is no deck to collect into, so the gesture grammar is
//! reduced to navigation: swipe **left** for the next card, **down** to go back
//! one. Right/up are not allowed by the `SwipeConfig`, so those gestures return
//! the card to center instead of committing. The action dock carries the same
//! eye as the other swipe screens for full card details.
//!
//! Cards come from the plain (deck-agnostic) `search_cards` route with a single
//! `oracle_tags_contains_any` criterion. What the browse knows and does (the
//! cards, the cursor, paging) is `zwipe_ui::screens::oracle_tag_examples`; this
//! file draws it through the same `SwipeStack` as the add screen.

use components::{HintBullet, HintBullets, HintDialog, Toasts, use_overlay_back, use_toast};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::user::models::hints::HINT_OTAG_EXAMPLES;
use zwipe_ui::screens::oracle_tag_examples::{
    self as model, Action, Destination, Effect, State, Status,
};
use zwipe_ui::stack;

use crate::inbound::components::action_dock::EyeIcon;
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::interactions::swipe::SwipeStack;
use crate::inbound::components::interactions::swipe::config::SwipeConfig;
use crate::inbound::components::interactions::swipe::direction::Direction;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{OracleTagScreen, Screen, screen};
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::screens::deck::card::components::card_info::{
    CardDetailsSheet, CardInfoDisplay, CardSkeleton,
};
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;

/// Pixels a swipe travels to commit.
const SWIPE_DISTANCE_PX: f64 = 90.0;
/// Pixels per millisecond a flick needs to commit.
const SWIPE_SPEED_PX_PER_MS: f64 = 2.5;
/// Pixels a downward pull on the end card travels to go back.
const BACK_PULL_PX: f64 = 60.0;

/// The examples' model with the handles that perform its effects. Copy, so
/// every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    open: Signal<bool>,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    /// The top card's entrance, passed straight through as `SwipeStack`'s
    /// `entering` prop, which clears it once the card has come in.
    entering: Signal<Option<Direction>>,
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

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::SearchCards { ticket, query } => {
                let authed = self.authed;
                spawn(async move {
                    // The facade refreshes, reports, and toasts on failure (a
                    // true auth failure also clears the session and the
                    // AuthGate redirects).
                    let reply = authed
                        .run("load_examples", |c, s| async move {
                            c.search_cards(&query, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Some(cards) => Action::Loaded { ticket, cards },
                        None => Action::Failed { ticket },
                    });
                });
            }
            Effect::Enter(side) => {
                let mut entering = self.entering;
                entering.set(Some(match side {
                    stack::Direction::Left => Direction::Left,
                    stack::Direction::Right => Direction::Right,
                    stack::Direction::Up => Direction::Up,
                }));
            }
            Effect::Navigate(Destination::Previous) => {
                let mut open = self.open;
                open.set(false);
            }
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }
}

/// Deck-free swipe browse of the cards carrying a single oracle tag. An in-place
/// overlay (host renders it when `open`, mounting fresh per tag); back-swipe or the
/// Back button closes it.
#[component]
pub fn OracleTagExamples(open: Signal<bool>, slug: String) -> Element {
    use_overlay_back(open);
    let driver = Driver {
        model: use_signal(State::new),
        open,
        toast: use_toast(),
        authed: use_authed(Screen::OracleTag(OracleTagScreen::Examples)),
        session: use_context(),
        client: use_context(),
        // Swipe-navigation hint: auto-opens once per account, reopenable via the "?".
        hint_open: use_signal(|| false),
        entering: use_signal(|| None),
    };
    let examples_hint_open = driver.hint_open;

    // Offers the hint and asks for the first page as the browse first draws.
    // A different tag always remounts (the dictionary sits between them), so
    // loading once per mount is enough.
    use_hook(move || driver.dispatch(Action::Load { slug }));

    // Gesture start point for the end-of-stack panel, which accepts a down-swipe
    // to step back into the pile (no card there, so no `SwipeStack`).
    let mut end_swipe_start: Signal<Option<(f64, f64)>> = use_signal(|| None);
    // Eyeball → card details sheet (same util as Add/Remove swipe screens).
    let show_details = use_signal(|| false);
    // Printings browse from the details sheet. View-only: this browse is
    // deck-free, so switching printings changes nothing.
    let mut printing_open = use_signal(|| false);

    // Only left (next) and down (back) commit; right/up return to center.
    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Down],
        SWIPE_DISTANCE_PX,
        SWIPE_SPEED_PX_PER_MS,
    );

    let (status, window, current) = {
        let model = driver.model.read();
        (
            model.status(),
            model.window().to_vec(),
            model.current().cloned(),
        )
    };

    let go_back = move || driver.dispatch(Action::BackOne);

    let no_card = current.is_none();
    rsx! {
        div { class: "screen examples-overlay",
            // An overlay screen paints its own backdrop, since .screen is clear
            // and the screen beneath would show through.
            ::components::GridCells { still: true }
            ScreenHeader { title: model::TITLE, hint: examples_hint_open, back: move |()| driver.dispatch(Action::Back) }

            HintDialog {
                open: examples_hint_open,
                actions: guide_link::one_time(HINT_OTAG_EXAMPLES),
                title: model::HINT_TITLE,
                HintBullets {
                    HintBullet { "Swipe " HintColored { color: "--color-success", "left" } " for the next card" }
                    HintBullet { "Swipe " HintColored { color: "--accent-secondary", "down" } " to go back one" }
                }
            }

            div { class: "screen-content card-swipe content-enter",
                div { class: "form-container",
                    match status {
                        Status::Showing => rsx! {
                            SwipeStack {
                                cards: window,
                                config: swipe_config,
                                entering: driver.entering,
                                on_swipe_left: move |_card: Card| driver.dispatch(Action::Next),
                                // Right / up are inert in a read-only browse (also
                                // blocked by SwipeConfig, so they never commit).
                                on_swipe_right: move |_card: Card| {},
                                on_swipe_up: move |_card: Card| {},
                                on_swipe_down: move |_card: Card| {
                                    go_back();
                                },
                            }

                            if let Some(card) = current.clone() {
                                CardInfoDisplay { card }
                            }
                        },
                        Status::Loading => rsx! {
                            CardSkeleton {}
                        },
                        // Cursor past the last card. Still keep it down-swipeable
                        // so "back" leads into the pile from the end panel.
                        Status::End => rsx! {
                            div {
                                class: "flex-col flex-center",
                                style: "flex: 1; min-height: 0; width: 100%;",
                                ontouchstart: move |e: Event<TouchData>| {
                                    if let Some(t) = e.touches().into_iter().next() {
                                        let p = t.client_coordinates();
                                        end_swipe_start.set(Some((p.x, p.y)));
                                    }
                                },
                                ontouchend: move |e: Event<TouchData>| {
                                    let Some((sx, sy)) = end_swipe_start() else {
                                        return;
                                    };
                                    end_swipe_start.set(None);
                                    if let Some(t) = e.touches_changed().into_iter().next() {
                                        let p = t.client_coordinates();
                                        if p.y - sy >= BACK_PULL_PX && (p.y - sy).abs() > (p.x - sx).abs() {
                                            go_back();
                                        }
                                    }
                                },
                                onmousedown: move |e: Event<MouseData>| {
                                    let p = e.client_coordinates();
                                    end_swipe_start.set(Some((p.x, p.y)));
                                },
                                onmouseup: move |e: Event<MouseData>| {
                                    let Some((sx, sy)) = end_swipe_start() else {
                                        return;
                                    };
                                    end_swipe_start.set(None);
                                    let p = e.client_coordinates();
                                    if p.y - sy >= BACK_PULL_PX && (p.y - sy).abs() > (p.x - sx).abs() {
                                        go_back();
                                    }
                                },
                                div { class: "dict-empty", {model::END} }
                            }
                        },
                        Status::Empty => rsx! {
                            div { class: "dict-empty", {model::EMPTY} }
                        },
                    }
                }
            }

            // Card details as the action dock.
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": "Card details",
                    disabled: no_card,
                    onclick: move |_| {
                        let mut open = show_details;
                        open.toggle();
                    },
                    EyeIcon {}
                }
            }

            // Root-mounted so the sheets cover the footer, the printing sheet
            // after the details sheet so it draws over it. View-only printings
            // browse (deck-free, so it never mutates anything).
            if let Some(card) = current {
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printing_open.set(true),
                }
                PrintingSheet {
                    host_screen: screen::ORACLE_TAG_EXAMPLES,
                    card,
                    open: printing_open,
                    on_save: move |_: Card| {},
                    read_only: true,
                }
            }
        }
    }
}
