//! Oracle-tag example cards — a deck-free swipe browse.
//!
//! Reached by tapping a row in the oracle-tag dictionary: serves real cards that
//! carry the tag so you can see what it catches, most-iconic first (EDHREC rank).
//! Read-only — there is no deck to collect into, so the gesture grammar is
//! reduced to navigation: swipe **left** for the next card, **down** to go back
//! one. Right/up are not allowed by the `SwipeConfig`, so those gestures return
//! the card to center instead of committing. The ActionBar carries the same
//! eyeball [`RulesButton`] as the other swipe screens for full card details.
//!
//! Cards come from the plain (deck-agnostic) `search_cards` route with a single
//! `oracle_tags_contains_any` criterion; the same linear `CardStack` +
//! `SwipeStack` machinery as the add screen, minus all the deck side effects.

use crate::{
    inbound::{
        components::{
            auth::ensure_session::EnsureFresh,
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
            navigation::overlay_stack::use_overlay_back,
            screen_header::ScreenHeader,
        },
        screens::deck::card::components::{
            action_history::{BrowseAction, MAX_CARDS_IN_STACK},
            card_info::{CardDetailsDialog, CardInfoDisplay, CardSkeleton, RulesButton},
            card_stack::use_card_stack,
            printing_sheet::PrintingSheet,
        },
    },
    outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::{
    auth::models::session::Session,
    card::{
        Card,
        search_card::card_filter::{builder::CardQueryBuilder, card_sort_key::CardSortKey},
    },
};

/// One page of example cards per fetch (matches the backend default).
const PAGE_LIMIT: u32 = 25;
/// Prefetch the next page once the cursor is within this many cards of the end.
const LOAD_MORE_THRESHOLD: usize = 5;

/// Deck-free swipe browse of the cards carrying a single oracle tag. An in-place
/// overlay (host renders it when `open`, mounting fresh per tag); back-swipe or the
/// Back button closes it.
#[component]
pub fn OracleTagExamples(mut open: Signal<bool>, slug: String) -> Element {
    use_overlay_back(open);
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();

    // The tag we're serving, held in a signal so the fetch closures stay `Copy`.
    // Re-navigating to a different tag always remounts (the dictionary sits
    // between them), so seeding once on mount is enough.
    let tag_slug = use_signal(|| slug.clone());

    let mut stack = use_card_stack::<BrowseAction>();
    let mut current_offset = use_signal(|| 0_u32);
    let mut is_loading_more = use_signal(|| false);
    let mut is_loading_cards = use_signal(|| true);
    let mut pagination_exhausted = use_signal(|| false);
    // Gesture start point for the end-of-stack panel, which accepts a down-swipe
    // to step back into the pile (no card there, so no `SwipeStack`).
    let mut end_swipe_start: Signal<Option<(f64, f64)>> = use_signal(|| None);
    // Eyeball → card details dialog (same util as Add/Remove swipe screens).
    let show_details = use_signal(|| false);
    // Printings browse from the details dialog. View-only: this browse is
    // deck-free, so switching printings changes nothing.
    let mut printing_open = use_signal(|| false);

    // Only left (next) and down (back) commit; right/up return to center.
    let swipe_config = SwipeConfig::new(vec![Direction::Left, Direction::Down], 60.0, 1.5);

    let current_card = move || stack.current();

    // Fetch the next page. Reads its guards via `peek` so it never subscribes a
    // caller (the mount effect) to its own writes.
    let mut load_more = move || {
        if *is_loading_more.peek() || *pagination_exhausted.peek() {
            return;
        }
        // Cap the in-memory stack like the add screen. A fat tag would otherwise
        // append forever; stop paging and let the end panel show gracefully.
        if stack.peek_len() >= MAX_CARDS_IN_STACK {
            pagination_exhausted.set(true);
            is_loading_cards.set(false);
            return;
        }
        is_loading_more.set(true);

        let offset = *current_offset.peek();
        let mut builder = CardQueryBuilder::new();
        builder
            .set_oracle_tags_contains_any(vec![tag_slug.peek().clone()])
            .set_is_token(false)
            .set_sort(CardSortKey::EdhrecRank)
            .set_limit(PAGE_LIMIT)
            .set_offset(offset);

        let Ok(filter) = builder.build() else {
            is_loading_more.set(false);
            is_loading_cards.set(false);
            return;
        };

        spawn(async move {
            let session = match session.ensure_fresh(client).await {
                Ok(session) => session,
                Err(e) => {
                    // A true auth failure clears the session and the AuthGate
                    // redirects; a transient network/server error leaves it intact,
                    // so surface it instead of a misleading empty state.
                    toast.error(e.to_user_message(), ToastOptions::default());
                    is_loading_more.set(false);
                    is_loading_cards.set(false);
                    return;
                }
            };

            match client().search_cards(&filter, &session).await {
                Ok(new_cards) => {
                    if new_cards.is_empty() {
                        pagination_exhausted.set(true);
                    } else {
                        // Image-less cards render as a text identity frame
                        // (FlippableCardImage), so we keep them — no client
                        // filter means no barren pages either.
                        current_offset.set(offset + PAGE_LIMIT);
                        stack.append(new_cards);
                    }
                    is_loading_more.set(false);
                    is_loading_cards.set(false);
                }
                Err(e) => {
                    tracing::warn!("oracle-tag examples fetch failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                    is_loading_more.set(false);
                    is_loading_cards.set(false);
                }
            }
        });
    };

    // Mount load. Subscribes only to `tag_slug`, so it runs once (a new tag
    // remounts the screen and re-seeds it).
    use_effect(move || {
        let _ = tag_slug();
        load_more();
    });

    // Advance past the just-committed card. The cursor may land one past the end
    // (empty window) — that's the "no more cards" state.
    let mut advance_after_commit = move || {
        let total = stack.len();
        if !stack.advance() {
            return;
        }
        if stack.index() + 1 >= total.saturating_sub(LOAD_MORE_THRESHOLD) {
            load_more();
        }
    };

    // Step the cursor back one card (down swipe / undo). Primes the enter
    // animation from the left, the side cards exit on advance.
    let mut go_back = move || {
        let Some(action) = stack.pop_action() else {
            toast.info(
                "Already at the first card".to_string(),
                ToastOptions::default().duration(Duration::from_millis(1500)),
            );
            return;
        };
        if !stack.step_back(&action) {
            // Couldn't step back (already at the top) — restore the history.
            stack.record(action);
        }
    };

    rsx! {
        div { class: "screen examples-overlay",
            ScreenHeader { title: "Examples: {slug}" }

            div { class: "screen-content card-swipe content-enter",
                div { class: "form-container",
                    if !stack.is_empty() && stack.index() < stack.len() {
                        SwipeStack {
                            cards: stack.window(),
                            config: swipe_config,
                            entering: stack.entering(),
                            on_swipe_left: move |_card: Card| {
                                stack.record(BrowseAction::Next);
                                advance_after_commit();
                            },
                            // Right / up are inert in a read-only browse (also
                            // blocked by SwipeConfig, so they never commit).
                            on_swipe_right: move |_card: Card| {},
                            on_swipe_up: move |_card: Card| {},
                            on_swipe_down: move |_card: Card| {
                                go_back();
                            },
                        }

                        if let Some(card) = current_card() {
                            CardInfoDisplay { card: card.clone() }
                            CardDetailsDialog {
                                open: show_details,
                                card,
                                on_printings: move |_| printing_open.set(true),
                            }
                        }
                    } else if !stack.is_empty() {
                        // Cursor past the last card. Still keep it down-swipeable
                        // so "back" leads into the pile from the end panel.
                        if is_loading_more() {
                            CardSkeleton { is_loading: true }
                        } else {
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
                                        if p.y - sy >= 60.0 && (p.y - sy).abs() > (p.x - sx).abs() {
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
                                    if p.y - sy >= 60.0 && (p.y - sy).abs() > (p.x - sx).abs() {
                                        go_back();
                                    }
                                },
                                div { class: "dict-empty", "No more cards. Swipe down to go back." }
                            }
                        }
                    } else if is_loading_cards() {
                        CardSkeleton { is_loading: true }
                    } else {
                        div { class: "dict-empty", "No example cards for this tag yet." }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| open.set(false),
                    "Back"
                }
                if current_card().is_some() {
                    RulesButton { open: show_details }
                }
            }

            // View-only printings browse (deck-free, so it never mutates anything).
            if let Some(card) = current_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    on_save: move |_: Card| {},
                    read_only: true,
                }
            }
        }
    }
}
