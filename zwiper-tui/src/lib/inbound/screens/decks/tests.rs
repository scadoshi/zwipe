use crossterm::event::KeyCode;
use zwipe_core::domain::deck::{DeckOtherTag, Format};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::{self, deck, deck_form};
use crate::test_support::{deck_profile, render, screen_text, user};

fn colored(name: &str, identity: &[&str]) -> DeckProfile {
    let mut profile = deck_profile(name);
    profile.color_identity = identity.iter().map(|&c| c.to_owned()).collect();
    profile
}

/// The deck list loaded with `decks` for a verified user, the hint closed and nothing queued.
fn loaded(decks: Vec<DeckProfile>) -> Harness {
    let mut harness = Harness::on(Screen::Decks(Box::new(DeckList::loading())));
    harness.app.shared.user = Some(user(true));
    harness.apply(Update::Loaded(decks));
    harness.app.shared.hint = None;
    harness.clear();
    harness
}

fn list(harness: &Harness) -> &DeckList {
    match harness.screen() {
        Screen::Decks(list) => list,
        other => panic!("not on the deck list: {other:?}"),
    }
}

#[test]
fn the_first_load_selects_the_first_deck_and_enter_opens_it() {
    let (first, second) = (deck_profile("First"), deck_profile("Second"));
    let mut harness = loaded(vec![second.clone(), first]);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Deck(view) if view.profile.id == second.id));
    assert_eq!(
        harness.pending(),
        [
            screens::Request::Deck(deck::Request::Load(second.id)),
            screens::Request::Deck(deck::Request::LoadOracleTags),
            screens::Request::MarkHintShown(deck::HINT_KEY.to_owned()),
        ]
    );
}

#[test]
fn a_loaded_list_shows_its_hint_once_per_visit() {
    let mut harness = Harness::on(Screen::Decks(Box::new(DeckList::loading())));
    harness.app.shared.user = Some(user(true));
    harness.apply(Update::Loaded(Vec::new()));
    assert!(
        harness.app.shared.hint.is_none(),
        "an empty list has no hint"
    );
    harness.apply(Update::Loaded(vec![deck_profile("A")]));
    assert!(screen_text(&harness.app).contains("Your decks"));
    assert_eq!(
        harness.pending(),
        [screens::Request::MarkHintShown(HINT_KEY.to_owned())]
    );
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('?'));
    assert!(harness.app.shared.hint.is_some(), "? opens it again");
}

#[test]
fn reloading_keeps_the_selection_and_refreshes_the_user_with_the_decks() {
    let mut harness = loaded(vec![deck_profile("A"), deck_profile("B")]);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('r'));
    assert!(matches!(harness.screen(), Screen::Decks(list) if list.loading && list.selected == 2));
    assert_eq!(harness.pending(), [screens::Request::Decks(Request::Load)]);
    let mut unverified = user(false);
    unverified.username = zwipe_core::domain::user::Username::new("fresh").unwrap();
    harness.apply(Update::LoadedWithUser(
        vec![deck_profile("A")],
        Box::new(unverified),
    ));
    assert_eq!(harness.app.shared.username.as_deref(), Some("fresh"));
}

#[test]
fn group_by_folds_the_list_and_headers_collapse() {
    let mut modern = deck_profile("Burn");
    modern.format = Some(Format::Modern);
    let mut harness = loaded(vec![modern, deck_profile("Pile")]);
    harness.press(KeyCode::Tab);
    assert_eq!(list(&harness).focus, Focus::List, "Tab stops at the list");
    harness.press(KeyCode::BackTab);
    harness.press(KeyCode::BackTab);
    assert_eq!(list(&harness).focus, Focus::GroupBy);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    assert_eq!(list(&harness).group_by, GroupBy::Format);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Modern (1)"));
    assert!(screen.contains("No format (1)"));

    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    assert_eq!(list(&harness).selected, 1, "still on the first deck");
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(list(&harness).collapsed.contains("No format"));
    assert_eq!(
        list(&harness).rows().len(),
        3,
        "the folded group hides its deck"
    );
}

#[test]
fn show_filters_by_color_and_tag_and_all_clears() {
    let mut tokens = colored("Tokens", &["W"]);
    tokens.tags = vec!["tokens".to_owned()];
    tokens.other_tags = vec![DeckOtherTag::Budget];
    let mut harness = loaded(vec![tokens, colored("Red", &["R"])]);
    harness.press(KeyCode::BackTab);
    assert_eq!(list(&harness).focus, Focus::Show);
    for _ in 0..4 {
        harness.press(KeyCode::Char('l'));
    }
    harness.press(KeyCode::Enter);
    assert!(list(&harness).show.colors.contains(&Color::Red));
    assert_eq!(list(&harness).groups()[0].decks.len(), 1);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    assert!(list(&harness).show.colors.contains(&Color::Green));
    assert!(screen_text(&harness.app).contains("No decks match"));

    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    assert!(list(&harness).tags_open, "the Tags chip opens the tag row");
    harness.press(KeyCode::Char('j'));
    assert_eq!(list(&harness).focus, Focus::Tags);
    harness.press(KeyCode::Enter);
    assert!(list(&harness).show.tags.contains("Budget"));
    harness.press(KeyCode::Char('k'));
    for _ in 0..10 {
        harness.press(KeyCode::Char('h'));
    }
    harness.press(KeyCode::Enter);
    assert!(list(&harness).show.is_all());
}

