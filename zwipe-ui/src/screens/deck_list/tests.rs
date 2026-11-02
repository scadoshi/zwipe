use zwipe_core::domain::deck::{DeckOtherTag, Format};

use super::*;
use crate::deck::VERIFY_TO_CREATE;
use crate::test_support::deck_profile;
use crate::toast::{LONG, ToastKind};

fn deck(name: &str, format: Option<Format>, identity: &[&str], tags: &[&str]) -> DeckProfile {
    let mut profile = deck_profile(name);
    profile.format = format;
    profile.color_identity = identity.iter().map(|&c| c.to_owned()).collect();
    profile.tags = tags.iter().map(|&t| t.to_owned()).collect();
    profile
}

/// Three decks: a Simic tokens Commander deck, a mono red Modern deck and a colorless deck with no format or tag.
fn library() -> Vec<DeckProfile> {
    vec![
        deck("simic", Some(Format::Commander), &["G", "U"], &["tokens"]),
        deck("Burn", Some(Format::Modern), &["R"], &[]),
        deck("Artifacts", None, &[], &[]),
    ]
}

/// A state whose fetch is out, with the ticket it holds.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load);
    let [Effect::RefreshUser, Effect::FetchDecks(ticket)] = effects.as_slice() else {
        panic!("Load fetches: {effects:?}");
    };
    (state, *ticket)
}

/// A state with `decks` loaded and the hint already offered.
fn loaded(decks: Vec<DeckProfile>) -> State {
    let (mut state, ticket) = loading();
    let _ = state.update(Action::Loaded { ticket, decks });
    state
}

fn headers(state: &State) -> Vec<String> {
    state.groups().iter().map(DeckGroup::key).collect()
}

fn names(state: &State) -> Vec<Vec<String>> {
    state
        .groups()
        .iter()
        .map(|g| g.decks.iter().map(|d| d.name.to_string()).collect())
        .collect()
}

#[test]
fn a_new_state_is_loading_with_nothing_picked() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert!(!state.chips_shown());
    assert_eq!(state.message(), None);
    assert_eq!(state.groups(), []);
    assert_eq!(state.decks(), []);
    assert_eq!(state.group_by(), GroupBy::None);
    assert!(state.show().is_all());
    assert!(!state.tags_open());
}

#[test]
fn load_refreshes_the_user_then_fetches_the_decks() {
    let (state, _) = loading();
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn the_first_list_with_a_deck_offers_the_hint_once() {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Loaded {
        ticket,
        decks: library(),
    });
    assert_eq!(effects, [Effect::OfferHint(OneTimeHint::DeckList)]);
    assert_eq!(state.status(), Status::Loaded);
    assert!(state.chips_shown());
    assert_eq!(state.decks().len(), 3);

    let effects = state.update(Action::Load);
    let [_, Effect::FetchDecks(ticket)] = effects.as_slice() else {
        panic!("Load fetches: {effects:?}");
    };
    let effects = state.update(Action::Loaded {
        ticket: *ticket,
        decks: library(),
    });
    assert!(effects.is_empty(), "the hint came back: {effects:?}");
}

#[test]
fn an_empty_list_says_no_decks_and_saves_the_hint() {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Loaded {
        ticket,
        decks: Vec::new(),
    });
    assert!(effects.is_empty(), "an empty list offered the hint");
    assert_eq!(state.status(), Status::Empty);
    assert_eq!(state.message(), Some(NO_DECKS));
    assert!(!state.chips_shown());
    assert_eq!(state.groups(), []);

    let effects = state.update(Action::Load);
    let [_, Effect::FetchDecks(ticket)] = effects.as_slice() else {
        panic!("Load fetches: {effects:?}");
    };
    let effects = state.update(Action::Loaded {
        ticket: *ticket,
        decks: library(),
    });
    assert_eq!(
        effects,
        [Effect::OfferHint(OneTimeHint::DeckList)],
        "the hint waits for a list with a deck"
    );
}

#[test]
fn a_failed_fetch_says_so_without_chip_rows() {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Failed { ticket });
    assert!(
        effects.is_empty(),
        "the client raises the error toast: {effects:?}"
    );
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.message(), Some(LOAD_FAILED));
    assert!(!state.chips_shown());
    assert_eq!(state.groups(), []);
    assert_eq!(state.tags(), []);
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, older) = loading();
    let _ = state.update(Action::Load);
    let effects = state.update(Action::Loaded {
        ticket: older,
        decks: library(),
    });
    assert!(effects.is_empty(), "a stale reply offered the hint");
    assert_eq!(state.status(), Status::Loading, "a stale reply landed");
    let _ = state.update(Action::Failed { ticket: older });
    assert_eq!(state.status(), Status::Loading, "a stale failure landed");
}

