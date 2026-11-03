use crossterm::event::KeyCode;
use ratatui::style::Modifier;
use zwipe_core::domain::user::Username;
use zwipe_core::domain::user::hints::HINT_PROFILE;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::{render, screen_text, user};

/// Home with the user known (the profile hint already seen), then Account over it, with nothing left queued.
fn on_account(verified: bool) -> Harness {
    let mut harness = Harness::new();
    let mut known = user(verified);
    known.hints_shown.insert(HINT_PROFILE.to_owned(), true);
    harness.app.shared.user = Some(known);
    harness.open(home::open);
    harness.clear();
    harness.press(KeyCode::Char('a'));
    harness.clear();
    harness
}

fn view(harness: &Harness) -> &AccountView {
    match harness.screen() {
        Screen::Account(view) => view,
        other => panic!("not on the account screen: {other:?}"),
    }
}

/// The account requests waiting, in order.
fn jobs(harness: &Harness) -> Vec<Request> {
    harness
        .pending()
        .iter()
        .filter_map(|request| match request {
            screens::Request::Account(request) => Some(request.clone()),
            _ => None,
        })
        .collect()
}

/// The ticket on the last account job waiting.
fn last_ticket(harness: &Harness) -> Ticket {
    match jobs(harness).last() {
        Some(
            Request::FetchUser { ticket }
            | Request::CheckVerification { ticket }
            | Request::Resend { ticket }
            | Request::Wait { ticket }
            | Request::Change { ticket, .. },
        ) => *ticket,
        None => panic!("no account job went out"),
    }
}

fn toasts(harness: &Harness) -> Vec<(ToastKind, String)> {
    harness
        .app
        .shared
        .toasts
        .items()
        .iter()
        .map(|t| (t.kind, t.message.clone()))
        .collect()
}

/// Opens the focused row's sheet and types `values` into its fields in order, ending on the last.
fn fill(harness: &mut Harness, values: &[&str]) {
    harness.press(KeyCode::Enter);
    for (at, value) in values.iter().enumerate() {
        if at > 0 {
            harness.press(KeyCode::Tab);
        }
        harness.type_text(value);
    }
}

/// Focuses `stop`.
fn focus(harness: &mut Harness, stop: Stop) {
    while view(harness).focused() != Some(stop) {
        harness.press(KeyCode::Char('j'));
    }
}

#[test]
fn opening_fetches_the_user_and_shows_the_profile_hint_the_first_time() {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    assert!(matches!(
        harness.pending(),
        [
            screens::Request::MarkHintShown(hint),
            screens::Request::Account(Request::FetchUser { .. }),
        ] if hint == HINT_PROFILE
    ));
    assert!(screen_text(&harness.app).contains("Your account"));
}

#[test]
fn the_screen_shows_its_rows_and_the_red_unverified_badge() {
    let harness = on_account(false);
    let screen = screen_text(&harness.app);
    for text in [
        "Account",
        "Username",
        "scotty",
        "Email [?]",
        "Unverified",
        "Verification [?]",
        "Resend",
        "•••••••",
        "About",
        "zwipe.net ↗",
        "Privacy policy",
        "View",
        "Changelog",
        "Log out",
        "More",
    ] {
        assert!(screen.contains(text), "{text}");
    }
    assert!(!screen_text(&on_account(true).app).contains("Verification"));
}

#[test]
fn a_fresh_user_lands_once_and_a_late_one_changes_nothing() {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(false));
    harness.open(open);
    let ticket = last_ticket(&harness);
    let fresh = Update::UserLoaded {
        ticket,
        user: Box::new(user(true)),
    };
    harness.apply(fresh);
    assert!(view(&harness).model.is_verified());
    let stale = Update::UserLoaded {
        ticket,
        user: Box::new(user(false)),
    };
    harness.apply(stale);
    assert!(
        view(&harness).model.is_verified(),
        "a repeated reply landed"
    );
    assert_eq!(toasts(&harness), []);
}

#[test]
fn a_reply_to_an_earlier_visit_never_lands_in_a_later_one() {
    let mut harness = on_account(false);
    let earlier = harness
        .sent()
        .rev()
        .find_map(|r| match r {
            screens::Request::Account(Request::FetchUser { ticket }) => Some(*ticket),
            _ => None,
        })
        .unwrap();
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('a'));
    assert!(matches!(harness.screen(), Screen::Account(_)));
    assert_ne!(
        last_ticket(&harness),
        earlier,
        "each visit's fetch holds its own ticket"
    );
    harness.apply(Update::UserLoaded {
        ticket: earlier,
        user: Box::new(user(true)),
    });
    assert!(!view(&harness).model.is_verified());
}

