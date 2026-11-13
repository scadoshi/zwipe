//! Oracle tag examples: real cards that carry one tag, most played first, browsed one at a time.
//! Not a route; it opens over the screen that asked.
//!
//! The app swipes left for the next card and down to go back one. Here the card still drags, and
//! ← and ↓ (or `Next` and `Back one`) do the same. On a wide window it is a large modal with the
//! card's details beside it; below the breakpoint it is the app's full-screen overlay.
//!
//! Components never call the API. The host provides an [`ExamplesSource`], which fetches a page
//! of a tag's examples and replies with it, and a [`PrintingsSource`](crate::PrintingsSource)
//! for the Printings sheet.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::user::models::hints::OneTimeHint;
use zwipe_ui::hint::one_time;
use zwipe_ui::shortcuts::{self, CardKey};
use zwipe_ui::stack::{CardStack, Direction, StackAction};
use zwipe_ui::timing::{EXAMPLES_MAX_CARDS, LOAD_MORE_AT, PAGE_SIZE};

use crate::back_arrow::BackArrow;
use crate::card_skeleton::CardSkeleton;
use crate::details_eye::{DockEye, Surface, use_details_eye};
use crate::hint::{ContentHintDialog, guide_for_one_time};
use crate::keys::{ShortcutsList, use_offer_shortcuts, use_page_keys};
use crate::swipe_stack::{
    self, Swipe, SwipeDock, SwipeStack, use_swipe_control, use_swipe_keys_or_undo,
};
use crate::{
    CardDetailsPanel, CardDetailsPanelSkeleton, CardDetailsSheet, PageHeader, PrintingSheet,
    use_overlay_back,
};

/// One page of a tag's examples the screen asks its host for: the tag's slug, how many examples
/// come before the page, and where the host replies with the page's cards, or `None` when the
/// fetch failed.
#[derive(Clone, PartialEq)]
#[expect(
    missing_debug_implementations,
    reason = "a callback, which has no Debug"
)]
pub struct ExamplesPage {
    /// The tag.
    pub slug: String,
    /// How many examples come before this page.
    pub offset: u32,
    /// Takes the page's cards, most played first and no tokens, at most
    /// [`PAGE_SIZE`](zwipe_ui::timing::PAGE_SIZE) of them; `None` when the fetch failed.
    pub reply: Callback<Option<Vec<Card>>>,
}

/// The host's examples: fetches the page it is handed and replies with it.
#[derive(Clone, Copy, PartialEq)]
#[expect(
    missing_debug_implementations,
    reason = "a callback, which has no Debug"
)]
pub struct ExamplesSource {
    /// Fetches one page.
    pub fetch: Callback<ExamplesPage>,
}

/// A step forward through the examples. It leaves to the left, so a step back brings the card in
/// from there.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct Next;

impl StackAction for Next {
    fn exited(&self) -> Direction {
        Direction::Left
    }
}

/// The examples for `slug`, over the screen while `open`, from the [`ExamplesSource`] in context.
/// Mount it fresh per tag. `hint_open` opens its hint, which the header's `?` opens again;
/// `describe_tag` names the tags in the card's details; `on_first` runs when Back one is pressed
/// on the first card.
#[component]
pub fn OracleTagExamples(
    mut open: Signal<bool>,
    slug: String,
    hint_open: Signal<bool>,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_first: Option<Callback<()>>,
) -> Element {
    let layer = use_overlay_back(open);
    let source = try_use_context::<ExamplesSource>();
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

    let reply = use_callback(move |(at, fetched): (u32, Option<Vec<Card>>)| {
        if let Some(cards) = fetched {
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

    let mut load_more = move || {
        if *loading_more.peek() || *exhausted.peek() {
            return;
        }
        if stack.peek().len() >= EXAMPLES_MAX_CARDS {
            exhausted.set(true);
            loading.set(false);
            return;
        }
        let Some(source) = source else {
            loading.set(false);
            return;
        };
        loading_more.set(true);
        let at = *offset.peek();
        source.fetch.call(ExamplesPage {
            slug: tag.peek().clone(),
            offset: at,
            reply: Callback::new(move |fetched: Option<Vec<Card>>| reply.call((at, fetched))),
        });
    };

    use_hook(load_more);

    let go_back = use_callback(move |()| {
        let mut s = stack.write();
        let Some(action) = s.pop_action() else {
            drop(s);
            if let Some(on_first) = on_first {
                on_first.call(());
            }
            return;
        };
        if !s.step_back(&action) {
            s.record(action);
        }
    });
    // Shift+H lists the modal's keys while it is open.
    use_offer_shortcuts(ShortcutsList {
        groups: shortcuts::examples_groups(),
        claims_steps: true,
        note: None,
    });
    // `v` flips the top card or shows its meld and `d` opens or closes its details, as on every
    // swipe screen.
    use_page_keys(Callback::new(
        move |key: String| match shortcuts::card_key(&key) {
            Some(CardKey::Flip) => swipe_stack::flip_top_card(),
            Some(CardKey::Details) if stack.peek().current().is_some() => eye.press(),
            _ => {}
        },
    ));
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
        div {
            class: "screen examples-overlay web-examples web-large-modal",
            style: layer().map(|z| format!("--overlay-z: {z};")),
            crate::GridCells { still: true }
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
                            describe_tag,
                            on_printings: move |()| printing_open.set(true),
                        }
                    }
                } else if loading() || (empty && loading_more()) {
                    div { class: "web-examples-details",
                        CardDetailsPanelSkeleton {}
                    }
                }
            }
            // The swipes, a rule and the details eye as one dock; Back is the header's arrow.
            SwipeDock {
                control,
                buttons: vec![(Swipe::Left, "Next"), (Swipe::Down, "Back one")],
                card_on_top: has_card,
                on_undo: move |()| go_back.call(()),
                tail: rsx! {
                    DockEye { eye, disabled: !has_card }
                },
            }
            ExamplesHint { open: hint_open }
            if let Some(card) = current {
                CardDetailsSheet {
                    open: show_details,
                    card: card.clone(),
                    on_printings: move |()| printing_open.set(true),
                    describe_tag,
                }
                PrintingSheet {
                    card,
                    open: printing_open,
                }
            }
        }
    }
}

/// The examples' hint.
#[component]
fn ExamplesHint(open: Signal<bool>) -> Element {
    let Some(content) = one_time::content_for(OneTimeHint::OtagExamples) else {
        return rsx! {};
    };
    let actions = guide_for_one_time(OneTimeHint::OtagExamples);
    rsx! {
        ContentHintDialog { open, content, actions }
    }
}
