//! Raising or flipping one card in the deck's stacks re-renders only that card's stack, not every
//! stack on the page. Each `CardStack` counts its renders (`count_render!`, debug builds only).

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::cell::Cell;
use std::collections::HashMap;

use components::{StackDetails, render_count, use_stack_details};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::{Gesture, Reveal};

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// How many stacks the page holds, and how many cards each.
const STACKS: usize = 20;
const PER_STACK: usize = 5;

/// The stacks' reveal and faces, and the cards, reached from the test between renders.
#[derive(Clone, Copy)]
struct Probe {
    details: StackDetails,
    faces: Signal<HashMap<Uuid, usize>>,
}

thread_local! {
    static PROBE: Cell<Option<Probe>> = const { Cell::new(None) };
    static TOUCH: Cell<bool> = const { Cell::new(true) };
}

/// The card ids, stack by stack: stack `s`'s card `c` is `id(s, c)`.
fn id(stack: usize, card: usize) -> Uuid {
    let n = stack
        .saturating_mul(PER_STACK)
        .saturating_add(card)
        .saturating_add(1);
    Uuid::from_u128(u128::try_from(n).unwrap())
}

fn sections() -> Vec<(components::DeckSection, Vec<RowInfo>)> {
    let card: Card = serde_json::from_str(CARD_JSON).unwrap();
    (0..STACKS)
        .map(|s| {
            let rows: Vec<RowInfo> = (0..PER_STACK)
                .map(|c| {
                    let mut card = card.clone();
                    card.scryfall_data.id = id(s, c);
                    RowInfo {
                        card,
                        qty: 1,
                        kind: RowKind::Main { mvp: false },
                    }
                })
                .collect();
            let key = format!("Group {s}");
            let section = components::DeckSection {
                key: key.clone(),
                title: key,
                pips: None,
                cards: rows.iter().map(|r| r.card.clone()).collect(),
            };
            (section, rows)
        })
        .collect()
}

fn app() -> Element {
    let details = use_stack_details(use_signal(Reveal::default), use_signal(|| false));
    let faces = use_signal(HashMap::new);
    PROBE.with(|probe| probe.set(Some(Probe { details, faces })));
    let sections = use_hook(sections);
    rsx! {
        components::DeckStacks {
            sections,
            currency: PriceCurrency::default(),
            collapsed: use_signal(Default::default),
            details,
            faces,
            group_buy: components::use_group_buy(),
            touch: TOUCH.with(Cell::get),
        }
    }
}

/// Runs the page's effects and renders until it settles.
fn settle(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
}

/// How many stacks re-render when `change` runs on a settled page.
fn stacks_rendered_by(touch: bool, change: impl FnOnce(Probe)) -> usize {
    TOUCH.with(|t| t.set(touch));
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    settle(&mut dom);
    assert_eq!(
        dioxus_ssr::render(&dom).matches("card-stack-slot").count(),
        STACKS.saturating_mul(PER_STACK),
        "every card is on the page"
    );
    render_count::reset();
    let probe = PROBE.with(Cell::get).unwrap();
    dom.in_runtime(|| change(probe));
    settle(&mut dom);
    render_count::renders("CardStack")
}

#[test]
fn raising_one_card_re_renders_only_its_stack() {
    let raised = stacks_rendered_by(true, |probe| {
        let mut reveal = probe.details.reveal;
        reveal.write().apply(Gesture::Tap(id(7, 2)));
    });
    assert_eq!(raised, 1, "only the raised card's stack, of {STACKS}");
}

#[test]
fn hovering_one_card_re_renders_only_its_stack() {
    let hovered = stacks_rendered_by(false, |probe| {
        let mut reveal = probe.details.reveal;
        reveal.write().apply(Gesture::Hover(id(3, 1)));
    });
    assert_eq!(hovered, 1, "only the hovered card's stack, of {STACKS}");
}

#[test]
fn flipping_one_card_re_renders_only_its_stack() {
    let flipped = stacks_rendered_by(true, |probe| {
        let mut faces = probe.faces;
        faces.write().insert(id(11, 0), 1);
    });
    assert_eq!(flipped, 1, "only the flipped card's stack, of {STACKS}");
}
