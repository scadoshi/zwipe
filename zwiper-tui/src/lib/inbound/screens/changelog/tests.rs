use crossterm::event::KeyCode;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{render, screen_text};

fn release(version: &str, date: &str, day: &str) -> HttpRelease {
    HttpRelease {
        version: version.to_owned(),
        date: date.to_owned(),
        released_on: chrono::NaiveDate::parse_from_str(day, "%Y-%m-%d").ok(),
        entries: vec![format!("Entry for {version}")],
    }
}

fn on_changelog(cache: ChangelogCache) -> Harness {
    let mut harness = Harness::new();
    harness.app.shared.changelog = cache;
    harness.open(open);
    harness
}

fn loaded() -> Harness {
    on_changelog(ChangelogCache::Loaded(HttpChangelog {
        upcoming: Vec::new(),
        releases: vec![
            release("1.15.1", "Oct 27, 2026", "2026-10-27"),
            release("1.15.0", "Oct 20, 2026", "2026-10-20"),
            release("1.14.0", "Sep 2, 2026", "2026-09-02"),
        ],
    }))
}

fn view(harness: &Harness) -> &ChangelogView {
    match harness.screen() {
        Screen::Changelog(view) => view,
        other => panic!("not on the changelog: {other:?}"),
    }
}

use crate::domain::changelog::{ChangelogCache, Filter};

#[test]
fn it_starts_on_latest_with_the_newest_release_tagged() {
    let harness = loaded();
    assert_eq!(view(&harness).filter, Filter::Latest);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("1.15.1") && screen.contains("Latest"));
    assert!(screen.contains("Entry for 1.15.1"));
    assert!(!screen.contains("1.14.0"));
    assert!(screen.contains("2026"));
}

#[test]
fn chips_narrow_to_a_year_month_and_minor_and_the_deepest_goes_up() {
    let mut harness = loaded();
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).filter, Filter::Year("2026".to_owned()));
    assert!(screen_text(&harness.app).contains("1.14.0"));
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(matches!(&view(&harness).filter, Filter::Month { month, .. } if month == "2026-10"));
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(matches!(&view(&harness).filter, Filter::Minor { minor, .. } if minor == "1.15"));
    harness.press(KeyCode::Enter);
    assert!(
        matches!(&view(&harness).filter, Filter::Month { .. }),
        "the deepest lit chip goes up a level"
    );
    harness.press(KeyCode::BackTab);
    harness.press(KeyCode::BackTab);
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).filter, Filter::Latest);
}

#[test]
fn loading_shows_a_skeleton_and_a_failure_the_note() {
    let loading = on_changelog(ChangelogCache::Loading);
    let screen = screen_text(&loading.app);
    assert!(screen.contains('▒') && !screen.contains("Latest"));
    let failed = on_changelog(ChangelogCache::Failed);
    assert!(screen_text(&failed.app).contains(FAILED));
}

#[test]
fn esc_goes_back_and_the_screen_fits_a_tiny_terminal() {
    let mut harness = loaded();
    for (width, height) in [(12, 5), (1, 1)] {
        render(&harness.app, width, height);
    }
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}