#[test]
fn a_bad_username_shows_live_and_a_good_one_is_sent_as_typed() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Enter);
    harness.type_text("ab");
    let Some(Overlay::Form(form)) = &view(&harness).overlay else {
        panic!("no sheet");
    };
    assert_eq!(
        form.inputs[0].error(),
        Some("Must be at least 3 characters long")
    );
    harness.type_text("cdef");
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Enter);
    assert_eq!(jobs(&harness), [], "the password is required");
    harness.type_text("hunter2");
    harness.press(KeyCode::Enter);
    assert!(matches!(
        jobs(&harness).as_slice(),
        [Request::Change {
            change: Change::Username { username, password },
            ..
        }] if username == "abcdef" && password.read() == "hunter2"
    ));
    assert!(screen_text(&harness.app).contains("Saving..."));
    harness.press(KeyCode::Esc);
    assert!(
        matches!(view(&harness).overlay, Some(Overlay::Form(_))),
        "the sheet takes no keys while its change is out"
    );
}

#[test]
fn a_change_closes_the_sheet_and_toasts_for_its_band_once() {
    let mut harness = on_account(true);
    fill(&mut harness, &["abcdef", "hunter2"]);
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    let mut renamed = user(true);
    renamed.username = Username::new("abcdef").unwrap();
    let changed = Update::UsernameChanged {
        ticket,
        user: Box::new(renamed.clone()),
    };
    harness.apply(changed);
    assert!(view(&harness).overlay.is_none());
    assert_eq!(
        view(&harness).model.user().map(|u| u.username.to_string()),
        Some("abcdef".to_owned())
    );
    assert_eq!(harness.app.shared.username.as_deref(), Some("abcdef"));
    let up = &harness.app.shared.toasts.items()[0];
    assert_eq!(up.message, "Username changed to abcdef");
    assert_eq!(up.expires_at.duration_since(up.shown_at), toast::QUICK);
    let repeated = Update::UsernameChanged {
        ticket,
        user: Box::new(renamed),
    };
    harness.apply(repeated);
    assert_eq!(toasts(&harness).len(), 1, "a repeated reply toasted again");
}

#[test]
fn a_failed_change_toasts_and_keeps_the_sheet() {
    let mut harness = on_account(true);
    fill(&mut harness, &["abcdef", "hunter2"]);
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    let failed = Update::Failed {
        ticket,
        error: "wrong password".to_owned(),
    };
    harness.apply(failed);
    assert!(matches!(&view(&harness).overlay, Some(Overlay::Form(_))));
    assert!(!view(&harness).model.is_saving(Sheet::Username));
    assert_eq!(
        toasts(&harness),
        [(ToastKind::Error, "wrong password".to_owned())]
    );
}

#[test]
fn a_password_change_asks_first_and_checks_against_the_current_one() {
    let mut harness = on_account(true);
    focus(&mut harness, Stop::ChangePassword);
    fill(&mut harness, &["Same!pass1", "Same!pass1", "Same!pass1"]);
    harness.press(KeyCode::Enter);
    let Some(Overlay::Form(form)) = &view(&harness).overlay else {
        panic!("no sheet");
    };
    assert_eq!(
        form.inputs[1].error(),
        Some("New password must be different from your current password")
    );
    harness.press(KeyCode::Esc);
    fill(&mut harness, &["Old!pass1", "New!pass22", "New!pass22"]);
    harness.press(KeyCode::Enter);
    assert!(matches!(
        view(&harness).overlay,
        Some(Overlay::ConfirmPassword { .. })
    ));
    assert!(screen_text(&harness.app).contains("Changing your password will log you out"));
    harness.press(KeyCode::Char('y'));
    assert!(matches!(
        jobs(&harness).as_slice(),
        [Request::Change {
            change: Change::Password { current, new },
            ..
        }] if current.read() == "Old!pass1" && new.read() == "New!pass22"
    ));
    assert!(matches!(view(&harness).overlay, Some(Overlay::Form(_))));
}

#[test]
fn resend_counts_down_a_minute_after_it_lands_unless_it_failed() {
    let mut harness = on_account(false);
    focus(&mut harness, Stop::Resend);
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).model.resend_label(), "Sending...");
    harness.press(KeyCode::Enter);
    assert_eq!(jobs(&harness).len(), 1, "once while sending");
    let ticket = last_ticket(&harness);
    let resent = Update::Resent { ticket };
    harness.apply(resent);
    assert_eq!(view(&harness).model.resend_label(), "Resend in 60s");
    assert_eq!(
        toasts(&harness),
        [(ToastKind::Success, "Verification email sent".to_owned())]
    );
    let wait = last_ticket(&harness);
    let tick = Update::Tick { ticket: wait };
    harness.apply(tick);
    assert_eq!(view(&harness).model.resend_label(), "Resend in 59s");
    assert!(matches!(jobs(&harness).last(), Some(Request::Wait { .. })));
}

