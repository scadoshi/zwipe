use crossterm::event::KeyCode;
use zwipe_ui::account::INVALID_RESET_EMAIL;
use zwipe_ui::screens::forgot_password::{SEND, SENT_TITLE};

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

/// The ticket the latest send went out with.
fn last_ticket(harness: &Harness) -> Ticket {
    match harness.pending().last() {
        Some(screens::Request::ForgotPassword(Request::Send { ticket, .. })) => *ticket,
        other => panic!("no send went out: {other:?}"),
    }
}

/// Reset password with a good email typed and sent.
fn sending() -> (Harness, Ticket) {
    let mut harness = forgetting();
    harness.type_text("me@zwipe.net");
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    (harness, ticket)
}

#[test]
fn a_bad_email_gets_the_fixed_message_once_typed_and_sends_nothing() {
    let mut harness = forgetting();
    harness.type_text("me@");
    assert_eq!(form(&harness).email.error(), Some(INVALID_RESET_EMAIL));
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending(), []);
}

#[test]
fn an_empty_submit_shows_the_error_too() {
    let mut harness = forgetting();
    assert_eq!(form(&harness).email.error(), None);
    harness.press(KeyCode::Enter);
    assert_eq!(form(&harness).email.error(), Some(INVALID_RESET_EMAIL));
}

#[test]
fn a_good_email_sends_as_typed_spins_and_may_send_again() {
    let (mut harness, ticket) = sending();
    assert!(form(&harness).model.is_loading());
    assert_eq!(
        harness.pending(),
        [screens::Request::ForgotPassword(Request::Send {
            ticket,
            email: "me@zwipe.net".to_owned()
        })]
    );
    assert!(screen_text(&harness.app).contains('◐'));
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert_eq!(harness.pending().len(), 2, "a second press sends again");
    assert_ne!(
        last_ticket(&harness),
        ticket,
        "each send takes its own ticket"
    );
}

#[test]
fn success_replaces_the_form_with_the_panel() {
    let (mut harness, ticket) = sending();
    harness.apply(Update::Sent { ticket });
    let sent = form(&harness);
    assert!(sent.model.is_sent() && !sent.model.is_loading());
    assert_eq!(sent.focus, None);
    let screen = screen_text(&harness.app);
    assert!(screen.contains(SENT_TITLE));
    assert!(screen.contains("If that email is registered"));
    assert!(!screen.contains(SEND));
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).focus, None, "nothing to focus once sent");
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}

#[test]
fn a_failure_toasts_and_stops_the_spinner() {
    let (mut harness, ticket) = sending();
    harness.apply(Update::Failed {
        ticket,
        error: "connection error".to_owned(),
    });
    assert!(!form(&harness).model.is_loading());
    assert_eq!(
        harness.app.shared.toasts.items()[0].message,
        "connection error"
    );
}

#[test]
fn a_reply_already_answered_changes_nothing() {
    let (mut harness, first) = sending();
    harness.apply(Update::Failed {
        ticket: first,
        error: "connection error".to_owned(),
    });
    harness.press(KeyCode::Enter);
    harness.apply(Update::Sent { ticket: first });
    let form = form(&harness);
    assert!(form.model.is_loading(), "the second send still spins");
    assert!(
        !form.model.is_sent(),
        "the panel waits for the second reply"
    );
}

#[test]
fn the_screen_shows_its_field_and_buttons_and_fits_a_tiny_terminal() {
    let mut harness = forgetting();
    let screen = screen_text(&harness.app);
    for text in ["Email", "Send reset link"] {
        assert!(screen.contains(text), "{text}");
    }
    assert!(!screen.contains("Back to login"));
    harness.press(KeyCode::Esc);
    assert!(
        matches!(harness.screen(), Screen::ForgotPassword(_)),
        "Esc ends typing"
    );
    harness.press(KeyCode::Esc);
    assert!(
        matches!(harness.screen(), Screen::Login(_)),
        "then goes back"
    );
    harness.open(open);
    for (width, height) in [(12, 5), (1, 1)] {
        render(&harness.app, width, height);
    }
}

#[test]
fn reset_keeps_every_key_on_a_narrow_terminal() {
    let mut harness = forgetting();
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc done");
    harness.press(KeyCode::Esc);
    crate::test_support::assert_key_bar_fits(&harness.app, "q quit");
}

#[test]
fn the_key_bar_and_letters_follow_the_focus() {
    let bar = |harness: &Harness| {
        let rows = crate::test_support::rows(&harness.app, 120, 40);
        rows.last()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut harness = forgetting();
    assert_eq!(bar(&harness), "Tab move Enter send link F1 help Esc done");
    harness.type_text("q");
    assert_eq!(form(&harness).email.value(), "q", "q types in the field");
    harness.press(KeyCode::Esc);
    assert_eq!(bar(&harness), "Tab fields F1 help Esc back q quit");
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).focus, Some(ForgotField::Email));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::BackTab);
    assert_eq!(form(&harness).focus, Some(ForgotField::SendResetLink));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Tab);
    harness.type_text("me@zwipe.net");
    harness.press(KeyCode::Enter);
    let ticket = last_ticket(&harness);
    harness.apply(Update::Sent { ticket });
    assert_eq!(bar(&harness), "F1 help Esc back q quit");
    assert_eq!(
        harness.press(KeyCode::Char('q')),
        crate::inbound::event_loop::Flow::Quit
    );
}

#[test]
fn send_reset_link_sits_right_under_the_email_field() {
    let harness = forgetting();
    for (width, height) in [(120, 40), (80, 24)] {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let [(_, label)] = crate::test_support::find_all(buffer, " Email ")[..] else {
            panic!("one Email at {width}x{height}:\n{rows}");
        };
        let [(_, button)] = crate::test_support::find_all(buffer, SEND)[..] else {
            panic!("the button at {width}x{height}:\n{rows}");
        };
        let gap = button - (label + 2) - 1;
        assert!(gap <= 1, "{gap} rows at {width}x{height}:\n{rows}");
    }
}
