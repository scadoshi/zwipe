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

#[test]
fn a_stack_header_s_cart_sits_outside_its_fold_toggle() {
    let html = render(|| stacks(true));
    let headers: Vec<&str> = html.split(r#"class="card-group-toggle""#).skip(1).collect();
    assert_eq!(headers.len(), 2, "each stack header has a toggle: {html}");
    for header in headers {
        let (toggle, rest) = header.split_once("</div>").unwrap();
        assert!(
            toggle.contains("card-group-label"),
            "the title folds: {toggle}"
        );
        assert!(
            !toggle.contains("cart-button"),
            "the cart never folds: {toggle}"
        );
        assert!(
            rest.contains("cart-button"),
            "the cart follows the toggle: {rest}"
        );
    }
}

/// The stacks' reveal, from the test, between renders.
#[derive(Clone, Copy)]
struct BackProbe {
    stack: components::OverlayBackStack,
    details: StackDetails,
}

thread_local! {
    static BACK: std::cell::Cell<Option<BackProbe>> = const { std::cell::Cell::new(None) };
    static TOUCH: std::cell::Cell<bool> = const { std::cell::Cell::new(true) };
}

fn back_app() -> Element {
    let stack = components::use_overlay_back_stack();
    use_context_provider(|| stack);
    let details = StackDetails {
        reveal: use_signal(Reveal::default),
        details_open: use_signal(|| false),
    };
    BACK.with(|b| b.set(Some(BackProbe { stack, details })));
    rsx! {
        components::DeckStacks {
            sections: vec![group("Creatures", 3)],
            currency: PriceCurrency::default(),
            collapsed: use_signal(Default::default),
            details,
            faces: use_signal(HashMap::new),
            group_buy: components::use_group_buy(),
            touch: TOUCH.with(std::cell::Cell::get),
        }
    }
}

fn settle(dom: &mut VirtualDom) {
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
}

/// The back stack after `taps` on one card, in touch mode or not.
fn back_after(touch: bool, taps: usize) -> (VirtualDom, BackProbe) {
    TOUCH.with(|t| t.set(touch));
    let mut dom = VirtualDom::new(back_app);
    dom.rebuild_in_place();
    settle(&mut dom);
    let probe = BACK.with(std::cell::Cell::get).unwrap();
    let id = Uuid::new_v4();
    for _ in 0..taps {
        dom.in_runtime(|| probe.details.tap(id));
    }
    settle(&mut dom);
    (dom, probe)
}

#[test]
fn back_clears_a_revealed_card_before_leaving() {
    let (mut dom, probe) = back_after(true, 1);
    let BackProbe { mut stack, details } = probe;
    assert!(
        dom.in_runtime(|| stack.close_top()),
        "back takes the reveal"
    );
    settle(&mut dom);
    assert_eq!(dom.in_runtime(|| details.reveal.peek().revealed()), None);
    assert!(
        !dom.in_runtime(|| stack.close_top()),
        "the next back leaves"
    );
}

#[test]
fn back_passes_by_stacks_with_nothing_revealed_and_an_open_card_keeps_the_reveal_s_step() {
    let (dom, probe) = back_after(true, 0);
    let BackProbe { mut stack, .. } = probe;
    assert!(!dom.in_runtime(|| stack.close_top()), "nothing revealed");
    // The open card's drawer registers its own step over this one (its host mounts it), so
    // Back closes the drawer first and the reveal after.
    let (dom, probe) = back_after(true, 2);
    let BackProbe { stack, .. } = probe;
    assert_eq!(
        dom.in_runtime(|| stack.depth()),
        1,
        "the reveal keeps its step under the open card's drawer"
    );
    let (dom, probe) = back_after(false, 1);
    let BackProbe { mut stack, .. } = probe;
    assert!(
        !dom.in_runtime(|| stack.close_top()),
        "a pointer's reveal is the hover's"
    );
}

/// The group keys in each touch column, left then right, for `sections` with `folded` folded.
fn columns_of(html: &str) -> Vec<Vec<String>> {
    html.split(r#"class="deck-stacks-column""#)
        .skip(1)
        .map(|column| {
            column
                .split(r#"data-fold=""#)
                .skip(1)
                .filter_map(|rest| rest.split_once('"').map(|(key, _)| key.to_owned()))
                .collect()
        })
        .collect()
}

thread_local! {
    static FOLDED: std::cell::RefCell<Vec<String>> = const { std::cell::RefCell::new(Vec::new()) };
}

fn masonry_app() -> Element {
    let details = StackDetails {
        reveal: use_signal(Reveal::default),
        details_open: use_signal(|| false),
    };
    let folded: std::collections::HashSet<String> =
        FOLDED.with(|f| f.borrow().iter().cloned().collect());
    rsx! {
        components::DeckStacks {
            sections: vec![group("Creatures", 30), group("Lands", 1), group("Artifacts", 1), group("Enchantments", 1)],
            currency: PriceCurrency::default(),
            collapsed: use_signal(|| folded),
            details,
            faces: use_signal(HashMap::new),
            group_buy: components::use_group_buy(),
            touch: true,
        }
    }
}

#[test]
fn touch_stacks_lie_in_two_columns_of_their_own_heights() {
    FOLDED.with(|f| f.borrow_mut().clear());
    let html = render(masonry_app);
    assert_eq!(
        columns_of(&html),
        [
            vec!["Creatures"],
            vec!["Lands", "Artifacts", "Enchantments"]
        ],
        "the short stacks fill the column beside the long one: {html}"
    );
    FOLDED.with(|f| *f.borrow_mut() = vec!["Creatures".to_owned()]);
    let html = render(masonry_app);
    assert_eq!(
        columns_of(&html),
        [
            vec!["Creatures", "Artifacts"],
            vec!["Lands", "Enchantments"]
        ],
        "a folded stack counts as its header: {html}"
    );
}

#[test]
fn pointer_stacks_keep_their_one_grid() {
    let html = render(|| stacks(false));
    assert!(!html.contains("deck-stacks-column"), "{html}");
    assert!(html.contains(r#"class="deck-stacks""#), "{html}");
}
