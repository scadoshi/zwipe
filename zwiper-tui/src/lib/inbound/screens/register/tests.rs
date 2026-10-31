use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::screens::login::LoginField;
use crate::test_support::{render, screen_text};

/// Login with Create profile open over it, nothing queued.
fn registering() -> Harness {
    let mut harness = Harness::new();
    harness.open(open);
    harness.clear();
    harness
}

fn form(harness: &Harness) -> &RegisterForm {
    match harness.screen() {
        Screen::Register(form) => form,
        other => panic!("left the register screen: {other:?}"),
    }
}

/// Types the four fields in order, Tab between them, skipping the Show buttons.
fn fill(harness: &mut Harness, username: &str, email: &str, password: &str, confirm: &str) {
    harness.type_text(username);
    harness.press(KeyCode::Tab);
    harness.type_text(email);
    harness.press(KeyCode::Tab);
    harness.type_text(password);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    harness.type_text(confirm);
}

#[test]
fn opening_records_the_view_anonymously() {
    let mut harness = Harness::new();
    harness.open(open);
    assert_eq!(
        harness.pending(),
        [
            screens::Request::FetchCounts,
            screens::Request::Record(AnonymousEventKind::RegisterViewed)
        ]
    );
    assert!(matches!(harness.screen(), Screen::Register(_)));
}

#[test]
fn fields_validate_live_once_they_have_held_something() {
    let mut harness = registering();
    harness.type_text("ab");
    assert_eq!(
        form(&harness).username.error(),
        Some("must be at least 3 characters long")
    );
    harness.type_text("c");
    assert_eq!(form(&harness).username.error(), None);
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).email.error(), None, "untouched");
    harness.type_text("me");
    assert_eq!(form(&harness).email.error(), Some("missing @ symbol"));
    harness.press(KeyCode::Tab);
    harness.type_text("short");
    assert_eq!(
        form(&harness).password.error(),
        Some("must be at least 8 characters long")
    );
}

#[test]
fn the_mismatch_shows_under_password_once_the_confirmation_is_as_long() {
    let mut harness = registering();
    fill(
        &mut harness,
        "scotty",
        "me@zwipe.net",
        "Str0ng!pass",
        "Str0ng",
    );
    assert_eq!(form(&harness).password.error(), None, "still typing");
    harness.type_text("!pasx");
    assert_eq!(form(&harness).password.error(), Some(MISMATCH));
}

#[test]
fn a_submit_shows_every_error_and_sends_nothing_until_they_pass() {
    let mut harness = registering();
    if let Screen::Register(form) = harness.screen_mut() {
        form.focus = RegisterField::CreateProfile;
    }
    harness.press(KeyCode::Enter);
    let form = form(&harness);
    assert!(form.attempted);
    assert!(form.username.error().is_some() && form.email.error().is_some());
    assert!(form.password.error().is_some());
    assert_eq!(harness.pending(), []);
}

#[test]
fn a_good_form_records_the_submit_and_sends_the_username_raw_and_the_email_trimmed() {
    let mut harness = registering();
    fill(
        &mut harness,
        "scotty",
        " me@zwipe.net ",
        "Str0ng!pass",
        "Str0ng!pass",
    );
    harness.press(KeyCode::Enter);
    assert!(form(&harness).busy);
    assert_eq!(
        harness.pending(),
        [
            screens::Request::Record(AnonymousEventKind::RegisterSubmitted),
            screens::Request::Register(Request::Register {
                username: "scotty".to_owned(),
                email: "me@zwipe.net".to_owned(),
                password: Password("Str0ng!pass".to_owned()),
            }),
        ]
    );
    harness.clear();
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), [], "one at a time");
}

#[test]
fn success_goes_back_to_login_with_the_username_and_the_apps_toast() {
    let mut harness = registering();
    harness.apply(Update::Registered {
        username: "scotty".to_owned(),
    });
    let Screen::Login(login) = harness.screen() else {
        panic!("not back on Login");
    };
    assert_eq!(login.identifier.value(), "scotty");
    assert!(login.password.is_empty());
    assert_eq!(login.focus, LoginField::Password);
    let toasts = harness.app.shared.toasts.items();
    assert_eq!(toasts[0].message, CHECK_INBOX);
    assert_eq!(toasts[0].kind, ToastKind::Info);
}

#[test]
fn a_failure_toasts_and_stays() {
    let mut harness = registering();
    harness.apply(Update::Failed("username taken".to_owned()));
    assert!(matches!(harness.screen(), Screen::Register(_)));
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "username taken"
    );
}

#[test]
fn f2_on_password_opens_the_rules_and_back_returns_to_login() {
    let mut harness = registering();
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::F(2));
    assert!(screen_text(&harness.app).contains("At least 6 different characters"));
    harness.press(KeyCode::Enter);
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}

#[test]
fn the_screen_shows_its_fields_and_buttons_and_fits_a_tiny_terminal() {
    let mut harness = registering();
    fill(&mut harness, "ab", "x", "y", "z");
    let screen = screen_text(&harness.app);
    for text in [
        "Create profile",
        "Username",
        "Email",
        "Password ?",
        "Confirm password",
        "Back to login",
        "must be at least 3 characters long",
    ] {
        assert!(screen.contains(text), "{text}");
    }
    if let Screen::Register(form) = harness.screen_mut() {
        form.busy = true;
    }
    assert!(screen_text(&harness.app).contains("Creating..."));
    for (width, height) in [(12, 5), (1, 1), (50, 14)] {
        render(&harness.app, width, height);
    }
}

#[test]
fn register_keeps_every_key_on_a_narrow_terminal() {
    let mut harness = registering();
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc back");
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).focus, RegisterField::Password);
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc back");
}
