use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};

use super::*;
use crate::inbound::event_loop::Flow;
use crate::inbound::event_loop::harness::Harness;
use crate::test_support::screen_text;

fn form(harness: &Harness) -> &LoginForm {
    match harness.screen() {
        Screen::Login(form) => form,
        other => panic!("left the login screen: {other:?}"),
    }
}

#[test]
fn login_focus_toggles_between_the_two_fields() {
    assert_eq!(LoginField::Identifier.other(), LoginField::Password);
    assert_eq!(LoginField::Password.other(), LoginField::Identifier);
}

#[test]
fn the_form_moves_to_the_password_then_signs_in() {
    let mut harness = Harness::new();
    harness.type_text(" scotty ");
    harness.press(KeyCode::Enter);
    harness.type_text("hunter2");
    harness.press(KeyCode::Enter);

    assert!(form(&harness).busy);
    assert_eq!(
        harness.pending(),
        [screens::Request::Login(Request::SignIn {
            identifier: "scotty".to_owned(),
            password: Password("hunter2".to_owned()),
        })]
    );
}

#[test]
fn an_empty_form_is_refused_without_a_request() {
    let mut harness = Harness::new();
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(form(&harness).error.is_some());
    assert_eq!(harness.pending(), []);
}

#[test]
fn a_failed_sign_in_shows_why_and_allows_another_try() {
    let mut harness = Harness::new();
    if let Screen::Login(form) = harness.screen_mut() {
        form.busy = true;
    }
    harness.apply(Update::Failed("invalid credentials".to_owned()));
    let form = form(&harness);
    assert!(!form.busy);
    assert_eq!(form.error.as_deref(), Some("invalid credentials"));
}

#[test]
fn q_types_on_the_login_screen_and_ctrl_c_quits() {
    let mut harness = Harness::new();
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Continue);
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    assert_eq!(harness.press_event(ctrl_c), Flow::Quit);
}

#[test]
fn the_login_screen_masks_the_password() {
    let harness = Harness::on(Screen::Login(LoginForm {
        identifier: "scotty".to_owned(),
        password: "hunter2".to_owned(),
        focus: LoginField::Password,
        error: Some("invalid credentials".to_owned()),
        busy: false,
    }));
    let screen = screen_text(&harness.app);
    assert!(screen.contains("Username or email"));
    assert!(screen.contains("scotty"));
    assert!(screen.contains("•••••••"));
    assert!(!screen.contains("hunter2"));
    assert!(screen.contains("invalid credentials"));
}
