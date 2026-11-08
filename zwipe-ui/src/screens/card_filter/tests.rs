use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_type::CardType;
use zwipe_core::domain::deck::Format;

use super::*;
use crate::filter::AddContext;
use crate::filter::draft::ResetRule;
use crate::filter::fields::{Match, MatchGroup};
use crate::toast::{NORMAL, QUICK};

/// A host's config: Format shown or not, the empty-search check, and its Reset rule.
fn config(show_format: bool, validate: bool, reset: ResetRule) -> SheetConfig {
    SheetConfig {
        show_format,
        validate,
        reset,
        sources: Sources::default(),
        deck_format: None,
    }
}

fn open_action(filter: &CardQueryBuilder, config: SheetConfig, notice: ResetNotice) -> Action {
    Action::Open {
        filter: Box::new(filter.clone()),
        config: Box::new(config),
        notice,
    }
}

/// A state with the sheet open over `filter` for a host like the deck cards view, the hint already offered.
fn opened_over(filter: &CardQueryBuilder) -> State {
    let mut state = State::new();
    let _ = state.update(open_action(
        filter,
        config(true, false, ResetRule::Plain),
        ResetNotice::Toast,
    ));
    state
}

fn named(name: &str) -> CardQueryBuilder {
    let mut b = CardQueryBuilder::default();
    b.set_name_contains(name);
    b
}

fn toasts(effects: &[Effect]) -> Vec<(ToastKind, &str)> {
    effects
        .iter()
        .filter_map(|effect| match effect {
            Effect::Toast(toast) => Some((toast.kind, toast.text.as_str())),
            _ => None,
        })
        .collect()
}

fn edit(state: &mut State, filter: &CardQueryBuilder) {
    let effects = state.update(Action::Edit(Box::new(filter.clone())));
    assert!(effects.is_empty(), "an edit asks nothing: {effects:?}");
}

#[test]
fn a_new_sheet_is_closed_with_no_draft() {
    let state = State::new();
    assert!(!state.is_open());
    assert_eq!(state.draft(), None);
    assert_eq!(state.applied(), None);
    assert!(!state.unchanged(), "Apply only greys out on an open sheet");
    assert_eq!(state.sections(), Section::shown(false));
    assert_eq!(state.active_sections(), Vec::<Section>::new());
    assert_eq!(state.dictionary(), None);
    assert_eq!(state.modes(), None);
    assert_eq!(state.sources(), None);
}

#[test]
fn the_first_opening_offers_the_hint_and_every_opening_asks_for_the_lists() {
    let mut state = State::new();
    let open = open_action(
        &CardQueryBuilder::default(),
        config(false, false, ResetRule::Plain),
        ResetNotice::Toast,
    );
    assert_eq!(
        state.update(open.clone()),
        [
            Effect::OfferHint(OneTimeHint::Filter),
            Effect::EnsureLists(&Kind::FILTER),
        ]
    );
    assert!(state.is_open());
    assert_eq!(
        state.update(open.clone()),
        [],
        "opening an open sheet does nothing"
    );
    let _ = state.update(Action::Back);
    assert_eq!(
        state.update(open),
        [Effect::EnsureLists(&Kind::FILTER)],
        "the hint is offered once a visit"
    );
}

#[test]
fn a_deck_host_asks_only_for_roles_and_tags() {
    let mut state = State::new();
    let deck = SheetConfig {
        sources: Sources::deck(&[]),
        ..SheetConfig::default()
    };
    let effects = state.update(open_action(
        &CardQueryBuilder::default(),
        deck,
        ResetNotice::Toast,
    ));
    assert_eq!(
        effects.last(),
        Some(&Effect::EnsureLists(&[Kind::CardRoles, Kind::OracleTags]))
    );
}

#[test]
fn opening_copies_the_hosts_filter_as_the_draft_and_what_it_applied() {
    let state = opened_over(&named("elf"));
    assert_eq!(state.draft(), Some(&named("elf")));
    assert_eq!(state.applied(), Some(&named("elf")));
    assert!(state.unchanged());
    assert_eq!(state.sections(), Section::shown(true));
    assert_eq!(state.active_sections(), [Section::Name]);
    assert_eq!(state.sources(), Some(&Sources::Catalog));
}