#[test]
fn a_failed_resend_toasts_long_and_frees_the_button() {
    let mut harness = on_account(false);
    focus(&mut harness, Stop::Resend);
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    let failed = Update::ResendFailed {
        ticket,
        error: "offline".to_owned(),
    };
    harness.apply(failed);
    assert_eq!(view(&harness).model.resend_label(), "Resend");
    let up = &harness.app.shared.toasts.items()[0];
    assert_eq!(up.expires_at.duration_since(up.shown_at), toast::LONG);
}

#[test]
fn checking_verification_toasts_either_way() {
    let mut harness = on_account(false);
    focus(&mut harness, Stop::Check);
    harness.press(KeyCode::Enter);
    assert!(view(&harness).model.is_checking());
    let ticket = last_ticket(&harness);
    let checked = Update::UserLoaded {
        ticket,
        user: Box::new(user(true)),
    };
    harness.apply(checked);
    assert!(view(&harness).model.is_verified() && !view(&harness).model.is_checking());
    assert_eq!(
        toasts(&harness),
        [(ToastKind::Success, "Email verified".to_owned())]
    );
}

#[test]
fn log_out_asks_then_signs_out_with_no_toast() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Char('L'));
    assert!(screen_text(&harness.app).contains("Are you sure you want to log out?"));
    harness.press(KeyCode::Char('n'));
    assert!(view(&harness).overlay.is_none());
    harness.press(KeyCode::Char('L'));
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), [screens::Request::SignOut]);
    assert_eq!(toasts(&harness), []);
}

#[test]
fn more_opens_delete_which_counts_down_before_it_sends() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Char('m'));
    assert!(screen_text(&harness.app).contains("Delete account"));
    harness.press(KeyCode::Enter);
    let screen = screen_text(&harness.app);
    assert!(screen.contains("This will permanently delete your account"));
    assert!(screen.contains("Delete (5)"));
    harness.type_text("pw");
    harness.press(KeyCode::Enter);
    assert!(
        matches!(jobs(&harness).as_slice(), [Request::Wait { .. }]),
        "still counting down"
    );
    for _ in 0..5 {
        let wait = last_ticket(&harness);
        let tick = Update::Tick { ticket: wait };
        harness.apply(tick);
    }
    assert!(screen_text(&harness.app).contains("Delete "));
    harness.clear();
    harness.press(KeyCode::Enter);
    assert!(matches!(
        jobs(&harness).as_slice(),
        [Request::Change {
            change: Change::Delete { password },
            ..
        }] if password.read() == "pw"
    ));
    assert!(screen_text(&harness.app).contains("Deleting..."));
    let ticket = last_ticket(&harness);
    let failed = Update::Failed {
        ticket,
        error: "wrong password".to_owned(),
    };
    harness.apply(failed);
    assert!(!view(&harness).model.is_deleting());
    harness.clear();
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    let deleted = Update::Deleted { ticket };
    harness.clear();
    harness.apply(deleted);
    assert_eq!(harness.pending(), [screens::Request::SignOut]);
}

#[test]
fn esc_on_delete_stops_its_countdown() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Enter);
    let wait = last_ticket(&harness);
    harness.press(KeyCode::Esc);
    assert!(matches!(view(&harness).overlay, Some(Overlay::More(None))));
    let tick = Update::Tick { ticket: wait };
    harness.clear();
    harness.apply(tick);
    assert_eq!(
        jobs(&harness),
        [],
        "a closed dialog's tick asked for another"
    );
}

#[test]
fn i_opens_the_rows_topic_and_the_about_rows_open_their_screens() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('i'));
    assert!(screen_text(&harness.app).contains("Where password resets and verification go"));
    harness.press(KeyCode::Enter);
    for _ in 0..3 {
        harness.press(KeyCode::Char('j'));
    }
    assert_eq!(view(&harness).focused(), Some(Stop::Privacy));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Privacy(_)));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Changelog(_)));
}

#[test]
fn the_screen_and_its_sheets_fit_a_tiny_terminal() {
    let mut harness = on_account(false);
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Enter);
    render(&harness.app, 12, 5);
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('m'));
    harness.press(KeyCode::Enter);
    render(&harness.app, 12, 5);
    render(&harness.app, 1, 1);
}

#[test]
fn account_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&on_account(false).app, "Esc back");
}

