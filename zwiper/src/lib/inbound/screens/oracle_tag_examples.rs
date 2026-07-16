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
            catalog_cache::CatalogCache,
            interactions::swipe::{SwipeStack, config::SwipeConfig, direction::Direction},
            screen_header::ScreenHeader,
        },
        screens::deck::card::components::{
            action_history::{BrowseAction, MAX_CARDS_IN_STACK},
            card_info::{CardDetailsDialog, CardInfoDisplay, CardSkeleton, RulesButton},
            card_stack::use_card_stack,
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
        scryfall_data::ImageSize,
        search_card::card_filter::{builder::CardQueryBuilder, card_sort_key::CardSortKey},
    },
};

/// One page of example cards per fetch (matches the backend default).
const PAGE_LIMIT: u32 = 25;
/// Prefetch the next page once the cursor is within this many cards of the end.
const LOAD_MORE_THRESHOLD: usize = 5;

/// Deck-free swipe browse of the cards carrying a single oracle tag.
#[component]
pub fn OracleTagExamples(slug: String) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();

    // The tag we're serving, held in a signal so the fetch closures stay `Copy`.
    // Re-navigating to a different tag always remounts (the dictionary sits
    // between them), so seeding once on mount is enough.
    let tag_slug = use_signal(|| slug.clone());

    // Warm the catalog for the human label (usually already loaded from the
    // dictionary we came from); falls back to the raw slug until it lands.
    use_effect(move || {
        cache.ensure_oracle_tags(client);
    });

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
                        // Advance the page cursor even if this page is all
                        // image-less — the near-end trigger will fetch the next.
                        current_offset.set(offset + PAGE_LIMIT);
                        let with_images: Vec<Card> = new_cards
                            .into_iter()
                            .filter(|c| {
                                c.scryfall_data
                                    .primary_image_url(ImageSize::Large)
                                    .is_some()
                            })
                            .collect();
                        if !with_images.is_empty() {
                            stack.append(with_images);
                        }
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

    // Human label for the tag from the shared catalog, else the raw slug.
    let label = {
        let cell = cache.oracle_tags.cell();
        let read = cell.read();
        read.loaded()
            .and_then(|tags| {
                tags.iter()
                    .find(|t| t.slug == slug)
                    .map(|t| t.label.clone())
            })
            .unwrap_or_else(|| slug.clone())
    };

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Examples: {label}" }

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
                            CardDetailsDialog { open: show_details, card }
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
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
                if current_card().is_some() {
                    RulesButton { open: show_details }
                }
            }
        }
    }
}