#[test]
fn a_reload_keeps_group_by_show_and_the_folds() {
    let mut state = loaded(library());
    let _ = state.update(Action::PickGroupBy(GroupBy::Format));
    let _ = state.update(Action::ToggleColor(Color::Red));
    let _ = state.update(Action::ToggleTagFold);
    let _ = state.update(Action::ToggleGroup("Modern".to_owned()));
    let effects = state.update(Action::Load);
    let [_, Effect::FetchDecks(ticket)] = effects.as_slice() else {
        panic!("Load fetches: {effects:?}");
    };
    assert!(!state.chips_shown(), "the chip rows ghost while it loads");
    let _ = state.update(Action::Loaded {
        ticket: *ticket,
        decks: library(),
    });
    assert_eq!(state.group_by(), GroupBy::Format);
    assert!(state.show().colors.contains(&Color::Red));
    assert!(state.tags_open());
    assert!(state.is_collapsed("Modern"));
}

#[test]
fn every_group_by_mode_folds_the_shown_decks() {
    let mut state = loaded(library());
    assert_eq!(headers(&state), ["All"]);
    assert_eq!(names(&state), [["Artifacts", "Burn", "simic"]]);

    let _ = state.update(Action::PickGroupBy(GroupBy::Format));
    assert_eq!(headers(&state), ["Commander", "Modern", "No format"]);

    let _ = state.update(Action::PickGroupBy(GroupBy::Color));
    assert_eq!(headers(&state), ["R", "UG", "Colorless"]);

    let _ = state.update(Action::PickGroupBy(GroupBy::Tag));
    assert_eq!(headers(&state), ["Tokens", "Untagged"]);
    assert_eq!(names(&state), [vec!["simic"], vec!["Artifacts", "Burn"]]);
}

#[test]
fn show_narrows_every_group_by_mode() {
    let mut state = loaded(library());
    let _ = state.update(Action::ToggleColor(Color::Green));
    for (by, want) in [
        (GroupBy::None, vec!["All"]),
        (GroupBy::Format, vec!["Commander"]),
        (GroupBy::Color, vec!["UG"]),
        (GroupBy::Tag, vec!["Tokens"]),
    ] {
        let _ = state.update(Action::PickGroupBy(by));
        assert_eq!(headers(&state), want, "{by:?}");
        assert_eq!(names(&state), [["simic"]], "{by:?}");
    }
}

#[test]
fn colors_must_all_be_in_the_identity_and_any_tag_passes() {
    let mut state = loaded(library());
    let _ = state.update(Action::ToggleColor(Color::Green));
    let _ = state.update(Action::ToggleColor(Color::Red));
    assert_eq!(state.message(), Some(NO_MATCH));
    let _ = state.update(Action::ToggleColor(Color::Red));
    let _ = state.update(Action::ToggleTag("tokens".to_owned()));
    assert_eq!(names(&state), [["simic"]]);
    let _ = state.update(Action::ShowAll);
    assert!(state.show().is_all());
    assert_eq!(state.message(), None);
}

#[test]
fn no_match_keeps_the_all_group_only_when_ungrouped() {
    let mut state = loaded(library());
    let _ = state.update(Action::ToggleColor(Color::White));
    assert_eq!(state.message(), Some(NO_MATCH));
    assert_eq!(headers(&state), ["All"], "the app draws All (0)");
    assert!(state.groups().iter().all(|g| g.decks.is_empty()));
    for by in [GroupBy::Format, GroupBy::Color, GroupBy::Tag] {
        let _ = state.update(Action::PickGroupBy(by));
        assert!(state.groups().is_empty(), "{by:?}");
    }
}

#[test]
fn a_group_collapses_and_stays_collapsed_across_group_by() {
    let mut state = loaded(library());
    let _ = state.update(Action::ToggleGroup("All".to_owned()));
    assert!(state.is_collapsed("All"));
    let _ = state.update(Action::PickGroupBy(GroupBy::Format));
    let _ = state.update(Action::PickGroupBy(GroupBy::None));
    assert!(state.is_collapsed("All"), "the app keeps collapsed groups");
    let _ = state.update(Action::ToggleGroup("All".to_owned()));
    assert!(!state.is_collapsed("All"));
}

