use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, render, screen_text};

/// Three printings of one oracle card, with distinct sets.
#[expect(
    clippy::arithmetic_side_effects,
    reason = "a test fixture: collector numbers count up from 100"
)]
fn printings_of(oracle_id: Uuid) -> Vec<Card> {
    ["Alpha", "Beta", "Unlimited"]
        .into_iter()
        .enumerate()
        .map(|(i, set)| {
            let mut printing = card("Llanowar Elves");
            printing.scryfall_data.oracle_id = Some(oracle_id);
            printing.scryfall_data.set_name = set.to_owned();
            printing.scryfall_data.set = set.get(..3).unwrap().to_lowercase();
            printing.scryfall_data.collector_number = format!("{}", 100 + i);
            printing
        })
        .collect()
}

/// Home with the printings of the second of three printings open in `mode`, loaded, and nothing queued.
fn on_printings(mode: Mode) -> (Harness, Vec<Card>) {
    let oracle_id = Uuid::new_v4();
    let printings = printings_of(oracle_id);
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let opened_on = printings[1].clone();
    harness.open(|cx| open(cx, opened_on, mode));
    assert_eq!(
        harness.pending(),
        [Request::Fetch { oracle_id }.into()],
        "opening fetches by oracle id"
    );
    harness.clear();
    harness.apply(Update::Loaded {
        oracle_id,
        printings: printings.clone(),
    });
    (harness, printings)
}

fn view(harness: &Harness) -> &PrintingsView {
    match harness.screen() {
        Screen::Printings(view) => view,
        other => panic!("not the printings: {other:?}"),
    }
}

fn toasts(harness: &Harness) -> Vec<String> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| t.message.clone())
        .collect()
}

#[test]
fn the_list_opens_on_the_cards_own_printing() {
    let (harness, _) = on_printings(Mode::Pick);
    assert_eq!(view(&harness).selected, 1);
    assert!(!view(&harness).changed());
    let screen = screen_text(&harness.app);
    for expected in [
        "Printings",
        "Alpha",
        "ALP #100",
        "Beta",
        "current",
        "i Image",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
    assert!(!screen.contains("Enter Save"), "nothing to save yet");
}

#[test]
fn enter_picks_another_printing_for_the_screen_under() {
    let (mut harness, printings) = on_printings(Mode::Pick);
    harness.press(KeyCode::Enter);
    assert!(
        matches!(harness.screen(), Screen::Printings(_)),
        "the same printing is no pick"
    );
    harness.press(KeyCode::Char('j'));
    assert!(screen_text(&harness.app).contains("Enter Save"));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert_eq!(
        harness.app.shared.caches.handoff.printing.as_ref(),
        Some(&printings[2])
    );
    assert_eq!(toasts(&harness), ["Printing saved"]);
}

#[test]
fn leaving_with_another_printing_selected_says_it_is_unchanged() {
    let (mut harness, _) = on_printings(Mode::Pick);
    harness.press(KeyCode::Char('k'));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert!(harness.app.shared.caches.handoff.printing.is_none());
    assert_eq!(toasts(&harness), ["Printing unchanged"]);
}

#[test]
fn read_only_never_picks() {
    let (mut harness, _) = on_printings(Mode::ReadOnly);
    harness.press(KeyCode::Down);
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Printings(_)));
    assert!(!screen_text(&harness.app).contains("Enter Save"));
    harness.press(KeyCode::Esc);
    assert!(harness.app.shared.caches.handoff.printing.is_none());
    assert!(
        toasts(&harness).is_empty(),
        "read only has nothing to leave unchanged"
    );
}

#[test]
fn i_shows_the_selected_printing() {
    let (mut harness, printings) = on_printings(Mode::ReadOnly);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('i'));
    let Screen::ImagePreview(preview) = harness.screen() else {
        panic!("no preview");
    };
    assert_eq!(preview.card.scryfall_data.id, printings[2].scryfall_data.id);
}

#[test]
fn a_failed_fetch_toasts_and_a_late_one_for_another_card_is_ignored() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let elves = card("Llanowar Elves");
    let oracle_id = elves.scryfall_data.oracle_id.unwrap();
    harness.open(|cx| open(cx, elves, Mode::Pick));
    harness.apply(Update::Loaded {
        oracle_id: Uuid::new_v4(),
        printings: Vec::new(),
    });
    assert!(view(&harness).loading);
    assert!(screen_text(&harness.app).contains('░'));
    harness.apply(Update::Failed {
        oracle_id,
        message: "Server unavailable".to_owned(),
    });
    assert!(!view(&harness).loading);
    assert_eq!(toasts(&harness), ["Server unavailable"]);
}

#[test]
fn a_card_without_an_oracle_id_shows_only_itself() {
    let mut harness = Harness::on(Screen::Home(Box::default()));
    let mut token = card("Soldier");
    token.scryfall_data.oracle_id = None;
    harness.open(|cx| open(cx, token, Mode::Pick));
    assert!(harness.pending().is_empty(), "nothing to fetch by");
    assert_eq!(view(&harness).printings.len(), 1);
}

#[test]
fn the_printings_fit_a_tiny_terminal() {
    let (harness, _) = on_printings(Mode::Pick);
    render(&harness.app, 8, 3);
    render(&harness.app, 1, 1);
}

#[test]
fn the_printings_keep_every_key_on_a_narrow_terminal() {
    let (harness, _) = on_printings(Mode::Pick);
    crate::test_support::assert_key_bar_fits(&harness.app, "i Image");
}
