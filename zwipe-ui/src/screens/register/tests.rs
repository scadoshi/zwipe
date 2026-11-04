use super::*;
use crate::account::PASSWORDS_DIFFER;
use crate::toast::NORMAL;

const GOOD: &str = "Abcdef1!";

fn type_in(state: &mut State, field: Field, value: &str) {
    let action = match field {
        Field::Username => Action::EditUsername(value.to_owned()),
        Field::Email => Action::EditEmail(value.to_owned()),
        Field::Password => Action::EditPassword(Password::new(value)),
        Field::ConfirmPassword => Action::EditConfirm(Password::new(value)),
    };
    assert_eq!(state.update(action), [], "typing does nothing else");
}

/// A form whose four fields pass.
fn filled() -> State {
    let mut state = State::new();
    type_in(&mut state, Field::Username, "newbie");
    type_in(&mut state, Field::Email, " newbie@zwipe.net ");
    type_in(&mut state, Field::Password, GOOD);
    type_in(&mut state, Field::ConfirmPassword, GOOD);
    state
}

/// A filled form whose registration is out, with the ticket it holds.
fn sending() -> (State, Ticket) {
    let mut state = filled();
    let effects = state.update(Action::Submit);
    let [Effect::Register { ticket, .. }] = effects.as_slice() else {
        panic!("Submit sends: {effects:?}");
    };
    (state, *ticket)
}

fn errors(state: &State) -> [Option<String>; 4] {
    Field::ALL.map(|field| state.error(field))
}

#[test]
fn a_new_screen_shows_no_lines_and_is_idle() {
    let state = State::new();
    assert_eq!(errors(&state), [None, None, None, None]);
    assert!(!state.is_loading());
    assert_eq!(state.submit_label(), CREATE_PROFILE);
}

#[test]
fn a_field_shows_its_line_once_it_has_held_something() {
    let mut state = State::new();
    type_in(&mut state, Field::Username, "ab");
    assert_eq!(
        state.error(Field::Username).as_deref(),
        Some("Must be at least 3 characters long")
    );
    assert_eq!(state.error(Field::Email), None, "untouched");
    type_in(&mut state, Field::Email, "me.zwipe.net");
    assert_eq!(
        state.error(Field::Email).as_deref(),
        Some("Missing @ symbol")
    );
    type_in(&mut state, Field::Username, "");
    assert_eq!(
        state.error(Field::Username).as_deref(),
        Some("Must be at least 3 characters long"),
        "an emptied field stays judged"
    );
}

#[test]
fn the_email_is_checked_trimmed() {
    let mut state = State::new();
    type_in(&mut state, Field::Email, "  me@zwipe.net ");
    assert_eq!(state.error(Field::Email), None);
}

#[test]
fn typing_in_the_confirmation_judges_the_password() {
    let mut state = State::new();
    type_in(&mut state, Field::ConfirmPassword, "x");
    assert_eq!(
        state.error(Field::Password).as_deref(),
        Some("Must be at least 8 characters long")
    );
    assert_eq!(
        state.error(Field::ConfirmPassword),
        None,
        "the confirmation has no line"
    );
}

#[test]
fn a_short_confirmation_waits_until_a_submit() {
    let mut state = State::new();
    type_in(&mut state, Field::Password, GOOD);
    type_in(&mut state, Field::ConfirmPassword, "Abc");
    assert_eq!(state.error(Field::Password), None, "still typing");
    type_in(&mut state, Field::ConfirmPassword, "Abcdef1?");
    assert_eq!(
        state.error(Field::Password).as_deref(),
        Some(PASSWORDS_DIFFER)
    );
    type_in(&mut state, Field::ConfirmPassword, "Abc");
    assert_eq!(state.update(Action::Submit), []);
    assert_eq!(
        state.error(Field::Password).as_deref(),
        Some(PASSWORDS_DIFFER),
        "judged after a submit"
    );
}

