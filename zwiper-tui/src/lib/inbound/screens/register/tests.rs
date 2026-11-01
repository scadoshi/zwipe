use crossterm::event::KeyCode;

use super::*;
use crate::inbound::event_loop::Flow;
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
        form.focus = Some(RegisterField::CreateProfile);
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
    assert_eq!(login.focus, Some(LoginField::Password));
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
fn f2_on_password_opens_the_rules() {
    let mut harness = registering();
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::F(2));
    assert!(screen_text(&harness.app).contains("At least 6 different characters"));
    harness.press(KeyCode::Enter);
    assert!(harness.app.shared.hint.is_none());
    assert!(matches!(harness.screen(), Screen::Register(_)));
}

#[test]
fn esc_ends_typing_then_goes_back_to_login() {
    let mut harness = registering();
    harness.type_text("scotty");
    harness.press(KeyCode::Esc);
    assert!(
        matches!(harness.screen(), Screen::Register(_)),
        "still here"
    );
    assert_eq!(form(&harness).focus, None, "no field is active");
    assert_eq!(harness.press(KeyCode::Char('x')), Flow::Continue);
    assert_eq!(
        form(&harness).username.value(),
        "scotty",
        "letters type nowhere"
    );
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).focus, Some(RegisterField::Username));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::BackTab);
    assert_eq!(form(&harness).focus, Some(RegisterField::Confirm));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
}

#[test]
fn q_types_in_a_field_and_quits_with_none_active() {
    let mut harness = registering();
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Continue);
    assert_eq!(form(&harness).username.value(), "q");
    harness.press(KeyCode::Esc);
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Quit);
}

#[test]
fn the_key_bar_names_enter_and_esc_for_where_the_focus_is() {
    let bar = |harness: &Harness| {
        let rows = crate::test_support::rows(&harness.app, 120, 40);
        rows.last()
            .unwrap()
            .split_whitespace()
            .collect::<Vec<_>>()
            .join(" ")
    };
    let mut harness = registering();
    assert_eq!(bar(&harness), "Tab move Enter next F1 help Esc done");
    if let Screen::Register(form) = harness.screen_mut() {
        form.focus = Some(RegisterField::CreateProfile);
    }
    assert_eq!(
        bar(&harness),
        "Tab move Enter create profile F1 help Esc done"
    );
    harness.press(KeyCode::Esc);
    assert_eq!(bar(&harness), "Tab fields F1 help Esc back q quit");
}

#[test]
fn create_profile_sits_right_under_confirm_password() {
    let harness = registering();
    for (width, height) in [(120, 40), (80, 24)] {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let [(_, label)] = crate::test_support::find_all(buffer, " Confirm password ")[..] else {
            panic!("one Confirm password at {width}x{height}:\n{rows}");
        };
        let found = crate::test_support::find_all(buffer, "Create profile");
        let Some(&(_, button)) = found.last() else {
            panic!("the button at {width}x{height}:\n{rows}");
        };
        let gap = button - (label + 2) - 1;
        assert!(gap <= 1, "{gap} rows at {width}x{height}:\n{rows}");
    }
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
        "Password [?]",
        "Confirm password",
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
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc done");
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Tab);
    assert_eq!(form(&harness).focus, Some(RegisterField::Password));
    crate::test_support::assert_key_bar_fits(&harness.app, "Esc done");
    harness.press(KeyCode::Esc);
    crate::test_support::assert_key_bar_fits(&harness.app, "q quit");
}

#[test]
fn the_password_label_carries_its_about_marker_lit_while_the_field_is_focused() {
    let mut harness = registering();
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    for (focus, lit) in [
        (Some(RegisterField::Username), false),
        (Some(RegisterField::Password), true),
    ] {
        if let Screen::Register(form) = harness.screen_mut() {
            form.focus = focus;
        }
        for (width, height) in crate::test_support::BUTTON_SIZES {
            let terminal = render(&harness.app, width, height);
            let buffer = terminal.backend().buffer();
            let Some(&(x, y)) = crate::test_support::find_all(buffer, " Password [?] ").first()
            else {
                assert!(height < 24, "the Password label shows at {width}x{height}");
                continue;
            };
            let context = format!("{focus:?} at {width}x{height}");
            crate::test_support::assert_about_marker(buffer, &palette, (x + 10, y), lit, &context);
            assert_eq!(
                buffer[(x + 1, y)].fg,
                palette.tertiary.fg.unwrap(),
                "{context}"
            );
        }
    }
}
