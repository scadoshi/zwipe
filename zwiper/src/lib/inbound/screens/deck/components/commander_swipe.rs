//! Full-screen "Swipe for Commander" screen.
//!
//! Rendered as a sibling overlay above the create/edit form and toggled by
//! `open` — it stays mounted while closed so its filter and swipe position
//! persist, letting you commit a commander and reopen right where you left off.
//! It's the add-card swipe screen with commander semantics: commander-eligible
//! cards for the chosen format in **EDHREC-popularity order** through the full
//! [`CardFilterSheet`]. Swipe **left** to skip, **right** to choose, **down** to
//! undo; up does nothing.

use crate::inbound::components::hint_dialog::{
    HintBullet, HintBullets, HintColored, HintDialog, HintLine, open_and_record_hint,
};
use crate::inbound::components::interactions::swipe::{
    STACK_DEPTH, SwipeStack, config::SwipeConfig, direction::Direction,
};
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::screens::deck::card::components::card_info::{CardInfoDisplay, CardSkeleton};
use crate::inbound::screens::deck::card::filter::card_filter_sheet::CardFilterSheet;
use crate::outbound::client::{ZwipeClient, card::search_cards::ClientSearchCards};
use dioxus::prelude::*;
use std::collections::HashSet;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::card::search_card::card_filter::builder::CardFilterBuilder;
use zwipe_core::domain::card::search_card::card_filter::order_by_option::OrderByOption;
use zwipe_core::domain::deck::format::Format;
use zwipe_core::domain::user::models::hints::HINT_SWIPE_SELECT;

/// Commanders fetched per page (matches the backend default).
const PAGE: u32 = 25;
/// Prefetch the next page when this many or fewer cards remain ahead of the top.
const LOAD_MORE_AT: usize = 5;

/// Applies the always-on commander constraints to a user-built filter: restrict
/// to commanders legal in `format`, default to EDHREC order, exclude tokens.
fn commander_filter(mut builder: CardFilterBuilder, format: Format, offset: u32) -> CardFilterBuilder {
    builder.set_is_commander_in_format(format);
    if builder.order_by().is_none() {
        builder.set_order_by(OrderByOption::EdhrecRank);
        builder.set_ascending(true);
    }
    builder.set_is_token(false);
    builder.set_limit(PAGE);
    builder.set_offset(offset);
    builder
}

