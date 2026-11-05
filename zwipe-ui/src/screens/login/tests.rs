use super::*;
use crate::account::MISSING_INPUTS;
use crate::toast::{NORMAL, ToastKind};

fn session() -> Box<Session> {
    Box::new(
        serde_json::from_value(serde_json::json!({
            "user": {
                "id": "00000000-0000-0000-0000-000000000001",
                "username": "scotty",
                "email": "scotty@zwipe.net",
                "email_verified_at": null,
            },
            "access_token": {"value": "aGVhZA.cGF5bG9hZA.c2ln", "expires_at": "2030-01-01T01:00:00Z"},
            "refresh_token": {"value": "r", "expires_at": "2030-01-15T00:00:00Z"},
            "settings": zwipe_core::domain::user::UserSettings::default(),
        }))
        .unwrap(),
    )
}

/// A form with both fields typed.
fn filled(identifier: &str, password: &str) -> State {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::EditIdentifier(identifier.to_owned())),
        []
    );
    assert_eq!(
        state.update(Action::EditPassword(Password::new(password))),
        []
    );
    state
}

/// A filled form whose login is out, with the ticket it holds.
fn sending() -> (State, Ticket) {
    let mut state = filled("scotty", "hunter22");
    let effects = state.update(Action::Submit);
    let [Effect::SignIn { ticket, .. }] = effects.as_slice() else {
        panic!("Submit sends: {effects:?}");
    };
    (state, *ticket)
}

#[test]
fn a_new_screen_is_empty_and_idle() {
    let state = State::new();
    assert_eq!(state.identifier(), "");
    assert!(!state.is_loading());
    assert_eq!(state.submit_label(), LOG_IN);
}

#[test]
fn submit_sends_the_identifier_as_typed() {
    let mut state = filled(" scotty ", "  ");
    let effects = state.update(Action::Submit);
    let [
        Effect::SignIn {
            identifier,
            password,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("Submit sends: {effects:?}");
    };
    assert_eq!(identifier, " scotty ", "untrimmed");
    assert_eq!(
        password.read(),
        "  ",
        "login applies no policy to the password"
    );
    assert!(state.is_loading());
    assert_eq!(state.submit_label(), LOGGING_IN);
}

#[test]
fn an_empty_field_raises_the_missing_inputs_toast() {
    for (identifier, password) in [("", "pw"), ("   ", "pw"), ("scotty", "")] {
        let mut state = filled(identifier, password);
        assert_eq!(
            state.update(Action::Submit),
            vec![Effect::Toast(Toast::new(
                ToastKind::Error,
                MISSING_INPUTS,
                NORMAL
            ))],
            "identifier {identifier:?}, password {password:?}"
        );
        assert!(!state.is_loading());
    }
}

#[test]
fn a_second_submit_while_one_is_out_does_nothing() {
    let (mut state, _) = sending();
    assert_eq!(state.update(Action::Submit), []);
}

#[test]
fn a_login_that_goes_through_hands_over_the_session_and_goes_home() {
    let (mut state, ticket) = sending();
    let session = session();
    assert_eq!(
        state.update(Action::LoggedIn {
            ticket,
            session: session.clone(),
        }),
        vec![
            Effect::SignedIn(session),
            Effect::ClearPassword,
            Effect::Navigate(Destination::Home),
        ]
    );
    assert!(!state.is_loading());
    assert_eq!(state.identifier(), "scotty", "the identifier stays");
    let effects = state.update(Action::Submit);
    assert_eq!(
        effects,
        vec![Effect::Toast(account::missing_inputs_toast())],
        "the password was emptied"
    );
}

#[test]
fn a_failed_login_frees_the_form_and_keeps_both_fields() {
    let (mut state, ticket) = sending();
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert!(!state.is_loading());
    assert_eq!(state.submit_label(), LOG_IN);
    let effects = state.update(Action::Submit);
    assert!(
        matches!(effects.as_slice(), [Effect::SignIn { identifier, password, .. }] if identifier == "scotty" && password.read() == "hunter22"),
        "{effects:?}"
    );
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, first) = sending();
    assert_eq!(state.update(Action::Failed { ticket: first }), []);
    let effects = state.update(Action::Submit);
    let [Effect::SignIn { ticket: second, .. }] = effects.as_slice() else {
        panic!("Submit sends: {effects:?}");
    };
    let second = *second;
    assert_eq!(
        state.update(Action::LoggedIn {
            ticket: first,
            session: session(),
        }),
        [],
        "an earlier login's reply lands over the newer one"
    );
    assert_eq!(state.update(Action::Failed { ticket: first }), []);
    assert!(state.is_loading(), "the newer login is still out");
    let newer = session();
    assert_eq!(
        state
            .update(Action::LoggedIn {
                ticket: second,
                session: newer.clone(),
            })
            .first(),
        Some(&Effect::SignedIn(newer))
    );
    assert_eq!(
        state.update(Action::LoggedIn {
            ticket: second,
            session: session(),
        }),
        [],
        "a repeated reply lands twice"
    );
}

#[test]
fn a_reply_with_nothing_out_is_ignored() {
    let mut state = filled("scotty", "hunter22");
    let ticket = Generation::default().next_ticket();
    assert_eq!(
        state.update(Action::LoggedIn {
            ticket,
            session: session(),
        }),
        []
    );
    assert_eq!(state, filled("scotty", "hunter22"));
}

#[test]
fn the_prefill_fills_the_identifier_and_empties_the_password() {
    let mut state = filled("old", "hunter22");
    assert_eq!(
        state.update(Action::Prefill("newbie".to_owned())),
        vec![
            Effect::SetIdentifier("newbie".to_owned()),
            Effect::ClearPassword,
        ]
    );
    assert_eq!(state.identifier(), "newbie");
    assert_eq!(
        state.update(Action::Submit),
        vec![Effect::Toast(account::missing_inputs_toast())],
        "the password is empty"
    );
}

#[test]
fn create_profile_and_forgot_password_navigate() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::CreateProfile),
        vec![Effect::Navigate(Destination::Register)]
    );
    assert_eq!(
        state.update(Action::ForgotPassword),
        vec![Effect::Navigate(Destination::ForgotPassword)]
    );
}

#[test]
fn the_other_buttons_are_held_while_a_login_is_out() {
    let (mut state, _) = sending();
    assert_eq!(state.update(Action::CreateProfile), []);
    assert_eq!(state.update(Action::ForgotPassword), []);
}

#[test]
fn no_password_shows_in_debug() {
    let (state, _) = sending();
    let mut other = filled("scotty", "hunter22");
    let effects = other.update(Action::Submit);
    for shown in [
        format!("{state:?}"),
        format!("{effects:?}"),
        format!("{:?}", Action::EditPassword(Password::new("hunter22"))),
    ] {
        assert!(!shown.contains("hunter22"), "{shown}");
        assert!(shown.contains("REDACTED"), "{shown}");
    }
}

#[test]
fn the_fields_keep_the_apps_labels() {
    assert_eq!(
        Field::ALL.map(|f| (f.label(), f.placeholder(), f.is_masked())),
        [
            ("Username or email", "Username or email", false),
            ("Password", "Password", true),
        ]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [TITLE, LOG_IN, LOGGING_IN, CREATE_PROFILE, FORGOT_PASSWORD],
        [
            "Login",
            "Log in",
            "Logging in...",
            "Create profile",
            "Forgot password"
        ]
    );
}
