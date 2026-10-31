use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::style::Modifier;

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

#[test]
fn login_keeps_every_key_on_a_narrow_terminal() {
    crate::test_support::assert_key_bar_fits(&Harness::new().app, "Esc quit");
}

#[test]
fn the_hero_boxes_each_counter_in_a_tile_under_the_logo() {
    let mut harness = Harness::new();
    harness.app.shared.counts = crate::inbound::widgets::hero::Counts {
        metrics: Some(zwipe_core::http::contracts::metrics::HttpPublicMetrics {
            cards_swiped: 261_100,
            searches: 31_200,
            decks_created: 2_220,
        }),
        fetched_at: Some(std::time::Instant::now()),
    };
    for (width, height) in [(200, 50), (120, 40), (80, 24), (60, 20)] {
        let rows = crate::test_support::rows(&harness.app, width, height);
        let text = rows.join("\n");
        let tiles = rows
            .iter()
            .find(|r| r.contains('╭'))
            .map(|r| r.matches('╭').count());
        assert_eq!(tiles, Some(3), "tiles at {width}x{height}:\n{text}");
        for (number, label) in [
            ("2,220", "Decks created"),
            ("31.2k", "Searches run"),
            ("261.1k", "Cards swiped"),
        ] {
            let row = rows.iter().position(|r| r.contains(label)).unwrap();
            assert!(
                rows[row - 1].contains(number),
                "{number} over {label} at {width}x{height}:\n{text}"
            );
        }
        assert!(text.contains("Everybody") && text.contains("Live"));
        if height >= 24 {
            assert!(
                text.contains('▉'),
                "the logo stays at {width}x{height}:\n{text}"
            );
        }
    }
}

#[test]
fn the_focused_button_fills_and_log_in_is_primary_at_every_size() {
    let mut harness = Harness::new();
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    if let Screen::Login(form) = harness.screen_mut() {
        form.focus = LoginField::CreateProfile;
    }
    for (width, height) in crate::test_support::BUTTON_SIZES {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let [(x, y)] = crate::test_support::find_all(buffer, "Create profile")[..] else {
            panic!("one Create profile at {width}x{height}:\n{rows}");
        };
        let create = &buffer[(x, y)];
        assert!(
            create
                .modifier
                .contains(Modifier::REVERSED | Modifier::BOLD),
            "focused at {width}x{height}"
        );
        assert_eq!(create.fg, palette.accent.fg.unwrap());
        let [(x, y)] = crate::test_support::find_all(buffer, "Log in")[..] else {
            panic!("one Log in at {width}x{height}:\n{rows}");
        };
        let log_in = &buffer[(x, y)];
        assert!(!log_in.modifier.contains(Modifier::REVERSED));
        assert!(
            log_in.modifier.contains(Modifier::BOLD),
            "primary at {width}x{height}"
        );
        assert_eq!(log_in.fg, palette.accent.fg.unwrap());
        let boxed = buffer[(x - 2, y)].symbol() == "│";
        assert_eq!(
            boxed,
            height != 24,
            "boxes unless they cost the logo at {width}x{height}:\n{rows}"
        );
    }
}

#[test]
fn placeholders_draw_dimmer_and_labels_sit_centered_at_every_size() {
    let mut harness = Harness::new();
    for (theme, dark) in [("gruvbox", true), ("ayu", false)] {
        let palette = crate::domain::theme::Palette::for_account(theme, dark);
        harness.app.shared.palette = palette;
        for (width, height) in crate::test_support::BUTTON_SIZES {
            let terminal = render(&harness.app, width, height);
            let buffer = terminal.backend().buffer();
            let found = crate::test_support::find_all(buffer, "Username or email");
            let [label, placeholder] = found[..] else {
                panic!("label and placeholder at {width}x{height}: {found:?}");
            };
            let (left, right) = crate::test_support::label_margins(
                buffer,
                (label.0 - 1, label.1),
                " Username or email ",
            );
            assert!(
                left.abs_diff(right) <= 1,
                "centered at {width}x{height}: {left} {right}"
            );
            let cell = &buffer[placeholder];
            assert_eq!(
                cell.fg,
                palette.placeholder().fg.unwrap(),
                "{theme} at {width}x{height}"
            );
            assert_ne!(cell.fg, palette.base.fg.unwrap());
            assert!(cell.modifier.contains(Modifier::ITALIC));
        }
    }
}