#[test]
fn the_lists_warning_shows_once_until_the_sheet_closes() {
    let mut state = opened_over(&CardQueryBuilder::default());
    let warning = state.update(Action::ListsFailed);
    assert_eq!(
        warning,
        [Effect::Toast(Toast::new(
            ToastKind::Warning,
            LISTS_FAILED,
            NORMAL
        ))]
    );
    assert_eq!(state.update(Action::ListsFailed), []);
    let _ = state.update(Action::Back);
    assert_eq!(
        state.update(Action::ListsFailed),
        warning,
        "a failure seen as the sheet reopens warns again, whichever comes first"
    );
}

#[test]
fn edits_change_the_draft_only_while_open() {
    let mut state = opened_over(&CardQueryBuilder::default());
    edit(&mut state, &named("bolt"));
    assert_eq!(state.draft(), Some(&named("bolt")));
    assert!(!state.unchanged());
    edit(&mut state, &CardQueryBuilder::default());
    assert!(
        state.unchanged(),
        "editing back to the start greys Apply again"
    );

    let _ = state.update(Action::Back);
    edit(&mut state, &named("late"));
    assert_eq!(
        state.draft(),
        Some(&CardQueryBuilder::default()),
        "a closed sheet takes no edits"
    );
}

#[test]
fn apply_does_nothing_while_the_draft_is_unchanged() {
    let mut state = opened_over(&named("elf"));
    assert_eq!(state.update(Action::Apply), []);
    assert!(state.is_open());
}

#[test]
fn apply_hands_the_draft_to_the_host_then_toasts_and_closes() {
    let mut state = opened_over(&CardQueryBuilder::default());
    edit(&mut state, &named("bolt"));
    assert_eq!(
        state.update(Action::Apply),
        [
            Effect::Apply(Box::new(named("bolt"))),
            Effect::Toast(Toast::new(ToastKind::Success, "Filter applied", QUICK)),
            Effect::Close,
        ]
    );
    assert!(!state.is_open());
    assert_eq!(state.applied(), None);
    assert_eq!(
        state.draft(),
        Some(&named("bolt")),
        "the draft stays readable while the sheet slides away"
    );
    assert_eq!(
        state.update(Action::Back),
        [],
        "the close that follows Apply drops nothing"
    );
}

#[test]
fn applying_an_emptied_filter_says_it_cleared() {
    let mut state = opened_over(&named("bolt"));
    edit(&mut state, &CardQueryBuilder::default());
    let effects = state.update(Action::Apply);
    assert_eq!(toasts(&effects), [(ToastKind::Success, "Filter cleared")]);
    assert_eq!(effects.first(), Some(&Effect::Apply(Box::default())));
}

#[test]
fn apply_refuses_a_contradiction_and_keeps_the_sheet_open() {
    let mut state = opened_over(&CardQueryBuilder::default());
    let mut both = CardQueryBuilder::default();
    both.set_card_type_contains_any(vec![CardType::Land]);
    both.set_card_type_excludes_any(vec![CardType::Land]);
    edit(&mut state, &both);
    assert_eq!(
        state.update(Action::Apply),
        [Effect::Toast(Toast::new(
            ToastKind::Warning,
            "Filter can't both include and exclude card types",
            NORMAL
        ))]
    );
    assert!(state.is_open());
}

#[test]
fn adds_search_refuses_an_empty_filter_over_an_empty_one() {
    let mut state = State::new();
    let _ = state.update(open_action(
        &CardQueryBuilder::default(),
        config(true, true, ResetRule::Plain),
        ResetNotice::Toast,
    ));
    // A page size is config, not a filter.
    let mut paged = CardQueryBuilder::default();
    paged.set_limit(50);
    edit(&mut state, &paged);
    assert_eq!(
        state.update(Action::Apply),
        [Effect::Toast(Toast::new(
            ToastKind::Warning,
            "Filter is already empty",
            QUICK
        ))]
    );
    assert!(state.is_open());
}