#[test]
fn picked_tags_stay_in_the_show_row_while_folded() {
    let mut tokens = deck_profile("Tokens");
    tokens.tags = vec!["tokens".to_owned()];
    let mut harness = loaded(vec![tokens]);
    harness.press(KeyCode::BackTab);
    {
        let Screen::Decks(list) = harness.screen_mut() else {
            panic!("not on the list");
        };
        list.show.toggle_tag("tokens");
    }
    let chips = list(&harness).show_chips();
    assert!(chips.contains(&ShowChip::Picked {
        key: "tokens".to_owned(),
        label: "Tokens".to_owned()
    }));
}

#[test]
fn create_is_refused_for_an_unverified_user_with_a_deck() {
    let mut harness = loaded(vec![deck_profile("A")]);
    harness.app.shared.user = Some(user(false));
    harness.press(KeyCode::Char('c'));
    assert!(matches!(harness.screen(), Screen::Decks(_)));
    let refused = &harness.app.shared.toasts.items()[0];
    assert_eq!(
        refused.message,
        "Verify your email to create more than 1 deck"
    );
    assert_eq!(
        refused.expires_at.duration_since(refused.shown_at),
        crate::inbound::widgets::toast::LONG,
        "a next step stays up long"
    );

    let mut empty = loaded(Vec::new());
    empty.app.shared.user = Some(user(false));
    empty.press(KeyCode::Char('c'));
    assert!(
        matches!(empty.screen(), Screen::DeckForm(form) if form.mode == deck_form::Mode::Create)
    );
}

#[test]
fn the_more_sheet_offers_two_screens_and_closes() {
    let mut harness = loaded(vec![deck_profile("A")]);
    harness.press(KeyCode::Char('m'));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("More actions"));
    assert!(screen.contains("Commander maybeboard"));
    assert!(screen.contains("Playstyle"));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Decks(list) if list.more.is_none()));
}

#[test]
fn more_opens_the_commander_maybeboard_and_playstyle() {
    let mut harness = loaded(vec![deck_profile("A")]);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Enter);
    assert!(list_under(&mut harness).is_some_and(|l| l.more.is_none()));
    assert!(matches!(harness.screen(), Screen::CommanderMaybeboard(_)));
    assert!(
        harness
            .pending()
            .contains(&screens::commander_maybeboard::Request::DeckProfiles.into())
    );

    let mut harness = loaded(vec![deck_profile("A")]);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Playstyle(_)));
    harness.app.shared.hint = None;
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Decks(_)));
}

/// The deck list under whatever is open over it.
fn list_under(harness: &mut Harness) -> Option<&DeckList> {
    harness.app.nav.screens_mut().find_map(|s| match s {
        Screen::Decks(list) => Some(&**list),
        _ => None,
    })
}

#[test]
fn states_read_as_the_app_words_them() {
    let mut harness = loaded(Vec::new());
    assert!(screen_text(&harness.app).contains("No decks"));
    harness.apply(Update::Failed("offline".to_owned()));
    assert!(screen_text(&harness.app).contains("Could not load decks"));
    assert_eq!(harness.app.shared.toasts.items()[0].message, "offline");
}

#[test]
fn rows_show_names_counts_chips_and_one_card_singular() {
    let mut first = colored("Simic Ramp", &["G", "U"]);
    first.card_count = 1;
    first.commander_name = Some("Tatyova".to_owned());
    first.format = Some(Format::Commander);
    first.tags = vec!["ramp".to_owned()];
    let harness = loaded(vec![first, deck_profile("Mono Red")]);
    let screen = screen_text(&harness.app);
    for expected in [
        "Group by",
        "Show",
        "All (2)",
        "> ",
        "Simic Ramp",
        "UG",
        "1 card ",
        "Commander",
        "Tatyova",
        "Mono Red",
        "0 cards",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_deck_list_fits_a_tiny_terminal() {
    let harness = loaded(vec![deck_profile("A")]);
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
    render(
        &Harness::on(Screen::Decks(Box::new(DeckList::loading()))).app,
        12,
        5,
    );
}
