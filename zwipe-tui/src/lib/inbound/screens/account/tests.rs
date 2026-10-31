use std::time::{Duration, Instant};

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::HINT_PROFILE;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::home;
use crate::test_support::{render, screen_text, user};

fn ask(request: Request) -> screens::Request {
    request.into()
}

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

fn view_mut(harness: &mut Harness) -> &mut AccountView {
    match harness.screen_mut() {
        Screen::Account(view) => view,
        other => panic!("not on the account screen: {other:?}"),
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

#[test]
fn opening_fetches_the_user_and_shows_the_profile_hint_the_first_time() {
    let mut harness = Harness::new();
    harness.app.shared.user = Some(user(true));
    harness.open(open);
    assert_eq!(
        harness.pending(),
        [
            ask(Request::LoadUser),
            screens::Request::MarkHintShown(HINT_PROFILE.to_owned())
        ]
    );
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
        "Email ?",
        "Unverified",
        "Verification ?",
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
fn a_bad_username_shows_live_and_a_good_one_is_sent_as_typed() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Enter);
    harness.type_text("ab");
    let Some(Overlay::Form(form)) = &view(&harness).overlay else {
        panic!("no sheet");
    };
    assert_eq!(
        form.inputs[0].error(),
        Some("must be at least 3 characters long")
    );
    harness.type_text("cdef");
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), [], "the password is required");
    harness.type_text("hunter2");
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [ask(Request::Change(AccountChange::Username {
            username: "abcdef".to_owned(),
            password: Password("hunter2".to_owned()),
        }))]
    );
}

#[test]
fn a_change_closes_the_sheet_and_toasts_for_its_band() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Enter);
    let mut renamed = user(true);
    renamed.username = zwipe_core::domain::user::Username::new("abcdef").unwrap();
    harness.apply(Update::Changed {
        user: Some(Box::new(renamed)),
        message: "Username changed to abcdef".to_owned(),
        duration: toast::QUICK,
    });
    assert!(view(&harness).overlay.is_none());
    assert_eq!(view(&harness).user.username.to_string(), "abcdef");
    let up = &harness.app.shared.toasts.items()[0];
    assert_eq!(up.expires_at.duration_since(up.shown_at), toast::QUICK);
}

#[test]
fn a_failed_change_toasts_and_keeps_the_sheet() {
    let mut harness = on_account(true);
    harness.press(KeyCode::Enter);
    if let Some(Overlay::Form(form)) = &mut view_mut(&mut harness).overlay {
        form.busy = true;
    }
    harness.apply(Update::ChangeFailed("wrong password".to_owned()));
    assert!(matches!(&view(&harness).overlay, Some(Overlay::Form(form)) if !form.busy));
    assert_eq!(
        toasts(&harness),
        [(ToastKind::Error, "wrong password".to_owned())]
    );
}

#[test]
fn a_password_change_asks_first_and_checks_against_the_current_one() {
    let mut harness = on_account(true);
    for _ in 0..2 {
        harness.press(KeyCode::Char('j'));
    }
    assert_eq!(view(&harness).focused(), Some(Stop::ChangePassword));
    harness.press(KeyCode::Enter);
    if let Some(Overlay::Form(form)) = &mut view_mut(&mut harness).overlay {
        form.fill(&["Same!pass1", "Same!pass1", "Same!pass1"]);
        form.focus = 2;
    }
    harness.press(KeyCode::Enter);
    let Some(Overlay::Form(form)) = &view(&harness).overlay else {
        panic!("no sheet");
    };
    assert_eq!(
        form.inputs[1].error(),
        Some("New password must be different from your current password")
    );
    if let Some(Overlay::Form(form)) = &mut view_mut(&mut harness).overlay {
        form.fill(&["Old!pass1", "New!pass22", "New!pass22"]);
    }
    harness.press(KeyCode::Enter);
    assert!(matches!(
        view(&harness).overlay,
        Some(Overlay::ConfirmPassword { .. })
    ));
    assert!(screen_text(&harness.app).contains("Changing your password will log you out"));
    harness.press(KeyCode::Char('y'));
    assert_eq!(
        harness.pending(),
        [ask(Request::Change(AccountChange::Password {
            current: Password("Old!pass1".to_owned()),
            new: Password("New!pass22".to_owned()),
        }))]
    );
}

#[test]
fn resend_waits_a_minute_after_it_lands_unless_it_failed() {
    let mut harness = on_account(false);
    harness.press(KeyCode::Char('j'));
    harness.press(KeyCode::Char('j'));
    assert_eq!(view(&harness).focused(), Some(Stop::Resend));
    harness.press(KeyCode::Enter);
    assert_eq!(view(&harness).resend_label(), "Sending...");
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [ask(Request::Resend)],
        "once while sending"
    );
    harness.apply(Update::Resent);
    assert!(view(&harness).resend_label().starts_with("Resend in "));
    assert!(view(&harness).animating());
    assert_eq!(
        toasts(&harness),
        [(ToastKind::Success, "Verification email sent".to_owned())]
    );

    view_mut(&mut harness).resend_ready_at = None;
    harness.press(KeyCode::Enter);
    harness.apply(Update::ResendFailed("offline".to_owned()));
    assert_eq!(view(&harness).resend_label(), "Resend");
    let failed = &harness.app.shared.toasts.items()[1];
    assert_eq!(
        failed.expires_at.duration_since(failed.shown_at),
        toast::LONG
    );
}

#[test]
fn checking_verification_toasts_either_way() {
    let mut harness = on_account(false);
    for _ in 0..3 {
        harness.press(KeyCode::Char('j'));
    }
    harness.press(KeyCode::Enter);
    assert!(view(&harness).checking);
    harness.apply(Update::User {
        user: Box::new(user(true)),
        toast: Some((ToastKind::Success, "Email verified")),
    });
    assert!(view(&harness).verified() && !view(&harness).checking);
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
    assert_eq!(harness.pending(), [], "still counting down");
    if let Some(Overlay::More(Some(delete))) = &mut view_mut(&mut harness).overlay {
        delete.opened_at = Instant::now()
            .checked_sub(Duration::from_secs(6))
            .expect("the clock is past six seconds");
    }
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [ask(Request::Change(AccountChange::Delete {
            password: Password("pw".to_owned()),
        }))]
    );
    assert!(screen_text(&harness.app).contains("Deleting..."));
    harness.apply(Update::DeleteFailed("wrong password".to_owned()));
    assert!(matches!(&view(&harness).overlay, Some(Overlay::More(Some(d))) if !d.busy));
    harness.apply(Update::Deleted);
    assert!(matches!(harness.screen(), Screen::Login(_)));
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
