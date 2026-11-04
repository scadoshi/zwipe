use zwipe_core::http::contracts::changelog::HttpRelease;

use super::*;
use crate::changelog::EARLIER_LABEL;

/// No effects.
const NONE: [Effect; 0] = [];

fn release(version: &str, date: &str) -> HttpRelease {
    HttpRelease {
        version: version.to_owned(),
        date: date.to_owned(),
        released_on: None,
        entries: Vec::new(),
    }
}

fn history(upcoming: &[&str], releases: &[(&str, &str)]) -> HttpChangelog {
    HttpChangelog {
        upcoming: upcoming
            .iter()
            .map(|version| release(version, ""))
            .collect(),
        releases: releases
            .iter()
            .map(|(version, date)| release(version, date))
            .collect(),
    }
}

/// Dec 2026 into Jan 2027, with an upcoming release.
fn two_years() -> HttpChangelog {
    history(
        &["1.16.0"],
        &[
            ("1.15.1", "Jan 20, 2027"),
            ("1.15.0", "Jan 4, 2027"),
            ("1.14.2", "Dec 30, 2026"),
            ("1.14.1", "Dec 9, 2026"),
            ("1.14.0", "Nov 2, 2026"),
        ],
    )
}

fn year(year: &str) -> Filter {
    Filter::Year(year.to_owned())
}

fn month(year: &str, month: &str) -> Filter {
    Filter::Month {
        year: year.to_owned(),
        month: month.to_owned(),
    }
}

fn minor(year: &str, month: &str, minor: &str) -> Filter {
    Filter::Minor {
        year: year.to_owned(),
        month: month.to_owned(),
        minor: minor.to_owned(),
    }
}

fn labels(row: &[Chip]) -> Vec<&str> {
    row.iter().map(|chip| chip.label.as_str()).collect()
}

fn lit(rows: &[Vec<Chip>]) -> Vec<&str> {
    rows.iter()
        .flatten()
        .filter(|chip| chip.selected)
        .map(|chip| chip.label.as_str())
        .collect()
}

/// A screen loaded with `data` and narrowed to `filter`.
fn on(data: HttpChangelog, filter: Filter) -> State {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Load(ChangelogCache::Loaded(data))),
        NONE
    );
    assert_eq!(state.update(Action::Pick(filter)), NONE);
    state
}

fn shown(state: &State) -> Vec<&str> {
    state
        .shown()
        .into_iter()
        .map(|item| item.release.version.as_str())
        .collect()
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(TITLE, "About");
    assert_eq!(WEBSITE_LINK, "zwipe.net ↗");
    assert_eq!(VIEW, "View");
    assert_eq!(version_label("1.15.0"), "v1.15.0");
    let labels: Vec<&str> = ROWS.into_iter().map(Row::label).collect();
    assert_eq!(labels, ["Version", "Website", "Privacy policy"]);
    assert_eq!(CREDITS, "Credits");
    assert_eq!(CHANGELOG, "Changelog");
    assert_eq!(FAILED, "Could not load the changelog");
    assert_eq!(LATEST, "Latest");
    assert_eq!(BACK, "Back");
}

#[test]
fn a_new_screen_is_loading_on_latest() {
    let state = State::new();
    assert_eq!(state.status(), Status::Loading);
    assert_eq!(state.filter(), &Filter::Latest);
    assert!(state.changelog().is_none());
    assert_eq!(state.chip_rows(), Vec::<Vec<Chip>>::new());
    assert_eq!(state.shown().len(), 0);
}

#[test]
fn a_failed_fetch_shows_the_note() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Load(ChangelogCache::Failed)), NONE);
    assert_eq!(state.status(), Status::Failed);
    assert_eq!(state.chip_rows(), Vec::<Vec<Chip>>::new());
}

#[test]
fn the_fetch_landing_while_open_loads_the_history() {
    let mut state = State::new();
    let _ = state.update(Action::Load(ChangelogCache::Loading));
    let _ = state.update(Action::Load(ChangelogCache::Loaded(two_years())));
    assert_eq!(state.status(), Status::Loaded);
    assert_eq!(state.changelog(), Some(&two_years()));
}

#[test]
fn picking_before_the_history_loads_does_nothing() {
    let mut state = State::new();
    let _ = state.update(Action::Pick(year("2026")));
    assert_eq!(state.filter(), &Filter::Latest);
}

#[test]
fn loading_again_starts_on_latest() {
    let mut state = on(two_years(), year("2026"));
    let _ = state.update(Action::Load(ChangelogCache::Loaded(two_years())));
    assert_eq!(state.filter(), &Filter::Latest);
}

#[test]
fn the_rows_open_the_website_and_the_privacy_policy() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Website),
        [Effect::OpenUrl(WEB_BASE.to_owned())]
    );
    assert_eq!(
        state.update(Action::PrivacyPolicy),
        [Effect::Navigate(Destination::PrivacyPolicy)]
    );
}

