use super::*;
use crate::account::INVALID_RESET_EMAIL;

fn typed(email: &str) -> State {
    let mut state = State::new();
    assert_eq!(state.update(Action::EditEmail(email.to_owned())), []);
    state
}

/// A send for `state`, with the ticket it holds.
fn send(state: &mut State) -> Ticket {
    let effects = state.update(Action::Submit);
    let [Effect::RequestReset { ticket, .. }] = effects.as_slice() else {
        panic!("Submit sends: {effects:?}");
    };
    *ticket
}

#[test]
fn a_new_screen_shows_the_form_with_no_line() {
    let state = State::new();
    assert_eq!(state.error(), None);
    assert!(!state.is_loading());
    assert!(!state.is_sent());
}

#[test]
fn the_line_shows_once_the_field_has_held_something() {
    let mut state = typed("nope");
    assert_eq!(state.error().as_deref(), Some(INVALID_RESET_EMAIL));
    assert_eq!(state.update(Action::EditEmail(String::new())), []);
    assert_eq!(
        state.error().as_deref(),
        Some(INVALID_RESET_EMAIL),
        "an emptied field stays judged"
    );
    assert_eq!(state.update(Action::EditEmail("a@b.co".to_owned())), []);
    assert_eq!(state.error(), None);
}

#[test]
fn a_failing_submit_shows_the_line_and_sends_nothing() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Submit), []);
    assert_eq!(state.error().as_deref(), Some(INVALID_RESET_EMAIL));
    assert!(!state.is_loading());
}

#[test]
fn the_email_is_checked_and_sent_untrimmed() {
    let mut state = typed(" a@b.co");
    assert_eq!(state.update(Action::Submit), [], "a space fails");
    let mut state = typed("a@b.co");
    assert!(matches!(
        state.update(Action::Submit).as_slice(),
        [Effect::RequestReset { email, .. }] if email == "a@b.co"
    ));
    assert!(state.is_loading());
}

#[test]
fn a_request_that_goes_in_shows_the_panel() {
    let mut state = typed("a@b.co");
    let ticket = send(&mut state);
    assert_eq!(state.update(Action::Sent { ticket }), []);
    assert!(state.is_sent());
    assert!(!state.is_loading());
    assert_eq!(state.update(Action::Submit), [], "the button is gone");
}

#[test]
fn a_failed_request_stops_the_spinner_and_keeps_the_form() {
    let mut state = typed("a@b.co");
    let ticket = send(&mut state);
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert!(!state.is_loading());
    assert!(!state.is_sent());
}

#[test]
fn a_second_send_while_one_is_out_sends_again_and_both_replies_land() {
    let mut state = typed("a@b.co");
    let first = send(&mut state);
    let second = send(&mut state);
    assert_ne!(first, second);
    assert_eq!(state.update(Action::Failed { ticket: first }), []);
    assert!(
        !state.is_loading(),
        "any reply stops the spinner, as in the app"
    );
    assert_eq!(state.update(Action::Sent { ticket: second }), []);
    assert!(state.is_sent());
}

#[test]
fn a_late_reply_is_ignored() {
    let mut state = typed("a@b.co");
    let ticket = send(&mut state);
    assert_eq!(state.update(Action::Failed { ticket }), []);
    let before = state.clone();
    assert_eq!(state.update(Action::Sent { ticket }), []);
    assert_eq!(state, before, "a reply already settled lands again");
    let stranger = Generation::default().next_ticket();
    let mut idle = typed("a@b.co");
    assert_eq!(idle.update(Action::Sent { ticket: stranger }), []);
    assert!(!idle.is_sent(), "a reply to nothing sent shows the panel");
}

#[test]
fn back_goes_back_even_while_a_request_is_out() {
    let mut state = typed("a@b.co");
    send(&mut state);
    assert_eq!(
        state.update(Action::Back),
        vec![Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [TITLE, EMAIL, BACK, SEND, SENT_TITLE, SENT_BODY],
        [
            "Reset password",
            "Email",
            "Back to login",
            "Send reset link",
            "Check your email",
            "If that email is registered, a reset link is on its way."
        ]
    );
}
