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

/// A group of `n` copies of the shared card under `key`, each row its own card id.
fn group(key: &str, n: usize) -> (components::DeckSection, Vec<RowInfo>) {
    let rows: Vec<RowInfo> = (0..n)
        .map(|_| {
            let mut row = row();
            row.card.scryfall_data.id = Uuid::new_v4();
            row
        })
        .collect();
    let section = components::DeckSection {
        key: key.to_owned(),
        title: key.to_owned(),
        pips: None,
        cards: rows.iter().map(|r| r.card.clone()).collect(),
    };
    (section, rows)
}

#[test]
fn the_list_lays_its_groups_down_two_plain_columns_in_order() {
    fn app() -> Element {
        let details = StackDetails {
            reveal: use_signal(Reveal::default),
            details_open: use_signal(|| false),
        };
        rsx! {
            components::DeckCardsView {
                sections: vec![group("Creatures", 6), group("Lands", 2), group("Artifacts", 2)],
                view: zwipe_ui::deck_cards::stacks::CardsView::List,
                currency: PriceCurrency::default(),
                collapsed: use_signal(Default::default),
                details,
                selected: use_signal(|| None),
                face: use_signal(|| 0),
                faces: use_signal(HashMap::new),
                boards: HashMap::new(),
                group_buy: components::use_group_buy(),
                actions: None,
                host: components::DeckRowHost::default(),
            }
        }
    }
    let html = render(app);
    assert!(
        !html.contains("column-count"),
        "no CSS multi-column: {html}"
    );
    let columns: Vec<&str> = html.split(r#"class="deck-list-column""#).skip(1).collect();
    assert_eq!(columns.len(), 2, "two columns: {html}");
    let left = columns.first().unwrap();
    let right = columns.get(1).unwrap();
    assert!(
        left.contains("Creatures") && !left.contains("Lands"),
        "{left}"
    );
    assert!(
        right.contains("Lands") && right.contains("Artifacts"),
        "{right}"
    );
}

/// Two stacks, in tap mode with `touch`.
fn stacks(touch: bool) -> Element {
    let details = StackDetails {
        reveal: use_signal(Reveal::default),
        details_open: use_signal(|| false),
    };
    rsx! {
        components::DeckStacks {
            sections: vec![group("Creatures", 3), group("Lands", 2)],
            currency: PriceCurrency::default(),
            collapsed: use_signal(Default::default),
            details,
            faces: use_signal(HashMap::new),
            group_buy: components::use_group_buy(),
            touch,
        }
    }
}

#[test]
fn touch_stacks_are_in_tap_mode_and_pointer_stacks_are_not() {
    let touch = render(|| stacks(true));
    assert_eq!(
        touch.matches(r#"class="card-stack tap""#).count(),
        2,
        "{touch}"
    );
    let pointer = render(|| stacks(false));
    assert!(!pointer.contains("card-stack tap"), "{pointer}");
}

#[test]
fn a_second_tap_opens_the_drawer_and_a_tap_off_keeps_it() {
    thread_local! {
        static DETAILS: std::cell::Cell<Option<StackDetails>> = const { std::cell::Cell::new(None) };
    }
    fn app() -> Element {
        let details = StackDetails {
            reveal: use_signal(Reveal::default),
            details_open: use_signal(|| false),
        };
        DETAILS.with(|d| d.set(Some(details)));
        rsx! {}
    }
    let mut dom = VirtualDom::new(app);
    dom.rebuild_in_place();
    let details = DETAILS.with(std::cell::Cell::get).unwrap();
    let id = Uuid::new_v4();
    let state = |dom: &VirtualDom| {
        dom.in_runtime(|| {
            let reveal = *details.reveal.peek();
            (reveal.revealed(), *details.details_open.peek())
        })
    };
    dom.in_runtime(|| details.tap(id));
    assert_eq!(state(&dom), (Some(id), false), "the first tap reveals");
    dom.in_runtime(|| details.dismiss());
    assert_eq!(state(&dom), (None, false), "a tap off closes the reveal");
    dom.in_runtime(|| {
        details.tap(id);
        details.tap(id);
    });
    assert_eq!(
        state(&dom),
        (Some(id), true),
        "the second tap opens the drawer"
    );
    dom.in_runtime(|| details.dismiss());
    assert_eq!(
        state(&dom),
        (Some(id), true),
        "a tap off leaves the open card"
    );
}