/// In-place commander-discovery swipe. Toggled by `open`; `format` is the deck's
/// current format. `on_select` fires with the chosen commander; `on_close`
/// cancels. Reads `Session` + `ZwipeClient` from context.
#[component]
pub(crate) fn CommanderSwipe(
    open: Signal<bool>,
    format: Signal<Option<Format>>,
    on_select: EventHandler<Card>,
    on_close: EventHandler<()>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();

    // Local filter context (seeded for commanders) so the shared CardFilterSheet
    // operates here without clobbering the app-wide add-screen filter. Persists
    // across open/close because this component stays mounted.
    let mut filter_builder = use_signal(|| {
        let mut b = CardFilterBuilder::new();
        b.set_order_by(OrderByOption::EdhrecRank).set_ascending(true);
        b
    });
    use_context_provider(|| filter_builder);
    let mut filter_reset_counter = use_signal(|| 0u32);
    use_context_provider(|| filter_reset_counter);

    let mut cards = use_signal(Vec::<Card>::new);
    let mut current_index = use_signal(|| 0usize);
    let mut offset = use_signal(|| 0u32);
    let mut exhausted = use_signal(|| false);
    // Counter value that produced the cards currently in the stack — lets us tell
    // a reopen (keep state) from a real filter change (refetch).
    let mut last_searched = use_signal(|| u32::MAX);
    let mut is_loading_cards = use_signal(|| false);
    let mut is_loading_more = use_signal(|| false);
    let mut entering = use_signal(|| Option::<Direction>::None);
    let mut filters_overlay_open = use_signal(|| false);
    // Swipe-vocabulary hint: the "?" button reopens it; it also auto-opens once
    // per account the first time the screen is opened (gated, since this stays
    // mounted while closed — a plain mount-time one-time hint would misfire).
    let mut hint_open = use_signal(|| false);
    let mut hint_fired = use_signal(|| false);
    use_effect(move || {
        if open() && !*hint_fired.peek() {
            hint_fired.set(true);
            open_and_record_hint(HINT_SWIPE_SELECT, session, client, hint_open);
        }
    });

    // Fetch fresh results: on first open, and whenever the filter sheet changes
    // the filter. A reopen with an unchanged filter keeps the persisted stack.
    use_effect(move || {
        if !open() {
            return;
        }
        let counter = filter_reset_counter();
        let Some(fmt) = *format.peek() else { return };
        if counter == *last_searched.peek() && !cards.peek().is_empty() {
            return;
        }
        last_searched.set(counter);
        cards.set(Vec::new());
        current_index.set(0);
        offset.set(0);
        exhausted.set(false);
        let builder = commander_filter(filter_builder.peek().clone(), fmt, 0);
        let (Some(session), Ok(filter)) = (session.peek().clone(), builder.build()) else {
            return;
        };
        is_loading_cards.set(true);
        usage_buffer().record_search();
        spawn(async move {
            match client().search_cards(&filter, &session).await {
                Ok(found) => {
                    let page: Vec<Card> = found
                        .into_iter()
                        .filter(|c| c.scryfall_data.primary_image_url(ImageSize::Large).is_some())
                        .collect();
                    if page.is_empty() {
                        exhausted.set(true);
                    }
                    offset.set(PAGE);
                    cards.set(page);
                    is_loading_cards.set(false);
                }
                Err(e) => {
                    tracing::warn!("commander search failed: {e}");
                    is_loading_cards.set(false);
                }
            }
        });
    });

    // Append the next page (left-swipe prefetch).
    let mut load_more = move || {
        if *is_loading_more.peek() || *exhausted.peek() {
            return;
        }
        let Some(fmt) = *format.peek() else { return };
        is_loading_more.set(true);
        spawn(async move {
            let off = *offset.peek();
            let builder = commander_filter(filter_builder.peek().clone(), fmt, off);
            let (Some(session), Ok(filter)) = (session.peek().clone(), builder.build()) else {
                is_loading_more.set(false);
                return;
            };
            match client().search_cards(&filter, &session).await {
                Ok(found) => {
                    let seen: HashSet<Uuid> =
                        cards.peek().iter().map(|c| c.scryfall_data.id).collect();
                    let mut page: Vec<Card> = found
                        .into_iter()
                        .filter(|c| {
                            c.scryfall_data.primary_image_url(ImageSize::Large).is_some()
                                && !seen.contains(&c.scryfall_data.id)
                        })
                        .collect();
                    if page.is_empty() {
                        exhausted.set(true);
                    } else {
                        offset.set(off + PAGE);
                        cards.write().append(&mut page);
                    }
                    is_loading_more.set(false);
                }
                Err(e) => {
                    tracing::warn!("commander load-more failed: {e}");
                    is_loading_more.set(false);
                }
            }
        });
    };

    // Left = skip: advance, prefetch when running low.
    let mut advance = move || {
        let total = cards.peek().len();
        let next = current_index() + 1;
        current_index.set(next);
        if !exhausted() && total.saturating_sub(next) <= LOAD_MORE_AT {
            load_more();
        }
    };

    // Down = undo the last skip: bring the previous commander back in.
    let mut undo = move || {
        if current_index() > 0 {
            current_index.set(current_index() - 1);
            entering.set(Some(Direction::Left));
        }
    };

    // Reset every user-set filter back to the commander default (all colors,
    // EDHREC order) and re-search.
    let mut clear_filter = move || {
        filter_builder.write().clear();
        filter_builder
            .write()
            .set_order_by(OrderByOption::EdhrecRank)
            .set_ascending(true);
        filter_reset_counter.set(filter_reset_counter() + 1);
    };

    if !open() {
        return rsx! {};
    }

    let swipe_config = SwipeConfig::new(
        vec![Direction::Left, Direction::Right, Direction::Down],
        60.0,
        1.5,
    );

    let idx = current_index();
    let window: Vec<Card> = cards().into_iter().skip(idx).take(STACK_DEPTH).collect();
    let current_card = cards().get(idx).cloned();
    let has_cards = !window.is_empty();

    rsx! {
        div { class: "screen commander-swipe-screen",
            div { class: "page-header", style: "position: relative;",
                h2 { "Zwipe select" }
                button {
                    class: "util-btn",
                    style: "position: absolute; right: 1rem; top: 50%; transform: translateY(-50%); opacity: 0.55; padding: 0.2rem 0.6rem;",
                    onclick: move |_| hint_open.set(true),
                    "?"
                }
            }

            div { class: "screen-content card-swipe content-enter",
                if has_cards {
                    SwipeStack {
                        cards: window,
                        config: swipe_config,
                        entering,
                        on_swipe_left: move |_card: Card| {
                            usage_buffer().record_swipe(Direction::Left);
                            advance();
                        },
                        on_swipe_right: move |card: Card| on_select.call(card),
                        on_swipe_up: move |_card: Card| {},
                        on_swipe_down: move |_card: Card| {
                            usage_buffer().record_swipe(Direction::Down);
                            undo();
                        },
                    }
                    if let Some(card) = current_card {
                        CardInfoDisplay { card }
                    }
                } else if is_loading_cards() || is_loading_more() {
                    CardSkeleton { is_loading: true }
                } else {
                    CardSkeleton {}
                }
            }

            div { class: "util-bar",
                button {
                    class: "util-btn",
                    onclick: move |_| on_close.call(()),
                    "Cancel"
                }
                button {
                    class: "util-btn",
                    onclick: move |_| filters_overlay_open.set(true),
                    "Filter"
                }
                if !filter_builder.read().is_empty() {
                    button {
                        class: "util-btn",
                        onclick: move |_| clear_filter(),
                        "Clear"
                    }
                }
            }

            CardFilterSheet {
                open: filters_overlay_open,
                show_format_filter: false,
                show_active_indicators: true,
                on_clear: move |_| clear_filter(),
            }

            HintDialog {
                open: hint_open,
                title: "Zwipe select",
                HintBullets {
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-success", "right" }
                        " to choose that commander."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--color-error", "left" }
                        " to skip it."
                    }
                    HintBullet {
                        "Swipe "
                        HintColored { color: "--accent-tertiary", "down" }
                        " to undo your last skip."
                    }
                }
                HintLine { "Most-played commanders come first. Tap Filter to narrow by color or anything else." }
            }
        }
    }
}
