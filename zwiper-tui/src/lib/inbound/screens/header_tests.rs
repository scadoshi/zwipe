//! Every screen's header sits on the first row, and its key bar names the key that opens Help & feedback.

use uuid::Uuid;

use super::{
    Screen, account, changelog, config, deck, deck_cards, deck_export, deck_form, deck_import,
    decks, privacy, remove_cards,
};
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{card, deck_profile, rows, user};

/// The sizes the owner checks a screen at.
const SIZES: [(u16, u16); 4] = [(200, 50), (120, 40), (80, 24), (60, 20)];

/// Each screen with a header, opened over home, and the title it shows.
fn screens() -> Vec<(&'static str, Harness)> {
    let on = |open: Box<dyn FnOnce(&mut crate::inbound::app::Context<'_>)>| {
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.user = Some(user(true));
        harness.open(open);
        harness.app.shared.hint = None;
        harness
    };
    let id = Uuid::new_v4();
    vec![
        ("Home", Harness::on(Screen::Home(Box::default()))),
        ("Decks", on(Box::new(decks::open))),
        (
            "Deck",
            on(Box::new(|cx| deck::open(cx, deck_profile("Pile")))),
        ),
        ("Export", on(Box::new(move |cx| deck_export::open(cx, id)))),
        ("Import", on(Box::new(move |cx| deck_import::open(cx, id)))),
        (
            "Deck cards",
            on(Box::new(move |cx| deck_cards::open(cx, id))),
        ),
        (
            "Remove deck cards",
            on(Box::new(move |cx| remove_cards::open(cx, id))),
        ),
        ("Create deck", on(Box::new(deck_form::open_create))),
        ("Add deck cards", Harness::swiping(vec![card("Krenko")]).0),
        ("Account", on(Box::new(account::open))),
        ("Config", on(Box::new(config::open))),
        ("Privacy policy", on(Box::new(privacy::open))),
        ("Changelog", on(Box::new(changelog::open))),
        ("Login", Harness::new()),
    ]
}

#[test]
fn every_header_holds_the_first_row_with_no_help_corner() {
    for (title, harness) in screens() {
        for (width, height) in SIZES {
            let rows = rows(&harness.app, width, height);
            let first = rows.first().unwrap();
            assert!(
                first.contains(title),
                "{title} is not on the first row at {width}x{height}:\n{}",
                rows.join("\n")
            );
            assert!(
                !first.trim_start().starts_with('!'),
                "{title}'s header still draws ! at {width}x{height}: {first:?}"
            );
        }
    }
}

#[test]
fn the_key_bar_names_f1_help_on_every_screen() {
    let help = "F1 help";
    for (title, harness) in screens() {
        for (width, height) in [(120, 40), (60, 20)] {
            let rows = rows(&harness.app, width, height);
            let bar = rows.iter().rev().take(4).any(|row| row.contains(help));
            assert!(
                bar,
                "{title}'s key bar has no {help:?} at {width}x{height}:\n{}",
                rows.join("\n")
            );
            let count: usize = rows.iter().map(|row| row.matches(" help").count()).sum();
            assert_eq!(count, 1, "{title} shows one help hint at {width}x{height}");
        }
    }
}
