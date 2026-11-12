//! The deck page's cards, List and Stacks, offer their edit controls only with edit actions: a
//! shared deck passes none and shows only what reads.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use std::collections::HashMap;

use components::{DeckEditActions, DeckRow, DeckStackDrawer, StackDetails};
use dioxus::dioxus_core::NoOpMutations;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::Board;
use zwipe_ui::deck_cards::keys::{RowInfo, RowKind};
use zwipe_ui::deck_cards::stacks::{Gesture, Reveal};

/// A real card as the API serves it, shared with zwipe-ui's tests.
const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

/// The words of the edit controls: the quantity stepper's count sits between `-` and `+`, so the
/// star and the board moves stand for them all.
const EDIT_WORDS: [&str; 3] = ["Star", "To maybeboard", "To sideboard"];

fn card() -> Card {
    serde_json::from_str(CARD_JSON).unwrap()
}

/// A main board row of the shared card.
fn row() -> RowInfo {
    RowInfo {
        card: card(),
        qty: 2,
        kind: RowKind::Main { mvp: false },
    }
}

/// Edit actions that do nothing.
fn actions() -> DeckEditActions {
    DeckEditActions {
        change_quantity: Callback::new(|_| {}),
        move_to_board: Callback::new(|_| {}),
        toggle_mvp: Callback::new(|_| {}),
    }
}

/// Renders `app` once its effects have run.
fn render(app: fn() -> Element) -> String {
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    dioxus_ssr::render(&dom)
}

/// The shared card's row, opened, with `actions`.
fn open_row(actions: Option<DeckEditActions>) -> Element {
    let id = card().scryfall_data.id;
    let selected = use_signal(|| Some(id));
    let face = use_signal(|| 0);
    let faces = use_signal(HashMap::new);
    rsx! {
        DeckRow {
            row: row(),
            board: Board::Deck,
            currency: PriceCurrency::default(),
            selected,
            face,
            faces,
            actions,
        }
    }
}

#[test]
fn an_open_row_offers_its_edits_with_actions() {
    let html = render(|| open_row(Some(actions())));
    for word in EDIT_WORDS {
        assert!(html.contains(word), "{word}: {html}");
    }
}

#[test]
fn an_open_row_without_actions_offers_no_edit() {
    let html = render(|| open_row(None));
    for word in EDIT_WORDS {
        assert!(!html.contains(word), "{word}: {html}");
    }
    assert!(
        html.contains("Research Assistant"),
        "the row still shows its card: {html}"
    );
}

/// The shared card's stack drawer, open, with `actions`.
fn open_drawer(actions: Option<DeckEditActions>) -> Element {
    let id = card().scryfall_data.id;
    let reveal = use_signal(|| {
        let mut reveal = Reveal::default();
        reveal.apply(Gesture::Click(id));
        reveal
    });
    let details = StackDetails {
        reveal,
        details_open: use_signal(|| true),
    };
    let faces = use_signal(HashMap::new);
    let boards: HashMap<Uuid, Board> = HashMap::new();
    rsx! {
        DeckStackDrawer {
            rows: vec![row()],
            boards,
            details,
            faces,
            actions,
        }
    }
}

#[test]
fn the_stack_drawer_offers_its_card_s_edits_with_actions() {
    let html = render(|| open_drawer(Some(actions())));
    for word in EDIT_WORDS {
        assert!(html.contains(word), "{word}: {html}");
    }
}

#[test]
fn the_stack_drawer_without_actions_offers_no_edit() {
    let html = render(|| open_drawer(None));
    for word in EDIT_WORDS {
        assert!(!html.contains(word), "{word}: {html}");
    }
    assert!(
        html.contains("Research Assistant"),
        "the drawer still shows its card: {html}"
    );
}
