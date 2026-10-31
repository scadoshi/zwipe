use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{render, screen_text};

/// Login with Reset password open over it.
fn forgetting() -> Harness {
    let mut harness = Harness::new();
    harness.open(open);
    harness.clear();
    harness
}

fn form(harness: &Harness) -> &ForgotForm {
    match harness.screen() {
        Screen::ForgotPassword(form) => form,
        other => panic!("left the reset screen: {other:?}"),
    }
}

#[test]
fn a_bad_email_gets_the_fixed_message_once_typed_and_sends_nothing() {
    let mut harness = forgetting();
    harness.type_text("me@");
    assert_eq!(form(&harness).email.error(), Some(INVALID_EMAIL));
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), []);
}

#[test]
fn an_empty_submit_shows_the_error_too() {
    let mut harness = forgetting();
    assert_eq!(form(&harness).email.error(), None);
    harness.press(KeyCode::Enter);
    assert_eq!(form(&harness).email.error(), Some(INVALID_EMAIL));
}

#[test]
fn a_good_email_sends_as_typed_spins_and_may_send_again() {
    let mut harness = forgetting();
    harness.type_text("me@zwipe.net");
    harness.press(KeyCode::Enter);
    assert!(form(&harness).sending());
    assert_eq!(
        harness.pending(),
        [screens::Request::ForgotPassword(Request::Send(
            "me@zwipe.net".to_owned()
        ))]
    );
    assert!(screen_text(&harness.app).contains('◐'));
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending().len(), 2, "a second press sends again");
}

#[test]
fn success_replaces_the_form_with_the_panel() {
    let mut harness = forgetting();
    harness.type_text("me@zwipe.net");
    harness.press(KeyCode::Enter);
    harness.apply(Update::Sent);
    let form = form(&harness);
    assert!(form.sent && !form.sending());
    let screen = screen_text(&harness.app);
    assert!(screen.contains(SENT_TITLE));
    assert!(screen.contains("If that email is registered"));
    assert!(!screen.contains("Send reset link"));
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}

#[test]
fn a_failure_toasts_and_stops_the_spinner() {
    let mut harness = forgetting();
    harness.apply(Update::Failed("connection error".to_owned()));
    assert!(!form(&harness).sending());
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "connection error"
    );
}

#[test]
fn the_screen_shows_its_title_and_buttons_and_fits_a_tiny_terminal() {
    let mut harness = forgetting();
    let screen = screen_text(&harness.app);
    for text in [
        "Reset password",
        "Email",
        "Back to login",
        "Send reset link",
    ] {
        assert!(screen.contains(text), "{text}");
    }
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
    harness.open(open);
    for (width, height) in [(12, 5), (1, 1)] {
        render(&harness.app, width, height);
    }
}
