use crossterm::event::KeyCode;
use zwipe_core::domain::user::preferences::PipStyle;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::screen_text;

fn ask(request: Request) -> screens::Request {
    request.into()
}

/// Home, then the config screen over it with the default preferences loaded, with nothing left queued.
fn on_config() -> Harness {
    let mut harness = Harness::new();
    harness.open(home::open);
    harness.press(KeyCode::Char('c'));
    assert_eq!(
        harness.pending().last(),
        Some(&ask(Request::LoadPreferences))
    );
    harness.apply(Update::Preferences(Box::default()));
    harness.clear();
    harness
}

fn config_view(harness: &Harness) -> &ConfigView {
    match harness.screen() {
        Screen::Config(view) => view,
        other => panic!("not on the config screen: {other:?}"),
    }
}

#[test]
fn loaded_preferences_set_the_theme_and_fill_the_screen() {
    let harness = on_config();
    assert!(config_view(&harness).shown.is_some());
    assert_eq!(
        harness.app.shared.palette,
        Palette::for_account("gruvbox", true)
    );
}

#[test]
fn a_toggle_shows_at_once_and_a_failure_puts_it_back() {
    let mut harness = on_config();
    for _ in 0..2 {
        harness.press(KeyCode::Char('j'));
    }
    let before = harness.app.shared.palette;
    harness.press(KeyCode::Enter);
    assert_eq!(
        config_view(&harness).shown.as_ref().map(|p| p.pip_style),
        Some(PipStyle::Natural)
    );
    assert_ne!(
        harness.app.shared.palette, before,
        "natural pips change the mana colors at once"
    );
    assert!(matches!(
        harness.pending(),
        [screens::Request::Config(Request::SavePreferences {
            seq: 1,
            ..
        })]
    ));

    harness.apply(Update::SaveFailed {
        seq: 1,
        message: "offline".to_owned(),
    });
    assert_eq!(harness.app.shared.palette, before);
    assert_eq!(harness.app.shared.status, "offline");
}

#[test]
fn the_theme_list_previews_and_esc_puts_the_theme_back() {
    let mut harness = on_config();
    let before = harness.app.shared.palette;
    harness.press(KeyCode::Enter);
    assert!(config_view(&harness).picker.is_some());
    harness.press(KeyCode::Char('j'));
    assert_ne!(
        harness.app.shared.palette, before,
        "the next theme previews"
    );
    harness.press(KeyCode::Esc);
    assert_eq!(harness.app.shared.palette, before);
    assert_eq!(harness.pending(), []);
    assert_eq!(harness.app.shared.status, "Theme unchanged");

    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(config_view(&harness).picker.is_none());
    assert!(matches!(
        harness.pending(),
        [screens::Request::Config(Request::SavePreferences {
            change: PreferenceChange::Theme { .. },
            ..
        })]
    ));
}

#[test]
fn exceptions_stay_shut_while_universes_beyond_is_shown() {
    let mut harness = on_config();
    for _ in 0..4 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(config_view(&harness).exceptions.is_none());
    assert_eq!(harness.pending(), []);

    harness.press(KeyCode::Char('k'));
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Char('j'));
    harness.clear();
    harness.press(KeyCode::Enter);
    assert!(config_view(&harness).exceptions.is_some());
    assert_eq!(harness.pending(), [ask(Request::LoadFranchises)]);
}

#[test]
fn the_config_screen_lines_up_every_row_and_marks_the_selection() {
    let harness = Harness::on(Screen::Config(Box::new(ConfigView {
        saved: Some(UserPreferences::default()),
        shown: Some(UserPreferences::default()),
        ..ConfigView::default()
    })));
    let screen = screen_text(&harness.app);
    for expected in [
        "> Theme",
        "Gruvbox",
        "  Mode",
        "Dark",
        "  Pips",
        "  Universes Beyond",
        "Show",
        "  Exceptions",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}");
    }
}

#[test]
fn the_theme_list_shows_sections_and_names() {
    let harness = Harness::on(Screen::Config(Box::new(ConfigView {
        saved: Some(UserPreferences::default()),
        shown: Some(UserPreferences::default()),
        picker: Some(ThemePicker::opening("gruvbox")),
        ..ConfigView::default()
    })));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Themes"));
    assert!(screen.contains("Warm & earthy"));
    assert!(screen.contains("> Gruvbox"));
}

fn loaded() -> ConfigView {
    ConfigView {
        saved: Some(UserPreferences::default()),
        shown: Some(UserPreferences::default()),
        ..ConfigView::default()
    }
}

#[test]
fn a_change_shows_at_once_and_a_failure_puts_it_back() {
    let mut view = loaded();
    let seq = view
        .begin(&PreferenceChange::Pips(PipStyle::Natural))
        .unwrap();
    assert_eq!(view.shown.as_ref().unwrap().pip_style, PipStyle::Natural);
    assert!(view.revert(seq));
    assert_eq!(view.shown.as_ref().unwrap().pip_style, PipStyle::Theme);
}

#[test]
fn only_the_latest_change_is_answered() {
    let mut view = loaded();
    let first = view
        .begin(&PreferenceChange::HideUniversesBeyond(true))
        .unwrap();
    let second = view
        .begin(&PreferenceChange::HideUniversesBeyond(false))
        .unwrap();
    assert!(
        !view.revert(first),
        "a late failure of an older change is ignored"
    );
    assert!(!view.shown.as_ref().unwrap().exclude_universes_beyond);
    let answer = UserPreferences::default();
    assert!(view.confirm(second, answer));
}

#[test]
fn nothing_changes_before_the_preferences_load() {
    assert_eq!(
        ConfigView::default().begin(&PreferenceChange::Pips(PipStyle::Natural)),
        None
    );
    assert!(!ConfigView::default().exceptions_open());
}

#[test]
fn the_picker_opens_on_the_current_theme_and_skips_headers() {
    let mut picker = ThemePicker::opening("gruvbox");
    assert_eq!(picker.theme(), Some("gruvbox"));
    let renamed = ThemePicker::opening("ristretto");
    assert_eq!(renamed.theme(), Some("monokai-pro-ristretto"));
    for _ in 0..200 {
        picker.down();
    }
    assert!(picker.theme().is_some());
    for _ in 0..200 {
        picker.up();
    }
    assert_eq!(
        picker.theme(),
        Some("darcula"),
        "the first theme, past the first header"
    );
}

#[test]
fn exceptions_toggle_in_pick_order_and_compare_as_a_set() {
    let franchise = |slug: &str| UbFranchiseView {
        slug: slug.to_owned(),
        name: slug.to_uppercase(),
    };
    let mut editor = ExceptionsEditor::opening(&["b".to_owned()]);
    editor.franchises = Some(vec![franchise("a"), franchise("b")]);
    editor.toggle();
    assert_eq!(editor.draft, ["b", "a"]);
    assert!(editor.changed());
    editor.down();
    editor.toggle();
    editor.toggle();
    assert_eq!(editor.draft, ["a", "b"]);
    editor.up();
    editor.toggle();
    assert!(!editor.changed(), "back to just b");
}