#[test]
fn the_show_row_has_no_tag_fold_without_tags() {
    let state = loaded(vec![deck("Burn", None, &["R"], &[])]);
    let chips = state.show_chips();
    assert_eq!(chips.len(), 6);
    assert_eq!(chips[0], ShowChip::All);
    assert_eq!(chips[1], ShowChip::Color(Color::White));
    assert_eq!(state.tags(), []);
}

#[test]
fn picked_tags_stay_in_the_show_row_while_folded() {
    let mut other = deck("Jank", None, &[], &[]);
    other.other_tags = vec![DeckOtherTag::Jank];
    let mut decks = library();
    decks.push(other);
    let mut state = loaded(decks);
    assert_eq!(state.tags().len(), 2);
    assert_eq!(state.show_chips().last(), Some(&ShowChip::TagFold));

    let _ = state.update(Action::ToggleTag("Jank".to_owned()));
    let jank = ShowChip::Picked {
        key: "Jank".to_owned(),
        label: "Jank".to_owned(),
    };
    assert_eq!(state.show_chips().last(), Some(&jank));
    assert!(state.is_picked(&jank));
    assert!(!state.is_picked(&ShowChip::All));

    let _ = state.update(Action::ToggleTagFold);
    assert!(state.is_picked(&ShowChip::TagFold));
    assert!(state.folded_picks().is_empty(), "the open row shows them");
    assert_eq!(state.show_chips().last(), Some(&ShowChip::TagFold));

    let _ = state.update(jank.action());
    assert!(state.show().is_all());
}

#[test]
fn each_show_chip_presses_its_action() {
    let mut state = loaded(library());
    for chip in state.show_chips() {
        let _ = state.update(chip.action());
    }
    assert_eq!(
        state.show().colors.len(),
        5,
        "All, then each color picked once"
    );
    assert!(state.tags_open());
    assert!(!state.is_picked(&ShowChip::All));
}

#[test]
fn create_opens_the_form_for_a_verified_user() {
    let mut state = loaded(library());
    assert_eq!(
        state.update(Action::Create { verified: true }),
        [Effect::Navigate(Destination::CreateDeck)]
    );
}

#[test]
fn create_warns_an_unverified_user_holding_a_deck() {
    let mut state = loaded(library());
    let effects = state.update(Action::Create { verified: false });
    let [Effect::Toast(toast)] = effects.as_slice() else {
        panic!("the gate warns: {effects:?}");
    };
    assert_eq!(toast.kind, ToastKind::Warning);
    assert_eq!(toast.text, VERIFY_TO_CREATE);
    assert_eq!(toast.duration, LONG);
}

#[test]
fn create_lets_an_unverified_user_with_no_deck_through() {
    let (mut state, ticket) = loading();
    assert_eq!(
        state.update(Action::Create { verified: false }),
        [Effect::Navigate(Destination::CreateDeck)],
        "a list still loading counts as no deck"
    );
    let _ = state.update(Action::Loaded {
        ticket,
        decks: Vec::new(),
    });
    assert_eq!(
        state.update(Action::Create { verified: false }),
        [Effect::Navigate(Destination::CreateDeck)]
    );
}

#[test]
fn rows_back_and_more_navigate() {
    let mut state = loaded(library());
    let id = state.decks()[0].id;
    assert_eq!(
        state.update(Action::OpenDeck(id)),
        [Effect::Navigate(Destination::Deck(id))]
    );
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Home)]
    );
    assert_eq!(
        state.update(Action::More(MoreItem::CommanderMaybeboard)),
        [Effect::Navigate(Destination::CommanderMaybeboard)]
    );
    assert_eq!(
        state.update(Action::More(MoreItem::Playstyle)),
        [Effect::Navigate(Destination::Playstyle)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [
            TITLE,
            GROUP_BY,
            SHOW,
            SHOW_ALL,
            TAGS,
            LOAD_FAILED,
            NO_DECKS,
            NO_MATCH
        ],
        [
            "Decks",
            "Group by",
            "Show",
            "All",
            "Tags",
            "Could not load decks",
            "No decks",
            "No decks match"
        ]
    );
    assert_eq!(
        [BACK, CREATE, MORE, MORE_TITLE],
        ["Back", "Create", "More", "More actions"]
    );
    let labels: Vec<&str> = MoreItem::ALL.iter().map(|i| i.label()).collect();
    assert_eq!(labels, ["Commander maybeboard", "Playstyle"]);
}
