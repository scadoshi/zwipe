use chrono::NaiveDate;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::{
    HttpActivityDay, HttpRoleRates, HttpSwipeOutcomes, HttpTagRates,
};

use super::*;
use crate::activity::{NO_TAG_STANDS_OUT, NONE_REMOVED, PASSES_EMPTY, ROLES_EMPTY, TAGS_EMPTY};
use crate::test_support::card;

fn through() -> NaiveDate {
    NaiveDate::from_ymd_opt(2026, 10, 27).unwrap()
}

fn outcomes(added: u64, skipped: u64, maybed: u64, removed: u64) -> HttpSwipeOutcomes {
    HttpSwipeOutcomes {
        added,
        skipped,
        maybed,
        removed,
    }
}

fn role(slug: &str, mine: HttpSwipeOutcomes) -> HttpRoleRates {
    HttpRoleRates {
        role: slug.to_owned(),
        outcomes: mine,
    }
}

fn tag(
    slug: &str,
    role: &str,
    mine: HttpSwipeOutcomes,
    everyone: HttpSwipeOutcomes,
) -> HttpTagRates {
    HttpTagRates {
        tag: slug.to_owned(),
        role: role.to_owned(),
        outcomes: mine,
        everyone,
    }
}

fn pass(name: &str, skipped: u32) -> HttpHardPass {
    HttpHardPass {
        card: card(name),
        skipped,
        commanders: vec!["Atraxa".to_owned()],
    }
}

/// An account that has swiped nothing.
fn nothing() -> HttpActivity {
    HttpActivity {
        through: through(),
        days: Vec::new(),
        outcomes: HttpSwipeOutcomes::default(),
        roles: Vec::new(),
        tags: Vec::new(),
        passes: Vec::new(),
    }
}

/// A busy account: two days, every outcome, two roles, a standout tag under ramp and two passes.
fn busy() -> HttpActivity {
    HttpActivity {
        through: through(),
        days: vec![
            HttpActivityDay {
                day: through(),
                swipes: 12,
                searches: 3,
            },
            HttpActivityDay {
                day: NaiveDate::from_ymd_opt(2026, 10, 20).unwrap(),
                swipes: 1,
                searches: 0,
            },
        ],
        outcomes: outcomes(4, 5, 1, 1),
        roles: vec![
            role("ramp", outcomes(1, 2, 0, 0)),
            role("removal", outcomes(9, 1, 0, 0)),
        ],
        tags: vec![tag(
            "mana-rock",
            "ramp",
            outcomes(9, 1, 0, 0),
            outcomes(10, 90, 0, 0),
        )],
        passes: vec![pass("Sol Ring", 12), pass("Arcane Signet", 1)],
    }
}

/// A state whose fetch is out, with the ticket it holds.
fn loading() -> (State, Ticket) {
    let mut state = State::new();
    let effects = state.update(Action::Load);
    let [
        Effect::OfferHint(OneTimeHint::Activity),
        Effect::FetchActivity(ticket),
    ] = effects.as_slice()
    else {
        panic!("Load offers the hint and fetches: {effects:?}");
    };
    (state, *ticket)
}

/// A state with `activity` landed.
fn landed(activity: HttpActivity) -> State {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Loaded {
        ticket,
        activity: Box::new(activity),
    });
    assert_eq!(effects, [], "a reply asks for nothing");
    state
}

fn keys(rates: &Rates) -> Vec<String> {
    match rates {
        Rates::Charted(bars) => bars.iter().map(|b| b.key.clone()).collect(),
        Rates::Unrated(_) | Rates::Uncharted(_) => Vec::new(),
    }
}

#[test]
fn a_new_state_is_loading_on_added_with_nothing_revealed() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert!(!state.counts_ready(), "the counters roll while loading");
    assert_eq!(state.message(), None);
    assert_eq!(state.sections(), []);
    assert_eq!(state.activity(), None);
    assert_eq!(state.rate(), RateKind::Added);
    assert_eq!(state.revealed_role(), None);
    assert_eq!(state.revealed_tag(), None);
    assert_eq!(state.passes(), []);
    assert_eq!(state.heat(), None);
    assert_eq!(state.totals(), []);
}

#[test]
fn load_offers_the_hint_once_and_fetches_every_time() {
    let (mut state, first) = loading();
    let effects = state.update(Action::Load);
    let [Effect::FetchActivity(second)] = effects.as_slice() else {
        panic!("a second Load only fetches: {effects:?}");
    };
    assert_ne!(first, *second, "each fetch takes a fresh ticket");
    assert_eq!(state.status(), Status::Loading);
}

#[test]
fn a_reload_goes_back_to_loading_and_keeps_the_picks() {
    let mut state = landed(busy());
    let _ = state.update(Action::PickRate(RateKind::Skipped));
    let _ = state.update(Action::ToggleRole("ramp".to_owned()));
    let _ = state.update(Action::Load);
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.rate(), RateKind::Skipped);
    assert_eq!(state.revealed_role(), Some("ramp"));
}