#[test]
fn a_submit_with_a_failing_field_judges_every_field_and_sends_nothing() {
    let mut state = State::new();
    assert_eq!(state.update(Action::Submit), []);
    assert_eq!(
        errors(&state),
        [
            Some("Must be at least 3 characters long".to_owned()),
            Some("Missing @ symbol".to_owned()),
            Some("Must be at least 8 characters long".to_owned()),
            None,
        ]
    );
    assert!(!state.is_loading());
}

#[test]
fn a_passing_submit_sends_the_email_trimmed() {
    let mut state = filled();
    let effects = state.update(Action::Submit);
    let [
        Effect::Register {
            username,
            email,
            password,
            ..
        },
    ] = effects.as_slice()
    else {
        panic!("Submit sends: {effects:?}");
    };
    assert_eq!(username, "newbie");
    assert_eq!(email, "newbie@zwipe.net");
    assert_eq!(password.read(), GOOD);
    assert!(state.is_loading());
    assert_eq!(state.submit_label(), CREATING);
}

#[test]
fn a_second_submit_while_one_is_out_does_nothing() {
    let (mut state, _) = sending();
    assert_eq!(state.update(Action::Submit), []);
}

#[test]
fn a_registration_that_goes_through_prefills_login_toasts_and_goes_there() {
    let (mut state, ticket) = sending();
    type_in(&mut state, Field::Username, "changed");
    assert_eq!(
        state.update(Action::Registered { ticket }),
        vec![
            Effect::Prefill("newbie".to_owned()),
            Effect::Toast(Toast::new(ToastKind::Info, CHECK_INBOX, NORMAL)),
            Effect::Navigate(Destination::Login),
        ],
        "the username sent, not the one typed since"
    );
    assert!(!state.is_loading());
}

#[test]
fn a_failed_registration_frees_the_form() {
    let (mut state, ticket) = sending();
    assert_eq!(state.update(Action::Failed { ticket }), []);
    assert!(!state.is_loading());
    assert!(matches!(
        state.update(Action::Submit).as_slice(),
        [Effect::Register { .. }]
    ));
}

#[test]
fn a_late_reply_is_ignored() {
    let (mut state, first) = sending();
    assert_eq!(state.update(Action::Failed { ticket: first }), []);
    let effects = state.update(Action::Submit);
    let [Effect::Register { ticket: second, .. }] = effects.as_slice() else {
        panic!("Submit sends: {effects:?}");
    };
    let second = *second;
    assert_eq!(
        state.update(Action::Registered { ticket: first }),
        [],
        "an earlier registration's reply lands over the newer one"
    );
    assert!(state.is_loading(), "the newer registration is still out");
    assert_eq!(state.update(Action::Registered { ticket: second }).len(), 3);
    assert_eq!(
        state.update(Action::Registered { ticket: second }),
        [],
        "a repeated reply lands twice"
    );
}

#[test]
fn back_goes_back() {
    let mut state = State::new();
    assert_eq!(
        state.update(Action::Back),
        vec![Effect::Navigate(Destination::Previous)]
    );
}

#[test]
fn no_password_shows_in_debug() {
    let (state, _) = sending();
    let mut other = filled();
    let effects = other.update(Action::Submit);
    for shown in [
        format!("{state:?}"),
        format!("{effects:?}"),
        format!("{:?}", Action::EditConfirm(Password::new(GOOD))),
    ] {
        assert!(!shown.contains(GOOD), "{shown}");
    }
}

#[test]
fn the_fields_keep_the_apps_labels() {
    assert_eq!(
        Field::ALL.map(|f| (f.label(), f.placeholder(), f.is_masked())),
        [
            ("Username", "Username", false),
            ("Email", "Email", false),
            ("Password", "Password", true),
            ("Confirm password", "Confirm password", true),
        ]
    );
}

#[test]
fn the_copy_matches_the_app() {
    assert_eq!(
        [TITLE, BACK, CREATE_PROFILE, CREATING, CHECK_INBOX],
        [
            "Create profile",
            "Back to login",
            "Create profile",
            "Creating...",
            "Check your inbox to verify your email, then log in"
        ]
    );
}
