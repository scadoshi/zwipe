use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::inbound::event_loop::Flow;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::{render, screen_text, session, user};

fn form(harness: &Harness) -> &LoginForm {
    match harness.screen() {
        Screen::Login(form) => form,
        other => panic!("left the login screen: {other:?}"),
    }
}

#[test]
fn tab_walks_the_fields_show_and_the_three_buttons_round() {
    let mut field = LoginField::Identifier;
    let mut seen = Vec::new();
    for _ in 0..6 {
        seen.push(field);
        field = field.next();
    }
    assert_eq!(field, LoginField::Identifier, "wraps round");
    assert_eq!(seen.len(), 6);
    assert_eq!(
        LoginField::Identifier.previous(),
        LoginField::ForgotPassword
    );
    assert_eq!(LoginField::CreateProfile.button(), Some(1));
    assert_eq!(LoginField::Password.button(), None);
}

#[test]
fn the_form_moves_to_the_password_then_signs_in_with_the_identifier_untrimmed() {
    let mut harness = Harness::new();
    harness.type_text(" scotty ");
    harness.press(KeyCode::Enter);
    harness.type_text("hunter2");
    harness.press(KeyCode::Enter);

    assert!(form(&harness).busy);
    assert_eq!(
        harness.pending(),
        [screens::Request::Login(Request::SignIn {
            identifier: " scotty ".to_owned(),
            password: Password("hunter2".to_owned()),
        })]
    );
    harness.clear();
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), [], "one request at a time");
}

#[test]
fn an_empty_form_is_refused_with_the_apps_toast() {
    let mut harness = Harness::new();
    harness.type_text("   ");
    harness.press(KeyCode::Tab);
    harness.type_text("pw");
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), []);
    let toasts = harness.app.shared.toasts.items();
    assert_eq!(toasts.len(), 1);
    assert_eq!(toasts[0].message, EMPTY_FORM);
    assert_eq!(toasts[0].kind, ToastKind::Error);
}

#[test]
fn a_failed_sign_in_toasts_why_and_keeps_the_fields() {
    let mut harness = Harness::new();
    harness.type_text("scotty");
    if let Screen::Login(form) = harness.screen_mut() {
        form.busy = true;
    }
    harness.apply(Update::Failed("invalid credentials".to_owned()));
    let form = form(&harness);
    assert!(!form.busy);
    assert_eq!(form.identifier.value(), "scotty");
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "invalid credentials"
    );
}

#[test]
fn signing_in_opens_home_in_the_accounts_theme_with_no_toast() {
    let mut harness = Harness::new();
    let preferences = zwipe_core::domain::user::UserPreferences {
        theme: "nord".to_owned(),
        dark_mode: false,
        ..Default::default()
    };
    let session = session(&user(true), &preferences);
    harness.apply(Update::SignedIn(Box::new(session)));
    assert!(matches!(harness.screen(), Screen::Home(_)));
    assert!(harness.app.shared.toasts.is_empty());
    assert_eq!(
        harness.app.shared.palette,
        crate::domain::theme::Palette::for_account("nord", false)
    );
}

#[test]
fn the_buttons_open_register_and_forgot_password_and_show_unmasks() {
    let mut harness = Harness::new();
    harness.press(KeyCode::Tab);
    harness.type_text("hunter2");
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(!form(&harness).password.is_masked(), "Show");
    assert!(screen_text(&harness.app).contains("hunter2"));
    harness.press(KeyCode::Enter);
    assert!(form(&harness).password.is_masked(), "Hide");

    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Right);
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::Register(_)));
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
    if let Screen::Login(form) = harness.screen_mut() {
        form.focus = LoginField::ForgotPassword;
    }
    harness.press(KeyCode::Enter);
    assert!(matches!(harness.screen(), Screen::ForgotPassword(_)));
}

#[test]
fn bang_opens_help_only_from_a_button() {
    let mut harness = Harness::new();
    harness.press(KeyCode::Char('!'));
    assert!(harness.app.shared.help.is_none(), "it types in a field");
    assert_eq!(form(&harness).identifier.value(), "!");
    if let Screen::Login(form) = harness.screen_mut() {
        form.focus = LoginField::LogIn;
    }
    harness.press(KeyCode::Char('!'));
    assert!(harness.app.shared.help.is_some());
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::F(1));
    assert!(harness.app.shared.help.is_some(), "F1 from anywhere");
}

#[test]
fn q_types_on_the_login_screen_and_ctrl_c_quits() {
    let mut harness = Harness::new();
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Continue);
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(harness.press_event(ctrl_c), Flow::Quit);
}

#[test]
fn the_login_screen_shows_the_hero_fields_and_buttons_and_masks_the_password() {
    let mut harness = Harness::new();
    harness.type_text("scotty");
    harness.press(KeyCode::Tab);
    harness.type_text("hunter2");
    let screen = screen_text(&harness.app);
    for text in [
        "Login",
        "Decks created",
        "Everybody",
        "Username or email",
        "scotty",
        "•••••••",
        "Show",
        "Log in",
        "Create profile",
        "Forgot password",
    ] {
        assert!(screen.contains(text), "{text}");
    }
    assert!(!screen.contains("hunter2"));
    if let Screen::Login(form) = harness.screen_mut() {
        form.busy = true;
    }
    assert!(screen_text(&harness.app).contains("Logging in..."));
}

#[test]
fn the_login_screen_fits_a_tiny_terminal() {
    let harness = Harness::new();
    for (width, height) in [(12, 5), (1, 1), (40, 12), (80, 24)] {
        render(&harness.app, width, height);
    }
}