#[test]
fn back_goes_to_the_screen_before() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        [Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn latest_shows_upcoming_and_the_newest_release() {
    let state = on(two_years(), Filter::Latest);
    let rows = state.chip_rows();
    assert_eq!(rows.len(), 1);
    assert_eq!(labels(&rows[0]), ["Latest", "2027", "2026"]);
    assert_eq!(lit(&rows), ["Latest"]);
    assert_eq!(shown(&state), ["1.16.0", "1.15.1"]);
}

#[test]
fn one_year_still_gets_a_year_chip() {
    let data = HttpChangelog::current();
    let upcoming = data.upcoming.len();
    let state = on(data, Filter::Latest);
    assert_eq!(labels(&state.chip_rows()[0]), ["Latest", "2026"]);
    assert_eq!(shown(&state).len(), upcoming + 1);
}

#[test]
fn a_year_adds_its_months_and_shows_the_year() {
    let state = on(two_years(), year("2026"));
    let rows = state.chip_rows();
    assert_eq!(rows.len(), 2);
    assert_eq!(labels(&rows[1]), ["Dec", "Nov"]);
    assert_eq!(lit(&rows), ["2026"]);
    assert_eq!(shown(&state), ["1.14.2", "1.14.1", "1.14.0"]);
}

#[test]
fn upcoming_counts_in_the_newest_month() {
    let state = on(two_years(), month("2027", "2027-01"));
    let rows = state.chip_rows();
    assert_eq!(rows.len(), 3);
    assert_eq!(labels(&rows[1]), ["Jan"]);
    assert_eq!(labels(&rows[2]), ["1.16", "1.15"]);
    assert_eq!(lit(&rows), ["2027", "Jan"]);
}

#[test]
fn a_minor_shows_its_patches_in_that_month() {
    let state = on(two_years(), minor("2026", "2026-12", "1.14"));
    let rows = state.chip_rows();
    assert_eq!(rows.len(), 3);
    assert_eq!(lit(&rows), ["2026", "Dec", "1.14"]);
    assert_eq!(shown(&state), ["1.14.2", "1.14.1"]);
    let november = on(two_years(), minor("2026", "2026-11", "1.14"));
    assert_eq!(shown(&november), ["1.14.0"]);
}

#[test]
fn higher_chips_clear_the_levels_below() {
    let rows = on(two_years(), minor("2026", "2026-12", "1.14")).chip_rows();
    assert_eq!(rows[0][0].target, Filter::Latest);
    assert_eq!(rows[0][1].target, year("2027"));
    assert_eq!(rows[0][2].target, year("2026"));
    assert_eq!(rows[1][1].target, month("2026", "2026-11"));
    assert_eq!(rows[1][0].target, month("2026", "2026-12"));
}

#[test]
fn the_deepest_lit_chip_goes_up_a_level() {
    let rows = on(two_years(), minor("2026", "2026-12", "1.14")).chip_rows();
    assert_eq!(rows[2][0].target, month("2026", "2026-12"));
    let rows = on(two_years(), month("2026", "2026-12")).chip_rows();
    assert_eq!(rows[1][0].target, year("2026"));
    let rows = on(two_years(), year("2026")).chip_rows();
    assert_eq!(rows[0][2].target, Filter::Latest);
    let rows = on(two_years(), Filter::Latest).chip_rows();
    assert_eq!(rows[0][0].target, Filter::Latest);
}

#[test]
fn pressing_a_chip_picks_its_target() {
    let mut state = on(two_years(), Filter::Latest);
    let target = state.chip_rows()[0][2].target.clone();
    let _ = state.update(Action::Pick(target));
    assert_eq!(state.filter(), &year("2026"));
    let target = state.chip_rows()[0][2].target.clone();
    let _ = state.update(Action::Pick(target));
    assert_eq!(state.filter(), &Filter::Latest, "the lit year goes back up");
}

#[test]
fn unreadable_dates_go_under_a_last_earlier_chip() {
    let data = history(
        &[],
        &[
            ("1.3.0", "Jan 3, 2027"),
            ("1.2.0", "someday"),
            ("1.1.0", "Sep 3, 2026"),
        ],
    );
    let earlier = on(data.clone(), Filter::Latest).chip_rows()[0][3]
        .target
        .clone();
    let state = on(data, earlier);
    assert_eq!(
        labels(&state.chip_rows()[0]),
        ["Latest", "2027", "2026", EARLIER_LABEL]
    );
    assert_eq!(shown(&state), ["1.2.0"]);
    let readable = on(two_years(), Filter::Latest).chip_rows();
    assert!(!labels(&readable[0]).contains(&EARLIER_LABEL));
}
