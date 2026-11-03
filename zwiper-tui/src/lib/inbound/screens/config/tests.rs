use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::{HINT_CONFIG, HINT_UNIVERSES_BEYOND_EXCEPTIONS};
use zwipe_core::domain::user::preferences::PipColor;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::{render, screen_text, user};

fn franchise(slug: &str, name: &str) -> UbFranchiseView {
    serde_json::from_value(serde_json::json!({"slug": slug, "name": name})).unwrap()
}

/// The config jobs waiting, in order.
fn jobs(harness: &Harness) -> Vec<Job> {
    harness
        .pending()
        .iter()
        .filter_map(|request| match request {
            screens::Request::Config(Request::Visit { job, .. }) => Some(job.clone()),
            _ => None,
        })
        .collect()
}

/// The tickets on the saves waiting, in order.
fn saves(harness: &Harness) -> Vec<SaveTicket> {
    jobs(harness)
        .into_iter()
        .filter_map(|job| match job {
            Job::Save { ticket, .. } => Some(ticket),
            _ => None,
        })
        .collect()
}

/// `reply` for the visit on screen.
fn reply(harness: &Harness, reply: Reply) -> Update {
    Update::Visit {
        visit: view(harness).visit,
        reply,
    }
}

/// Home, then Config over it with `preferences` loaded and two franchises, the hints already seen, nothing left queued.
fn on_config_with(preferences: UserPreferences) -> Harness {
    let mut harness = Harness::new();
    let mut known = user(true);
    for key in [HINT_CONFIG, HINT_UNIVERSES_BEYOND_EXCEPTIONS] {
        known.hints_shown.insert(key.to_owned(), true);
    }
    harness.app.shared.user = Some(known);
    harness.open(home::open);
    harness.press(KeyCode::Char('c'));
    let Some(Job::LoadPreferences { ticket }) = jobs(&harness).first().cloned() else {
        panic!("no preferences fetch");
    };
    let loaded = reply(
        &harness,
        Reply::Preferences {
            ticket,
            preferences: Box::new(preferences),
        },
    );
    harness.apply(loaded);
    let franchises = reply(
        &harness,
        Reply::Franchises(vec![
            franchise("doctor_who", "Doctor Who"),
            franchise("warhammer_40k", "Warhammer 40,000"),
        ]),
    );
    harness.apply(franchises);
    harness.clear();
    harness
}

/// Config on the default preferences.
fn on_config() -> Harness {
    on_config_with(UserPreferences::default())
}

/// Config with Universes Beyond hidden, so Exceptions opens.
fn hiding() -> Harness {
    on_config_with(UserPreferences {
        exclude_universes_beyond: true,
        ..UserPreferences::default()
    })
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
    assert!(matches!(
        harness.pending(),
        [
            screens::Request::MarkHintShown(hint),
            screens::Request::Config(Request::Visit { job: Job::LoadPreferences { .. }, .. }),
            screens::Request::Config(Request::Visit { job: Job::LoadFranchises, .. }),
        ] if hint == HINT_CONFIG
    ));
    assert!(
        !view(&harness).enabled(Stop::UniversesBeyond),
        "waits on the load"
    );
    assert!(
        view(&harness).enabled(Stop::Pips),
        "theme and pips show at once"
    );
    let Some(Job::LoadPreferences { ticket }) = jobs(&harness).first().cloned() else {
        panic!("no preferences fetch");
    };
    let failed = reply(&harness, Reply::PreferencesUnavailable { ticket });
    harness.apply(failed);
    assert_eq!(
        view(&harness).model.status(),
        zwipe_ui::screens::config::Status::Failed,
        "a failed load leaves them off for good"
    );
}

#[test]
fn a_late_preferences_reply_changes_nothing() {
    let mut harness = on_config();
    let earlier = view(&harness).visit;
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('c'));
    let Some(Job::LoadPreferences { ticket }) = jobs(&harness).first().cloned() else {
        panic!("no preferences fetch");
    };
    harness.apply(Update::Visit {
        visit: earlier,
        reply: Reply::Preferences {
            ticket,
            preferences: Box::default(),
        },
    });
    assert!(
        !view(&harness).enabled(Stop::UniversesBeyond),
        "an earlier visit's reply landed"
    );
}

#[test]
fn the_screen_shows_the_apps_rows() {
    let harness = on_config();
    let screen = screen_text(&harness.app);
    for text in [
        "Appearance",
        "Theme [?]",
        "Gruvbox",
        "☾ Dark",
        "Change",
        "Pips [?]",
        "Universes Beyond [?]",
        "Show",
        "Exceptions [?]",
    ] {
        assert!(screen.contains(text), "{text}");
    }
}