#[test]
fn a_focused_row_button_lines_up_with_the_row_buttons_under_it() {
    let mut harness = on_account(true);
    focus(&mut harness, Stop::Privacy);
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let terminal = crate::test_support::render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let row_of = |label: &str| {
            (0..height)
                .find(|&y| {
                    (0..width)
                        .map(|x| buffer[(x, y)].symbol())
                        .collect::<String>()
                        .contains(label)
                })
                .unwrap()
        };
        let closing = |y: u16| {
            (0..width)
                .rev()
                .find(|&x| buffer[(x, y)].symbol() == "]")
                .unwrap()
        };
        let privacy = row_of("Privacy policy");
        let focused = closing(privacy);
        assert!(buffer[(focused, privacy)].modifier.contains(Modifier::BOLD));
        assert_eq!(focused, closing(row_of("Changelog")), "at {width}x{height}");
    }
}

#[test]
fn the_change_sheet_and_log_out_dialog_draw_real_buttons_at_every_size() {
    let mut harness = on_account(true);
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let [(x, y)] = crate::test_support::find_all(buffer, "Log out")[..] else {
            panic!("one Log out button at {width}x{height}");
        };
        assert_eq!(buffer[(x, y)].fg, palette.error.fg.unwrap(), "danger");
    }
    harness.press(KeyCode::Enter);
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let [(x, y)] = crate::test_support::find_all(buffer, "Save changes")[..] else {
            panic!("Save changes at {width}x{height}:\n{rows}");
        };
        let save = &buffer[(x, y)];
        assert_eq!(save.fg, palette.base.fg.unwrap(), "plain like every button");
        assert!(!save.modifier.contains(Modifier::BOLD));
        assert_eq!(buffer[(x - 2, y)].symbol(), "[");
        assert_eq!(
            crate::test_support::find_all(buffer, "[ Back ]"),
            [],
            "Esc goes back"
        );
        assert!(
            rows.contains("Esc back"),
            "key bar at {width}x{height}:\n{rows}"
        );
        let label = crate::test_support::find_all(buffer, " New username ")[0];
        let (left, right) = crate::test_support::label_margins(buffer, label, " New username ");
        assert!(left.abs_diff(right) <= 1, "centered at {width}x{height}");
    }
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('L'));
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let focused = view(&harness).overlay.as_ref().map(|o| match o {
            Overlay::LogOut(alert) => alert.focused,
            _ => usize::MAX,
        });
        let (pressed, other) = if focused == Some(1) {
            ("Log out (y)", "Cancel (n)")
        } else {
            ("Cancel (n)", "Log out (y)")
        };
        let [(x, y)] = crate::test_support::find_all(buffer, pressed)[..] else {
            panic!("{pressed} at {width}x{height}:\n{rows}");
        };
        let label = buffer[(x, y)].modifier;
        assert!(
            label.contains(Modifier::BOLD) && !label.contains(Modifier::REVERSED),
            "the focused button is bold, not filled"
        );
        let bracket = &buffer[(x - 2, y)];
        assert_eq!(
            bracket.symbol(),
            "[",
            "bracketed at {width}x{height}:\n{rows}"
        );
        assert!(bracket.modifier.contains(Modifier::BOLD));
        assert_eq!(bracket.fg, buffer[(x, y)].fg, "brackets match the label");
        let [(x, y)] = crate::test_support::find_all(buffer, other)[..] else {
            panic!("{other} at {width}x{height}:\n{rows}");
        };
        assert!(!buffer[(x, y)].modifier.contains(Modifier::REVERSED));
        let log_out = crate::test_support::find_all(buffer, "Log out (y)")[0];
        let log_out = &buffer[log_out];
        assert_eq!(log_out.fg, palette.error.fg.unwrap(), "danger");
    }
    harness.press(KeyCode::Char('n'));
    assert!(view(&harness).overlay.is_none());
}

#[test]
fn a_rows_about_marker_is_a_button_that_lights_with_its_row() {
    let mut harness = on_account(false);
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    let marker = |harness: &Harness, label: &str| {
        let terminal = render(&harness.app, 100, 40);
        let buffer = terminal.backend().buffer().clone();
        let found = crate::test_support::find_all(&buffer, &format!("{label} [?]"));
        let (x, y) = *found.first().unwrap_or_else(|| panic!("no {label} [?]"));
        let width = u16::try_from(label.chars().count()).unwrap();
        (buffer, (x + width + 1, y))
    };
    let (buffer, at) = marker(&harness, "Email");
    crate::test_support::assert_about_marker(&buffer, &palette, at, false, "Email at rest");
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).focused(), Some(Stop::ChangeEmail));
    let (buffer, at) = marker(&harness, "Email");
    crate::test_support::assert_about_marker(&buffer, &palette, at, true, "Email focused");
    let (buffer, at) = marker(&harness, "Verification");
    crate::test_support::assert_about_marker(&buffer, &palette, at, false, "another row");
    let text = screen_text(&harness.app);
    assert!(
        text.contains("Verification [?]  "),
        "a gap before the value"
    );
}
