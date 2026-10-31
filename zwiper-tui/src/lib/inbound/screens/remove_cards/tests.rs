use crossterm::event::KeyCode;
use zwipe_core::domain::deck::Quantity;

use super::*;
use crate::domain::stack::Direction;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::widgets::card_motion::{CardMotion, Phase};
use crate::test_support::{card, deck, deck_profile, render, screen_text, word_colors};

/// Remove over a deck of A and B on the main board and M on the maybeboard, loaded.
fn loaded() -> (Harness, Uuid, Vec<Card>) {
    let cards = vec![card("A"), card("B"), card("M")];
    let mut d = deck(deck_profile("Trim"), cards.clone());
    d.entries[0].deck_card.quantity = Quantity::new(3).unwrap();
    d.entries[2].deck_card.board = Board::Maybeboard;
    let deck_id = d.deck_profile.id;
    let mut harness = Harness::on(Screen::Home(Box::default()));
    harness.open(|cx| open(cx, deck_id));
    harness.apply(Update::Loaded {
        deck_id,
        deck: Box::new(d),
    });
    harness.clear();
    (harness, deck_id, cards)
}

fn view(harness: &Harness) -> &RemoveView {
    match harness.screen() {
        Screen::RemoveCards(view) => view,
        other => panic!("not on Remove: {other:?}"),
    }
}

fn last_toast(harness: &Harness) -> String {
    harness
        .app
        .shared
        .toasts
        .items()
        .last()
        .unwrap()
        .message
        .clone()
}

fn requests(harness: &Harness) -> Vec<&Request> {
    harness
        .pending()
        .iter()
        .filter_map(|r| match r {
            screens::Request::RemoveCards(r) => Some(r),
            _ => None,
        })
        .collect()
}

fn current(harness: &Harness) -> String {
    view(harness).current().unwrap().scryfall_data.name.clone()
}

#[test]
fn opening_loads_the_deck_and_shows_the_main_board() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let deck_id = Uuid::new_v4();
    harness.open(|cx| open(cx, deck_id));
    assert_eq!(requests(&harness), [&Request::Load { deck_id }]);
    assert!(screen_text(&harness.app).contains("No cards"));
    let (harness, _, _) = loaded();
    assert_eq!(view(&harness).stack.len(), 2);
    let screen = screen_text(&harness.app);
    for expected in ["Remove deck cards", "Boards", "Main", "All", "A", "skip"] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn skip_keeps_the_card_cycles_and_its_undo_steps_back_with_the_apps_toasts() {
    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('h'));
    assert_eq!(last_toast(&harness), "Kept");
    assert_eq!(current(&harness), "B");
    harness.press(KeyCode::Char('h'));
    assert_eq!(current(&harness), "A", "it wraps");
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Undid keep");
    assert_eq!(current(&harness), "B");
}

#[test]
fn remove_deletes_every_copy_and_undo_recreates_them() {
    let (mut harness, deck_id, cards) = loaded();
    harness.press(KeyCode::Char('l'));
    assert_eq!(last_toast(&harness), "Removed from deck");
    assert_eq!(current(&harness), "B");
    let [Request::Remove { entry, .. }] = requests(&harness)[..] else {
        panic!("one remove");
    };
    let entry = entry.clone();
    harness.apply(Update::Removed { deck_id, entry });
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);
    harness.clear();
    harness.press(KeyCode::Char('u'));
    let [
        Request::UndoRemove {
            quantity, taken, ..
        },
    ] = requests(&harness)[..]
    else {
        panic!("one undo");
    };
    assert_eq!(*quantity, 3);
    assert!(taken.is_some());
    assert_eq!(current(&harness), "A", "the card is back on top");
    let deck_card = DeckCard {
        deck_id,
        scryfall_data_id: cards[0].scryfall_data.id,
        oracle_id: cards[0].scryfall_data.oracle_id.unwrap(),
        quantity: Quantity::new(3).unwrap(),
        board: Board::Deck,
        mvp_at: None,
    };
    harness.apply(Update::UndidRemove {
        deck_id,
        card: Box::new(cards[0].clone()),
        taken: None,
        result: Ok(Box::new(deck_card)),
    });
    assert_eq!(last_toast(&harness), "Undid remove");
}