#[test]
fn back_with_edits_puts_the_filter_back_and_says_so() {
    let mut state = opened_over(&named("elf"));
    edit(&mut state, &named("goblin"));
    assert_eq!(
        state.update(Action::Back),
        [
            Effect::Restore(Box::new(named("elf"))),
            Effect::Toast(Toast::new(ToastKind::Info, "Filter unchanged", QUICK)),
            Effect::Close,
        ]
    );
    assert!(!state.is_open());
}

#[test]
fn back_without_edits_only_closes() {
    let mut state = opened_over(&named("elf"));
    assert_eq!(state.update(Action::Back), [Effect::Close]);
    assert_eq!(
        state.update(Action::Back),
        [],
        "a closed sheet has nothing to close"
    );
}

#[test]
fn a_plain_reset_clears_and_toasts_every_time() {
    let mut state = opened_over(&named("elf"));
    let reset = [Effect::Toast(Toast::new(
        ToastKind::Info,
        "Filter reset",
        QUICK,
    ))];
    assert_eq!(state.update(Action::Reset), reset);
    assert_eq!(state.draft(), Some(&CardQueryBuilder::default()));
    assert_eq!(state.update(Action::Reset), reset, "the app toasts again");
    assert!(state.is_open(), "Reset stages; Apply commits");
}

#[test]
fn adds_resets_keep_synergy_or_say_they_are_at_default() {
    let context = AddContext {
        format: Some(Format::Commander),
        has_commander: true,
        ..AddContext::default()
    };
    let fresh = context.fresh_search();
    let mut state = State::new();
    let _ = state.update(open_action(
        &fresh,
        config(true, true, ResetRule::AddSearch(context)),
        ResetNotice::Toast,
    ));
    assert_eq!(
        toasts(&state.update(Action::Reset)),
        [(ToastKind::Warning, "Filter already at default")]
    );
    let mut sorted = fresh.clone();
    sorted.set_sort(CardSortKey::Name);
    edit(&mut state, &sorted);
    assert_eq!(
        toasts(&state.update(Action::Reset)),
        [(ToastKind::Info, "Filter reset")]
    );
    assert_eq!(state.draft(), Some(&fresh));

    let mut state = State::new();
    let _ = state.update(open_action(
        &CardQueryBuilder::default(),
        config(true, false, ResetRule::AddMaybeboard),
        ResetNotice::Toast,
    ));
    assert_eq!(
        toasts(&state.update(Action::Reset)),
        [(ToastKind::Warning, "Filter already at default")]
    );
}

#[test]
fn a_quiet_reset_stages_the_blank_filter_without_a_toast() {
    let mut state = State::new();
    let _ = state.update(open_action(
        &named("elf"),
        config(false, false, ResetRule::Plain),
        ResetNotice::Quiet,
    ));
    assert_eq!(state.update(Action::Reset), []);
    assert_eq!(state.draft(), Some(&CardQueryBuilder::default()));
    assert!(!state.unchanged());
}

#[test]
fn a_section_clear_unsets_only_that_section() {
    let mut both = named("elf");
    both.set_sort(CardSortKey::Name);
    let mut state = opened_over(&both);
    assert_eq!(state.active_sections(), [Section::Name, Section::Sort]);
    assert_eq!(state.update(Action::ClearSection(Section::Name)), []);
    assert_eq!(state.active_sections(), [Section::Sort]);
    assert_eq!(
        state.draft().and_then(CardQueryBuilder::name_contains),
        None
    );
}

#[test]
fn pickers_and_mode_chips_write_the_draft_in_the_mode_shown() {
    let mut state = opened_over(&CardQueryBuilder::default());
    let keywords = PickerField::Include(MatchGroup::Keywords);
    let _ = state.update(Action::FlipMode(ModeChip::Match(MatchGroup::Keywords)));
    assert_eq!(
        state
            .modes()
            .zip(state.draft())
            .map(|(modes, b)| modes.match_of(b, MatchGroup::Keywords)),
        Some(Match::All),
        "the chosen mode sticks while the field is empty"
    );
    let _ = state.update(Action::Pick {
        field: keywords,
        value: "flying".to_owned(),
    });
    assert_eq!(
        state
            .draft()
            .and_then(CardQueryBuilder::keywords_contains_all),
        Some(&["flying".to_owned()][..])
    );
    assert!(
        state
            .rows(Section::OracleText)
            .contains(&Row::Picker(keywords))
    );
    let _ = state.update(Action::Unpick {
        field: keywords,
        value: "flying".to_owned(),
    });
    assert_eq!(
        state
            .draft()
            .and_then(CardQueryBuilder::keywords_contains_all),
        None
    );
}

