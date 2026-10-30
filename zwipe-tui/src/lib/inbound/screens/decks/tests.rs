use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::deck;
use crate::test_support::{deck_profile, render, screen_text};

#[test]
fn decks_sort_by_name_ignoring_case() {
    let mut list = DeckList::loading();
    list.set_decks(vec![
        deck_profile("satya"),
        deck_profile("Big fat chud"),
        deck_profile("isshin"),
    ]);
    let names: Vec<String> = list.decks.iter().map(|d| d.name.to_string()).collect();
    assert_eq!(names, ["Big fat chud", "isshin", "satya"]);
    assert!(!list.loading);
}

#[test]
fn an_empty_deck_list_keeps_the_selection_at_zero() {
    let mut list = DeckList::default();
    list.next();
    list.previous();
    assert_eq!(list.selected, 0);
    assert!(list.selected_deck().is_none());
}

#[test]
fn enter_on_a_deck_opens_its_screen() {
    let mut harness = Harness::on(Screen::Decks(DeckList::loading()));
    let (first, second) = (deck_profile("First"), deck_profile("Second"));
    harness.apply(Update::Loaded(vec![first, second.clone()]));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.profile.id == second.id));
    assert_eq!(
        harness.pending(),
        [screens::Request::Deck(deck::Request::Load(second.id))]
    );
}

#[test]
fn reloading_keeps_the_selection() {
    let mut harness = Harness::on(Screen::Decks(DeckList::loading()));
    harness.apply(Update::Loaded(vec![deck_profile("A"), deck_profile("B")]));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('r'));
    assert!(matches!(harness.screen(), Screen::Decks(list) if list.loading && list.selected == 1));
    assert_eq!(harness.pending(), [screens::Request::Decks(Request::Load)]);
}

#[test]
fn the_deck_list_shows_names_counts_and_the_selection() {
    let mut first = deck_profile("Simic Ramp");
    first.card_count = 42;
    first.commander_name = Some("Tatyova".to_owned());
    let harness = Harness::on(Screen::Decks(DeckList {
        decks: vec![first, deck_profile("Mono Red")],
        ..DeckList::default()
    }));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("> Simic Ramp"));
    assert!(screen.contains("Tatyova"));
    assert!(screen.contains("42 cards"));
    assert!(screen.contains("Mono Red"));
}

#[test]
fn the_deck_list_fits_a_tiny_terminal() {
    let harness = Harness::on(Screen::Decks(DeckList::default()));
    render(&harness.app, 12, 5);
}