#[test]
fn an_undo_the_deck_view_already_did_says_already_changed() {
    let (mut harness, deck_id, cards) = loaded();
    harness.press(KeyCode::Char('m'));
    assert_eq!(last_toast(&harness), "Moved to maybeboard");
    if let Screen::RemoveCards(view) = harness.screen_mut() {
        let id = cards[0].scryfall_data.id;
        for entry in &mut view.entries {
            if entry.deck_card.scryfall_data_id == id {
                entry.deck_card.board = Board::Deck;
            }
        }
    }
    harness.clear();
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Already changed");
    assert_eq!(requests(&harness).len(), 0);
    assert!(harness.app.shared.cards.undo.is_empty(deck_id));
}

#[test]
fn a_failed_undo_gives_the_entry_back() {
    let (mut harness, deck_id, cards) = loaded();
    let taken = UndoAction::MovedBoard {
        card_id: cards[0].scryfall_data.id,
        card_name: "A".to_owned(),
        from: Board::Deck,
    };
    harness.apply(Update::UndidMove {
        deck_id,
        card: Box::new(cards[0].clone()),
        from: Board::Deck,
        taken: Some(Box::new(taken)),
        result: Err(UndoFailure::Call("offline".to_owned())),
    });
    assert_eq!(last_toast(&harness), "Failed to undo: offline");
    assert_eq!(harness.app.shared.cards.undo.log(deck_id).len(), 1);
}

#[test]
fn boards_filter_refresh_and_details() {
    let (mut harness, deck_id, _) = loaded();
    harness.press(KeyCode::Char('2'));
    assert_eq!(current(&harness), "M");
    harness.press(KeyCode::Char('m'));
    assert_eq!(last_toast(&harness), "Moved to main");
    harness.press(KeyCode::Char('4'));
    assert_eq!(view(&harness).stack.len(), 3);
    harness.press(KeyCode::Char('h'));
    harness.press(KeyCode::Char('r'));
    assert_eq!(last_toast(&harness), "Stack refreshed");
    assert_eq!(view(&harness).stack.index(), 0);
    harness.press(KeyCode::Char('f'));
    assert!(view(&harness).sheet.is_some());
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('e'));
    assert!(view(&harness).details.is_some());
    harness.press(KeyCode::Char('p'));
    assert!(
        view(&harness)
            .printing
            .as_ref()
            .is_some_and(|p| p.read_only)
    );
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert!(
        harness
            .app
            .shared
            .cards
            .filters
            .restore(FilterScope::Remove, deck_id)
            .is_some(),
        "leaving parks the filter"
    );
}

#[test]
fn nothing_to_undo_and_a_tiny_terminal() {
    let (mut harness, _, _) = loaded();
    harness.press(KeyCode::Char('u'));
    assert_eq!(last_toast(&harness), "Nothing to undo");
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Char('e'));
    render(&harness.app, 12, 5);
}

#[test]
fn each_swipe_flies_its_card_off_and_undo_brings_it_back() {
    let (mut harness, _, _) = loaded();
    assert!(!harness.screen().animating());
    harness.press(KeyCode::Char('l'));
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.direction(), Direction::Right);
    assert!(matches!(motion.phase(), Phase::Exit(s) if s.card.scryfall_data.name == "A"));
    assert!(harness.screen().animating());
    assert_eq!(current(&harness), "B", "the next card is up at once");
    let rendered = screen_text(&harness.app);
    assert!(rendered.contains('B'), "B draws under the flying card");

    harness.press(KeyCode::Char('h'));
    assert_eq!(
        view(&harness).motion.as_ref().map(CardMotion::direction),
        Some(Direction::Left),
        "a new swipe replaces the exit"
    );
    harness.press(KeyCode::Char('u'));
    let motion = view(&harness).motion.clone().unwrap();
    assert_eq!(motion.phase(), &Phase::Enter);
    assert_eq!(motion.direction(), Direction::Left);
    render(&harness.app, 100, 40);
    render(&harness.app, 12, 5);
}

#[test]
fn the_swipe_keys_use_the_apps_verbs_in_its_hint_colors() {
    let (harness, _, _) = loaded();
    let palette = harness.app.shared.palette;
    let row = "l remove  h skip  m maybeboard  u undo";
    assert!(screen_text(&harness.app).contains(row));
    assert_eq!(
        word_colors(&harness.app, row, &["remove", "skip", "maybeboard", "undo"]),
        [
            palette.add.fg,
            palette.error.fg,
            palette.warning.fg,
            palette.tertiary.fg
        ]
    );
    assert!(!screen_text(&harness.app).contains(" keep "));
}

#[test]
fn remove_keeps_every_key_on_a_narrow_terminal() {
    let (harness, _, _) = loaded();
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc back");
}