#[test]
fn a_busy_account_loads_every_section() {
    let found = busy();
    let state = landed(found.clone());
    assert_eq!(state.status(), Status::Loaded);
    assert!(state.counts_ready());
    assert_eq!(state.message(), None);
    assert_eq!(state.activity(), Some(&found));
    assert_eq!(
        state.sections(),
        [Section::Activity, Section::Roles, Section::Tags]
    );
    assert!(state.shows_heatmap());
    assert!(state.shows_totals());
    assert_eq!(state.passes().len(), 2);
    assert_eq!(state.passes_line(), None);
}

#[test]
fn an_account_with_nothing_is_empty() {
    let state = landed(nothing());
    assert_eq!(state.status(), Status::Empty);
    assert!(state.counts_ready());
    assert_eq!(state.message(), Some(NOTHING_YET));
    assert_eq!(state.sections(), []);
    assert_eq!(state.activity(), None);
}

#[test]
fn a_failed_fetch_says_so() {
    let (mut state, ticket) = loading();
    let effects = state.update(Action::Failed { ticket });
    assert_eq!(effects, [], "the client's runner raises the error toast");
    assert_eq!(state.status(), Status::Failed);
    assert!(
        state.counts_ready(),
        "the counters resolve on a failure too"
    );
    assert_eq!(state.message(), Some(LOAD_FAILED));
    assert_eq!(state.sections(), []);
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, old) = loading();
    let _ = state.update(Action::Load);
    let _ = state.update(Action::Loaded {
        ticket: old,
        activity: Box::new(busy()),
    });
    assert_eq!(state.status(), Status::Loading, "a stale success landed");
    let _ = state.update(Action::Failed { ticket: old });
    assert_eq!(state.status(), Status::Loading, "a stale failure landed");
}

#[test]
fn a_late_reply_never_overwrites_newer_data() {
    let (mut state, old) = loading();
    let effects = state.update(Action::Load);
    let [Effect::FetchActivity(fresh)] = effects.as_slice() else {
        panic!("{effects:?}");
    };
    let found = busy();
    let _ = state.update(Action::Loaded {
        ticket: *fresh,
        activity: Box::new(found.clone()),
    });
    let _ = state.update(Action::Failed { ticket: old });
    assert_eq!(state.status(), Status::Loaded);
    let _ = state.update(Action::Loaded {
        ticket: old,
        activity: Box::new(nothing()),
    });
    assert_eq!(state.activity(), Some(&found));
}

#[test]
fn activity_is_left_out_with_no_day_and_no_outcome() {
    let mut rates_only = nothing();
    rates_only.roles = busy().roles;
    let state = landed(rates_only);
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.sections(), [Section::Roles, Section::Tags]);
    assert!(!state.shows_heatmap());
    assert!(!state.shows_totals());
    assert_eq!(state.heat(), None);
    assert_eq!(state.totals(), []);
}

#[test]
fn activity_shows_with_only_days_or_only_outcomes() {
    let mut days_only = nothing();
    days_only.days = busy().days;
    let state = landed(days_only);
    assert_eq!(state.sections().first(), Some(&Section::Activity));
    assert!(state.shows_heatmap());
    assert!(!state.shows_totals(), "no outcome, no Totals");
    assert_eq!(state.totals(), []);

    let mut outcomes_only = nothing();
    outcomes_only.outcomes = outcomes(0, 3, 0, 0);
    let state = landed(outcomes_only);
    assert_eq!(state.sections().first(), Some(&Section::Activity));
    assert!(!state.shows_heatmap(), "no day, no heatmap");
    assert_eq!(state.heat(), None);
    assert!(state.shows_totals());
}

#[test]
fn the_year_reads_from_the_days() {
    let state = landed(busy());
    let heat = state.heat().unwrap();
    assert_eq!(heat, activity::heat_layout(through(), &busy().days));
    assert_eq!(state.year_tags(), ["13 swipes", "3 searches", "Last year"]);
    assert_eq!(
        state.year_line(),
        "13 swipes and 3 searches in the last year"
    );
    assert_eq!(
        state.totals(),
        activity::outcome_bars(&busy().outcomes),
        "Totals is the four outcomes"
    );
}

#[test]
fn the_year_reads_as_nothing_before_it_loads() {
    let state = State::new();
    assert_eq!(state.year_tags(), ["0 swipes", "0 searches", "Last year"]);
    assert_eq!(
        state.year_line(),
        "0 swipes and 0 searches in the last year"
    );
}

#[test]
fn empty_rate_sections_say_what_fills_them() {
    let mut days_only = nothing();
    days_only.days = busy().days;
    let state = landed(days_only);
    assert_eq!(state.role_rates(), Rates::Unrated(ROLES_EMPTY));
    assert_eq!(state.tag_rates(), Rates::Unrated(TAGS_EMPTY));
    assert_eq!(state.passes_line(), Some(PASSES_EMPTY));
    assert_eq!(State::new().role_rates(), Rates::Unrated(ROLES_EMPTY));
}