#[test]
fn number_boxes_write_or_say_why_not() {
    let mut state = opened_over(&CardQueryBuilder::default());
    let set = |field, text: &str| Action::SetNumber {
        field,
        text: text.to_owned(),
        other: String::new(),
    };
    assert_eq!(state.update(set(NumberField::ManaValue, "3")), []);
    assert_eq!(
        state.draft().and_then(CardQueryBuilder::cmc_equals),
        Some(3.0)
    );
    assert_eq!(
        state.update(set(NumberField::PriceMin, "abc")),
        [Effect::Toast(Toast::new(
            ToastKind::Error,
            "Invalid price",
            QUICK
        ))]
    );
}

#[test]
fn the_dictionary_feeds_the_list_it_was_opened_for() {
    let mut state = opened_over(&CardQueryBuilder::default());
    assert_eq!(
        state.update(Action::UseTag("ramp".to_owned())),
        [],
        "Use before a Dictionary chip has no list to feed"
    );
    let exclude = PickerField::Exclude(MatchGroup::Tags);
    assert_eq!(
        state.update(Action::OpenDictionary(exclude)),
        [Effect::Navigate(Destination::OracleTagDictionary)]
    );
    assert_eq!(state.dictionary(), Some(exclude));
    assert_eq!(
        state.update(Action::UseTag("ramp".to_owned())),
        [Effect::Toast(Toast::new(
            ToastKind::Success,
            "Added to filter",
            QUICK
        ))]
    );
    assert_eq!(
        state.update(Action::UseTag("ramp".to_owned())),
        [Effect::Toast(Toast::new(
            ToastKind::Info,
            "Already in filter",
            QUICK
        ))]
    );
    assert_eq!(
        state
            .draft()
            .and_then(CardQueryBuilder::oracle_tags_excludes),
        Some(&["ramp".to_owned()][..])
    );

    let _ = state.update(Action::OpenDictionary(PickerField::Include(
        MatchGroup::Tags,
    )));
    let _ = state.update(Action::UseTag("draw".to_owned()));
    assert_eq!(
        state
            .draft()
            .and_then(CardQueryBuilder::oracle_tags_contains_any),
        Some(&["draw".to_owned()][..])
    );
}

#[test]
fn a_closed_sheet_ignores_every_edit_and_button() {
    let mut state = opened_over(&named("elf"));
    let _ = state.update(Action::OpenDictionary(PickerField::Include(
        MatchGroup::Tags,
    )));
    let _ = state.update(Action::Back);
    assert_eq!(
        state.dictionary(),
        None,
        "closing drops the dictionary's list"
    );
    let closed = state.clone();
    for action in [
        Action::ClearSection(Section::Name),
        Action::FlipMode(ModeChip::Match(MatchGroup::Keywords)),
        Action::Pick {
            field: PickerField::Include(MatchGroup::Keywords),
            value: "flying".to_owned(),
        },
        Action::SetNumber {
            field: NumberField::PriceMin,
            text: "abc".to_owned(),
            other: String::new(),
        },
        Action::OpenDictionary(PickerField::Include(MatchGroup::Tags)),
        Action::UseTag("ramp".to_owned()),
        Action::Apply,
        Action::Reset,
        Action::Back,
    ] {
        assert_eq!(state.update(action.clone()), [], "{action:?}");
        assert_eq!(state, closed, "{action:?}");
    }
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Filter");
    assert_eq!(BACK, "Back");
    assert_eq!(APPLY, "Apply");
    assert_eq!(RESET, "Reset");
    assert_eq!(LISTS_FAILED, "Filter lists didn't load, trying again");
}
