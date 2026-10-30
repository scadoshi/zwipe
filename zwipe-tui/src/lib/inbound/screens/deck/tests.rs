use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck, deck_profile, render, screen_text};

#[test]
fn s_on_a_loaded_deck_starts_swiping_and_fetches_the_first_page() {
    let profile = deck_profile("Pile");
    let mut harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(profile.clone()))));
    harness.press(KeyCode::Char('s'));
    assert!(
        matches!(harness.screen(), Screen::Deck(_)),
        "nothing to swipe before the deck loads"
    );

    harness.apply(Update::Loaded(Box::new(deck(profile.clone(), Vec::new()))));
    harness.press(KeyCode::Char('s'));
    let view = harness.swipe_view();
    assert!(view.stack.is_some() && view.fetching);
    assert!(matches!(
        harness.pending(),
        [screens::Request::Swipe(screens::swipe::Request::FetchCards { deck_id, .. })] if *deck_id == profile.id
    ));
}

#[test]
fn the_deck_screen_steps_tabs_currency_and_turn() {
    let profile = deck_profile("Pile");
    let mut harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(profile.clone()))));
    harness.apply(Update::Loaded(Box::new(deck(profile, vec![card("A")]))));

    let deck_view = |harness: &Harness| match harness.screen() {
        Screen::Deck(view) => (view.section, view.currency, view.turn, view.sections()),
        other => panic!("left the deck screen: {other:?}"),
    };
    let (section, _, _, sections) = deck_view(&harness);
    assert_eq!(section, Section::Budget);
    assert!(sections.contains(&Section::Budget));

    harness.press(KeyCode::Char('c'));
    harness.press(KeyCode::Char('-'));
    for _ in 0..25 {
        harness.press(KeyCode::Char('+'));
    }
    let (_, currency, turn, _) = deck_view(&harness);
    assert_eq!(currency, PriceCurrency::Eur);
    assert_eq!(turn, MAX_TURN);

    let count = sections.len();
    for _ in 0..count {
        harness.press(KeyCode::Tab);
    }
    assert_eq!(
        deck_view(&harness).0,
        Section::Budget,
        "Tab wraps back around"
    );
    harness.press(KeyCode::BackTab);
    assert_eq!(Some(&deck_view(&harness).0), sections.last());
}

#[test]
fn a_deck_result_for_another_deck_is_ignored() {
    let mut harness = Harness::on(Screen::Deck(Box::new(DeckView::opening(deck_profile(
        "Open",
    )))));
    harness.apply(Update::Loaded(Box::new(deck(
        deck_profile("Other"),
        Vec::new(),
    ))));
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.deck.is_none()));
}

fn loaded_deck_screen(section: Section) -> Harness {
    let creature = |name: &str, cost: &str, cmc: f64| {
        let mut c = card(name);
        c.scryfall_data.mana_cost = Some(cost.to_owned());
        c.scryfall_data.cmc = Some(cmc);
        c.scryfall_data.type_line = Some("Creature — Elf".to_owned());
        c
    };
    let mut forest = card("Forest");
    forest.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
    let mut profile = deck_profile("Elves");
    profile.land_target = Some(36);
    let mut view = DeckView::opening(profile.clone());
    view.load(deck(
        profile,
        vec![
            creature("A", "{G}", 1.0),
            creature("B", "{1}{G}", 2.0),
            forest,
        ],
    ));
    view.section = section;
    Harness::on(Screen::Deck(Box::new(view)))
}

#[test]
fn the_deck_screen_shows_the_profile_and_tabs() {
    let screen = screen_text(&loaded_deck_screen(Section::Budget).app);
    for expected in [
        "Elves",
        "Profile",
        "Cards",
        "Budget",
        "Distributions",
        "Mana",
        "Draw odds",
        "USD",
        "Total price",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_mana_tab_shows_lands_against_the_target_and_the_charts() {
    let screen = screen_text(&loaded_deck_screen(Section::Mana).app);
    for expected in [
        "1 / 36",
        "Average mana value",
        "Mana curve",
        "Colors",
        "Mana cost fulfillment",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_draw_odds_tab_counts_from_the_opening_hand() {
    let screen = screen_text(&loaded_deck_screen(Section::DrawOdds).app);
    assert!(screen.contains("Odds of ≥1 in opening hand"));
    assert!(screen.contains("LAND"));
    assert!(
        screen.contains("100%"),
        "every land is drawn when the hand is bigger than the deck"
    );
}

#[test]
fn the_distributions_tab_shows_types_and_average_stats() {
    let screen = screen_text(&loaded_deck_screen(Section::Distributions).app);
    assert!(screen.contains("Average P/T"));
    assert!(screen.contains("Types"));
    assert!(screen.contains("creat"));
}

#[test]
fn the_deck_screen_fits_a_tiny_terminal() {
    render(&loaded_deck_screen(Section::Mana).app, 12, 5);
}
