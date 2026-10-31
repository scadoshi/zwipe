use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::{HINT_CONFIG, HINT_UNIVERSES_BEYOND_EXCEPTIONS};
use zwipe_core::domain::user::preferences::PipStyle;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::{render, screen_text, user};

fn ask(request: Request) -> screens::Request {
    request.into()
}

fn franchise(slug: &str, name: &str) -> UbFranchiseView {
    serde_json::from_value(serde_json::json!({"slug": slug, "name": name})).unwrap()
}

/// Home, then Config over it with the default preferences loaded and two franchises, the hints already seen, nothing left queued.
fn on_config() -> Harness {
    let mut harness = Harness::new();
    let mut known = user(true);
    for key in [HINT_CONFIG, HINT_UNIVERSES_BEYOND_EXCEPTIONS] {
        known.hints_shown.insert(key.to_owned(), true);
    }
    harness.app.shared.user = Some(known);
    harness.open(home::open);
    harness.press(KeyCode::Char('c'));
    harness.apply(Update::Preferences(Box::default()));
    harness.apply(Update::Franchises(vec![
        franchise("doctor_who", "Doctor Who"),
        franchise("warhammer_40k", "Warhammer 40,000"),
    ]));
    harness.clear();
    harness
}

fn view(harness: &Harness) -> &ConfigView {
    match harness.screen() {
        Screen::Config(view) => view,
        other => panic!("not on config: {other:?}"),
    }
}

fn focus(harness: &mut Harness, stop: Stop) {
    while view(harness).focused() != stop {
        harness.press(KeyCode::Char('j'));
    }
}

fn last_toast(harness: &Harness) -> Option<String> {
    harness
        .app
        .shared
        .toasts
        .items()
        .last()
        .map(|t| t.message.clone())
}

#[test]
fn opening_fetches_preferences_and_franchises_and_shows_the_config_hint() {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    assert_eq!(
        harness.pending(),
        [
            ask(Request::LoadPreferences),
            ask(Request::LoadFranchises),
            screens::Request::MarkHintShown(HINT_CONFIG.to_owned()),
        ]
    );
    assert!(
        !view(&harness).enabled(Stop::UniversesBeyond),
        "waits on the load"
    );
    assert!(
        view(&harness).enabled(Stop::Pips),
        "theme and pips show at once"
    );
    harness.apply(Update::PreferencesUnavailable);
    assert!(
        !view(&harness).loaded,
        "a failed load leaves them off for good"
    );
}

#[test]
fn the_screen_shows_the_apps_rows() {
    let harness = on_config();
    let screen = screen_text(&harness.app);
    for text in [
        "Config",
        "Appearance",
        "Theme ?",
        "Gruvbox",
        "☾ Dark",
        "Change",
        "Pips ?",
        "Universes Beyond ?",
        "Show",
        "Exceptions ?",
    ] {
        assert!(screen.contains(text), "{text}");
    }
}

#[test]
fn a_toggle_shows_at_once_and_toasts_the_servers_answer() {
    let mut harness = on_config();
    focus(&mut harness, Stop::Pips);
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).shown.pip_style, PipStyle::Natural);
    let Some(screens::Request::Config(Request::Save { seq, change, .. })) =
        harness.pending().first().cloned()
    else {
        panic!("no save");
    };
    let saved = UserPreferences {
        pip_style: PipStyle::Natural,
        ..UserPreferences::default()
    };
    harness.apply(Update::Saved {
        seq,
        change,
        preferences: Box::new(saved),
    });
    assert_eq!(
        last_toast(&harness).as_deref(),
        Some("Pips in natural colors")
    );
}

#[test]
fn a_failed_toggle_puts_back_the_value_from_before_that_tap_and_stale_answers_are_ignored() {
    let mut harness = on_config();
    focus(&mut harness, Stop::UniversesBeyond);
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Enter);
    let saves: Vec<(u64, PreferenceChange)> = harness
        .pending()
        .iter()
        .filter_map(|r| match r {
            screens::Request::Config(Request::Save { seq, undo, .. }) => Some((*seq, undo.clone())),
            _ => None,
        })
        .collect();
    assert_eq!(saves.len(), 2);
    assert!(!view(&harness).shown.exclude_universes_beyond, "two taps");
    harness.apply(Update::SaveFailed {
        seq: saves[0].0,
        undo: saves[0].1.clone(),
        message: "offline".to_owned(),
    });
    assert!(
        !view(&harness).shown.exclude_universes_beyond,
        "the older failure is ignored"
    );
    harness.apply(Update::SaveFailed {
        seq: saves[1].0,
        undo: saves[1].1.clone(),
        message: "offline".to_owned(),
    });
    assert!(
        view(&harness).shown.exclude_universes_beyond,
        "back to before the second tap"
    );
    assert_eq!(last_toast(&harness).as_deref(), Some("offline"));
}

#[test]
fn the_themes_sheet_previews_saves_and_restores() {
    let mut harness = on_config();
    let before = Palette::for_preferences(&UserPreferences::default());
    focus(&mut harness, Stop::Theme);
    harness.press(KeyCode::Enter);
    assert!(screen_text(&harness.app).contains("Warm & earthy"));
    harness.press(KeyCode::Char('j'));
    assert_ne!(harness.app.shared.palette, before, "a move previews");
    harness.press(KeyCode::Esc);
    assert_eq!(harness.app.shared.palette, before);
    assert_eq!(last_toast(&harness).as_deref(), Some("Theme unchanged"));

    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(view(&harness).sheet.is_none());
    assert!(matches!(
        harness.pending().last(),
        Some(screens::Request::Config(Request::Save {
            change: PreferenceChange::Theme { .. },
            ..
        }))
    ));
}

#[test]
fn exceptions_open_only_while_hidden_and_save_a_changed_set() {
    let mut harness = on_config();
    focus(&mut harness, Stop::Exceptions);
    harness.press(KeyCode::Enter);
    assert!(view(&harness).sheet.is_none(), "Universes Beyond is shown");
    if let Screen::Config(view) = harness.screen_mut() {
        view.shown.exclude_universes_beyond = true;
    }
    harness.press(KeyCode::Enter);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Doctor Who") && screen.contains(" ? "));
    harness.press(KeyCode::Char(' '));
    assert!(screen_text(&harness.app).contains("1 selected"));
    harness.press(KeyCode::Esc);
    assert_eq!(
        last_toast(&harness).as_deref(),
        Some("Exceptions unchanged")
    );
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Char('l'));
    harness.press(KeyCode::Char(' '));
    harness.press(KeyCode::Enter);
    assert!(matches!(
        harness.pending().last(),
        Some(screens::Request::Config(Request::Save { change: PreferenceChange::Exceptions(list), .. })) if list == &["warhammer_40k".to_owned()]
    ));
    assert!(
        view(&harness).shown.universes_beyond_exceptions.is_empty(),
        "not optimistic"
    );
}

#[test]
fn i_opens_the_rows_topic_and_the_screen_fits_a_tiny_terminal() {
    let mut harness = on_config();
    harness.press(KeyCode::Char('i'));
    assert!(screen_text(&harness.app).contains("The last four are color blind modes"));
    harness.press(KeyCode::Enter);
    for (width, height) in [(12, 5), (1, 1)] {
        render(&harness.app, width, height);
    }
    focus(&mut harness, Stop::Theme);
    harness.press(KeyCode::Enter);
    render(&harness.app, 12, 5);
}

#[test]
fn config_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&on_config().app, "Esc back");
}