#[test]
fn a_toggle_shows_at_once_and_toasts_the_servers_answer_once() {
    let mut harness = on_config();
    focus(&mut harness, Stop::Pips);
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).shown.pip_style, PipColor::Classic);
    let [ticket] = saves(&harness)[..] else {
        panic!("no save");
    };
    let saved = || UserPreferences {
        pip_style: PipColor::Classic,
        ..UserPreferences::default()
    };
    let landed = reply(
        &harness,
        Reply::Saved {
            ticket,
            preferences: Box::new(saved()),
        },
    );
    harness.apply(landed);
    assert_eq!(
        last_toast(&harness).as_deref(),
        Some("Pips in classic colors")
    );
    assert_eq!(
        harness.app.shared.preferences.as_ref().map(|p| p.pip_style),
        Some(PipColor::Classic),
        "kept for the next launch"
    );
    let repeated = reply(
        &harness,
        Reply::Saved {
            ticket,
            preferences: Box::new(saved()),
        },
    );
    harness.apply(repeated);
    assert_eq!(harness.app.shared.toasts.items().len(), 1);
}

#[test]
fn a_failed_toggle_puts_back_the_value_from_before_that_tap_and_stale_answers_are_ignored() {
    let mut harness = on_config();
    focus(&mut harness, Stop::UniversesBeyond);
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Enter);
    let tickets = saves(&harness);
    assert_eq!(tickets.len(), 2);
    assert!(!view(&harness).model.hides_universes_beyond(), "two taps");
    let failed = |harness: &Harness, ticket| {
        reply(
            harness,
            Reply::SaveFailed {
                ticket,
                message: "offline".to_owned(),
            },
        )
    };
    let older = failed(&harness, tickets[0]);
    harness.apply(older);
    assert!(
        !view(&harness).model.hides_universes_beyond(),
        "the older failure is ignored"
    );
    let newer = failed(&harness, tickets[1]);
    harness.apply(newer);
    assert!(
        view(&harness).model.hides_universes_beyond(),
        "back to before the second tap"
    );
    assert_eq!(last_toast(&harness).as_deref(), Some("offline"));
}

#[test]
fn a_failed_mode_tap_draws_the_mode_from_before_it() {
    let mut harness = on_config();
    let before = harness.app.shared.palette;
    harness.press(KeyCode::Enter);
    assert!(!view(&harness).shown.dark_mode);
    assert_ne!(harness.app.shared.palette, before, "shown at once");
    let [ticket] = saves(&harness)[..] else {
        panic!("no save");
    };
    let failed = reply(
        &harness,
        Reply::SaveFailed {
            ticket,
            message: "offline".to_owned(),
        },
    );
    harness.apply(failed);
    assert!(view(&harness).shown.dark_mode);
    assert_eq!(harness.app.shared.palette, before);
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
    assert_ne!(harness.app.shared.palette, before, "the pick stays");
    assert!(matches!(
        jobs(&harness).last(),
        Some(Job::Save {
            change: PreferenceChange::Theme { .. },
            ..
        })
    ));
}

#[test]
fn exceptions_open_only_while_hidden_and_save_a_changed_set() {
    let mut harness = on_config();
    focus(&mut harness, Stop::Exceptions);
    harness.press(KeyCode::Enter);
    assert!(view(&harness).sheet.is_none(), "Universes Beyond is shown");
    let mut harness = hiding();
    focus(&mut harness, Stop::Exceptions);
    harness.press(KeyCode::Enter);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Doctor Who") && screen.contains(" [?] "));
    harness.press(KeyCode::Enter);
    assert!(
        view(&harness).sheet.is_some(),
        "Save is off on the saved set"
    );
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
    assert!(view(&harness).sheet.is_none());
    assert!(matches!(
        jobs(&harness).last(),
        Some(Job::Save { change: PreferenceChange::Exceptions(list), .. }) if list == &["warhammer_40k".to_owned()]
    ));
    assert!(
        view(&harness).model.exceptions().is_empty(),
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

#[test]
fn the_themes_and_exceptions_sheets_read_save_as_the_app_does() {
    let mut harness = on_config();
    focus(&mut harness, Stop::Theme);
    harness.press(KeyCode::Enter);
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let text = crate::test_support::rows(&harness.app, width, height).join("\n");
        assert!(
            text.contains("Enter Save"),
            "Themes at {width}x{height}:\n{text}"
        );
        assert!(!text.contains("Enter -"), "Themes at {width}x{height}");
    }
    harness.press(KeyCode::Esc);
    let mut harness = hiding();
    focus(&mut harness, Stop::Exceptions);
    harness.press(KeyCode::Enter);
    let text = crate::test_support::rows(&harness.app, 80, 24).join("\n");
    assert!(text.contains("Enter Save"), "Exceptions:\n{text}");
}