#[test]
fn the_rate_charts_follow_the_pick() {
    let mut state = landed(busy());
    assert_eq!(keys(&state.role_rates()), ["removal", "ramp"], "Added");
    assert_eq!(keys(&state.tag_rates()), ["mana-rock"], "Added");
    let _ = state.update(Action::PickRate(RateKind::Skipped));
    assert_eq!(state.rate(), RateKind::Skipped);
    assert_eq!(keys(&state.role_rates()), ["ramp", "removal"], "Skipped");
    assert_eq!(
        state.tag_rates(),
        Rates::Uncharted(NO_TAG_STANDS_OUT),
        "the user skips mana rocks less than others"
    );
}

#[test]
fn removed_charts_no_role_without_an_added_card() {
    let mut never_added = nothing();
    never_added.roles = vec![role("ramp", outcomes(0, 4, 0, 0))];
    never_added.tags = vec![tag(
        "mana-rock",
        "ramp",
        outcomes(0, 4, 0, 0),
        outcomes(5, 5, 0, 0),
    )];
    let mut state = landed(never_added);
    let _ = state.update(Action::PickRate(RateKind::Removed));
    assert_eq!(state.role_rates(), Rates::Uncharted(NONE_REMOVED));
    assert_eq!(state.tag_rates(), Rates::Uncharted(NONE_REMOVED));
}

#[test]
fn a_role_reveals_its_swiped_tags_and_hides_them_again() {
    let mut state = landed(busy());
    assert_eq!(state.tags_under("ramp"), ["mana-rock"]);
    assert_eq!(state.tags_under("removal"), Vec::<String>::new());
    let _ = state.update(Action::ToggleRole("ramp".to_owned()));
    assert_eq!(state.revealed_role(), Some("ramp"));
    let _ = state.update(Action::ToggleRole("removal".to_owned()));
    assert_eq!(state.revealed_role(), Some("removal"), "one role at a time");
    let _ = state.update(Action::ToggleRole("removal".to_owned()));
    assert_eq!(state.revealed_role(), None);
}

#[test]
fn a_tag_reveal_is_independent_of_the_role_reveal() {
    let mut state = landed(busy());
    let _ = state.update(Action::ToggleRole("ramp".to_owned()));
    let _ = state.update(Action::ToggleTag("mana-rock".to_owned()));
    assert_eq!(state.revealed_role(), Some("ramp"));
    assert_eq!(state.revealed_tag(), Some("mana-rock"));
    let _ = state.update(Action::PickRate(RateKind::Maybed));
    assert_eq!(state.revealed_tag(), Some("mana-rock"), "a pick keeps it");
    let _ = state.update(Action::ToggleTag("mana-rock".to_owned()));
    assert_eq!(state.revealed_tag(), None);
    assert_eq!(state.revealed_role(), Some("ramp"));
}

#[test]
fn a_picked_printing_replaces_its_pass() {
    let found = busy();
    let mut state = landed(found.clone());
    let mut printing = card("Sol Ring");
    printing.scryfall_data.id = Uuid::new_v4();
    printing.scryfall_data.oracle_id = state.passes()[0].card.scryfall_data.oracle_id;
    let effects = state.update(Action::PickPrinting(Box::new(printing.clone())));
    assert_eq!(effects, []);
    assert_eq!(state.passes()[0].card, printing);
    assert_eq!(state.passes()[0].skipped, 12, "the counts stay");
    assert_eq!(state.passes()[1], found.passes[1]);
}

#[test]
fn a_printing_with_no_match_changes_nothing() {
    let found = busy();
    let mut state = landed(found.clone());
    let _ = state.update(Action::PickPrinting(Box::new(card("Stranger"))));
    let mut no_oracle = card("Sol Ring");
    no_oracle.scryfall_data.oracle_id = None;
    let _ = state.update(Action::PickPrinting(Box::new(no_oracle)));
    assert_eq!(state.activity(), Some(&found));
}

#[test]
fn back_goes_to_the_screen_before() {
    let mut state = landed(busy());
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
    assert_eq!(state.status(), Status::Loaded);
}

#[test]
fn a_pass_tags_its_skips() {
    assert_eq!(skips(&pass("Sol Ring", 1)), "1 skip");
    assert_eq!(skips(&pass("Sol Ring", 1200)), "1,200 skips");
}

#[test]
fn only_activity_opens_on_entry() {
    let open: Vec<Section> = Section::ALL
        .into_iter()
        .filter(|s| s.opens_on_entry())
        .collect();
    assert_eq!(open, [Section::Activity]);
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "Activity");
    assert_eq!(LOAD_FAILED, "Couldn't load your activity");
    assert_eq!(NOTHING_YET, "Swipe some cards to see your activity");
    assert_eq!(TOTALS, "Totals");
    assert_eq!(SHOW, "Show");
    assert_eq!(PASSES_TITLE, "Most skipped");
    assert_eq!(SKIPPED_UNDER, "Skipped under");
    assert_eq!(BACK, "Back");
    let titles: Vec<&str> = Section::ALL.into_iter().map(Section::title).collect();
    assert_eq!(titles, ["Activity", "Card roles", "Oracle tags"]);
}
