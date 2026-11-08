//! Oracle tag examples: real cards that carry one tag, most played first,
//! browsed one at a time. Not a route; it opens over the screen that asked.
//!
//! The app swipes left for the next card and down to go back one. Here the
//! card still drags, and ← and ↓ (or `Next` and `Back one`) do the same. On
//! a wide window it is a large modal with the card's details beside it;
//! below the breakpoint it is the app's full-screen overlay.

use components::{
    CardDetailsPanel, CardDetailsPanelSkeleton, PageHeader, TOAST_QUICK, use_overlay_back,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::card_sort_key::CardSortKey;
use zwipe_ui::stack::{CardStack, Direction, StackAction};
use zwipe_ui::telemetry::vocabulary::{OracleTagScreen, Screen};
use zwipe_ui::timing::{EXAMPLES_MAX_CARDS, LOAD_MORE_AT, PAGE_SIZE};

use crate::inbound::cards::card_info::{CardDetailsSheet, CardSkeleton};
use crate::inbound::cards::otag::use_describe_tag;
use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::cards::swipe_stack::{
    Swipe, SwipeDock, SwipeStack, use_swipe_control, use_swipe_keys_or_undo,
};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::back_arrow::BackArrow;
use crate::inbound::chrome::details_eye::{DockEye, Surface, use_details_eye};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::telemetry;

/// A step forward through the examples. It leaves to the left, so a step
/// back brings the card in from there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Next;

impl StackAction for Next {
    fn exited(&self) -> Direction {
        Direction::Left
    }
}

/// The examples for `slug`, over the screen while `open`. Mount it fresh
/// per tag.
#[component]
pub fn OracleTagExamples(mut open: Signal<bool>, slug: String) -> Element {
    telemetry::use_telemetry_screen(Screen::OracleTag(OracleTagScreen::Examples));
    use_overlay_back(open);
    let toast = use_toast();
    let authed = use_authed();
    let describe_tag = use_describe_tag();
    let hint_open = use_one_time_hint(OneTimeHint::OtagExamples);
    let mut hint_again = hint_open;
    let tag = use_signal(|| slug.clone());

    let mut stack: Signal<CardStack<Next>> = use_signal(CardStack::default);
    let mut offset = use_signal(|| 0_u32);
    let mut loading_more = use_signal(|| false);
    let mut loading = use_signal(|| true);
    let mut exhausted = use_signal(|| false);
    let show_details = use_signal(|| false);
    let eye = use_details_eye(show_details, Surface::LargeModal);
    let mut printing_open = use_signal(|| false);
    let control = use_swipe_control();

    let mut load_more = move || {
        if *loading_more.peek() || *exhausted.peek() {
            return;
        }
        if stack.peek().len() >= EXAMPLES_MAX_CARDS {
            exhausted.set(true);
            loading.set(false);
            return;
        }
        loading_more.set(true);
        let at = *offset.peek();
        let mut builder = CardQueryBuilder::new();
        builder
            .set_oracle_tags_contains_any(vec![tag.peek().clone()])
            .set_is_token(false)
            .set_sort(CardSortKey::EdhrecRank)
            .set_limit(PAGE_SIZE)
            .set_offset(at);
        let Ok(filter) = builder.build() else {
            loading_more.set(false);
            loading.set(false);
            return;
        };
        spawn(async move {
            if let Some(cards) = authed
                .run("load_examples", |c, s| async move {
                    c.search_cards(&filter, &s).await
                })
                .await
            {
                if cards.is_empty() {
                    exhausted.set(true);
                } else {
                    match at.checked_add(PAGE_SIZE) {
                        Some(next) => offset.set(next),
                        None => exhausted.set(true),
                    }
                    stack.write().append(cards);
                }
            }
            loading_more.set(false);
            loading.set(false);
        });
    };

    use_hook(load_more);

    let go_back = use_callback(move |()| {
        let mut s = stack.write();
        let Some(action) = s.pop_action() else {
            drop(s);
            toast.info(
                "Already at the first card".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
            return;
        };
        if !s.step_back(&action) {
            s.record(action);
        }
    });
    use_swipe_keys_or_undo(
        control,
        vec![Swipe::Left, Swipe::Down],
        Callback::new(move |()| stack.peek().current().is_some()),
        go_back,
    );

    let on_swipe = move |(swipe, _card): (Swipe, Card)| match swipe {
        Swipe::Left => {
            let near_end = {
                let mut s = stack.write();
                s.record(Next);
                let total = s.len();
                s.advance()
                    && s.index()
                        .checked_add(1)
                        .is_some_and(|seen| seen >= total.saturating_sub(LOAD_MORE_AT))
            };
            if near_end {
                load_more();
            }
        }
        Swipe::Down => go_back.call(()),
        Swipe::Right | Swipe::Up => {}
    };

    let (current, window, entering, past_end, empty) = {
        let s = stack.read();
        (
            s.current().cloned(),
            s.window().to_vec(),
            s.entering(),
            !s.is_empty() && s.index() >= s.len(),
            s.is_empty(),
        )
    };

    let has_card = current.is_some();

    rsx! {
        div { class: "screen examples-overlay web-examples web-large-modal",
            components::GridCells { still: true }
            PageHeader {
                title: "Examples",
                leading: rsx! {
                    BackArrow { class: "page-header-support", onclick: move |()| open.set(false) }
                },
                on_hint: move |()| hint_again.set(true),
            }
            div { class: "screen-content card-swipe content-enter web-examples-body {eye.body_class()}",
                div { class: "form-container web-examples-card",
                    if current.is_some() {
                        SwipeStack {
                            cards: window,
                            allowed: vec![Swipe::Left, Swipe::Down],
                            control,
                            entering,
                            on_entered: move |()| stack.write().cancel_entering(),
                            on_swipe,
                        }
                    } else if past_end {
                        if loading_more() {
                            CardSkeleton {}
                        } else {
                            div { class: "flex-col flex-center", style: "flex: 1; min-height: 0; width: 100%;",
                                div { class: "dict-empty",
                                    span { class: "chip-note", "No cards left. Swipe down to go back." }
                                }
                            }
                        }
                    } else if loading() || (empty && loading_more()) {
                        CardSkeleton {}
                    } else {
                        div { class: "dict-empty",
                            span { class: "chip-note", "No example cards for this tag yet." }
                        }
                    }
                }
                if let Some(card) = current.clone() {
                    div { class: "web-examples-details",
                        CardDetailsPanel {
                            key: "{card.scryfall_data.id}",
                            card,
                            describe_tag: Some(describe_tag),
                            on_printings: move |()| printing_open.set(true),
                        }
                    }
                } else if loading() || (empty && loading_more()) {
                    div { class: "web-examples-details",
                        CardDetailsPanelSkeleton {}
                    }
                }
            }
            // Card details as the action dock; Back is the header's arrow.
            SwipeDock {
                control,
                buttons: vec![(Swipe::Left, "Next"), (Swipe::Down, "Back one")],
                card_on_top: has_card,
                on_undo: move |()| go_back.call(()),
                div { class: "action-dock action-dock-right",
                    DockEye { eye, disabled: !has_card }
                }
            }
            OneTimeHintDialog { hint: OneTimeHint::OtagExamples, open: hint_open }
            if let Some(card) = current {
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printing_open.set(true),
                }
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
