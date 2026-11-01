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

/// Login with its focus set to `focus`.
fn focused(focus: Option<LoginField>) -> Harness {
    let mut harness = Harness::new();
    if let Screen::Login(form) = harness.screen_mut() {
        form.focus = focus;
    }
    harness
}

/// The key bar at 120 by 40, its hints one space apart.
fn bar(harness: &Harness) -> String {
    let rows = crate::test_support::rows(&harness.app, 120, 40);
    rows.last()
        .unwrap()
        .split_whitespace()
        .collect::<Vec<_>>()
        .join(" ")
}

#[test]
fn tab_walks_the_fields_show_and_log_in_round() {
    let mut field = LoginField::Identifier;
    let mut seen = Vec::new();
    for _ in 0..4 {
        seen.push(field);
        field = field.next();
    }
    assert_eq!(field, LoginField::Identifier, "wraps round");
    assert_eq!(
        seen,
        [
            LoginField::Identifier,
            LoginField::Password,
            LoginField::ShowPassword,
            LoginField::LogIn
        ]
    );
    assert_eq!(LoginField::Identifier.previous(), LoginField::LogIn);
    assert_eq!(LoginField::LogIn.button(), Some(0));
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
fn show_unmasks_and_hide_masks_again() {
    let mut harness = Harness::new();
    harness.press(KeyCode::Tab);
    harness.type_text("hunter2");
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::Enter);
    assert!(!form(&harness).password.is_masked(), "Show");
    assert!(screen_text(&harness.app).contains("hunter2"));
    harness.press(KeyCode::Enter);
    assert!(form(&harness).password.is_masked(), "Hide");
}

#[test]
fn esc_ends_typing_and_never_quits() {
    let mut harness = Harness::new();
    harness.type_text("scotty");
    assert_eq!(harness.press(KeyCode::Esc), Flow::Continue);
    assert_eq!(form(&harness).focus, None, "no field is active");
    assert_eq!(
        harness.press(KeyCode::Esc),
        Flow::Continue,
        "Esc again stays"
    );
    assert!(matches!(harness.screen(), Screen::Login(_)));
    assert_eq!(form(&harness).focus, None);
    assert_eq!(form(&harness).identifier.value(), "scotty");
    harness.press(KeyCode::Tab);
    assert_eq!(
        form(&harness).focus,
        Some(LoginField::Identifier),
        "Tab goes back in"
    );
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::BackTab);
    assert_eq!(
        form(&harness).focus,
        Some(LoginField::Password),
        "so does Shift+Tab"
    );
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Enter);
    assert_eq!(
        harness.pending(),
        [],
        "Enter with no field active sends nothing"
    );
}

#[test]
fn q_quits_only_with_no_field_active() {
    let mut harness = Harness::new();
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Continue);
    assert_eq!(form(&harness).identifier.value(), "q", "q types in a field");
    harness.press(KeyCode::Esc);
    assert_eq!(harness.press(KeyCode::Char('q')), Flow::Quit);
    let ctrl_c = KeyEvent::new(KeyCode::Char('c'), KeyModifiers::CONTROL);
    let mut harness = Harness::new();
    assert_eq!(
        harness.press_event(ctrl_c),
        Flow::Quit,
        "Ctrl+C from a field"
    );
}

#[test]
fn c_and_f_open_their_screens_only_with_no_field_active() {
    let mut harness = Harness::new();
    harness.type_text("cf");
    assert!(matches!(harness.screen(), Screen::Login(_)));
    assert_eq!(
        form(&harness).identifier.value(),
        "cf",
        "they type in a field"
    );
    harness.press(KeyCode::Tab);
    harness.type_text("cf");
    assert_eq!(form(&harness).password.value(), "cf");
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('c'));
    assert!(matches!(harness.screen(), Screen::Register(_)));
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Esc);
    assert!(matches!(harness.screen(), Screen::Login(_)));
    harness.press(KeyCode::Char('f'));
    assert!(matches!(harness.screen(), Screen::ForgotPassword(_)));
}

#[test]
fn the_key_bar_offers_the_forms_keys_while_typing_and_the_letters_after_esc() {
    let mut harness = Harness::new();
    assert_eq!(bar(&harness), "Tab move Enter next F1 help Esc done");
    harness.press(KeyCode::Tab);
    assert_eq!(bar(&harness), "Tab move Enter log in F1 help Esc done");
    harness.press(KeyCode::Tab);
    assert_eq!(
        bar(&harness),
        "Tab move Enter show c Create profile f Forgot password F1 help Esc done",
        "on Show"
    );
    harness.press(KeyCode::Tab);
    assert_eq!(
        bar(&harness),
        "Tab move Enter log in c Create profile f Forgot password F1 help Esc done",
        "on Log in"
    );
    harness.press(KeyCode::Esc);
    assert_eq!(
        bar(&harness),
        "Tab fields c Create profile f Forgot password F1 help q quit"
    );
}

#[test]
fn bang_opens_help_only_with_no_field_active() {
    let mut harness = Harness::new();
    harness.press(KeyCode::Char('!'));
    assert!(harness.app.shared.help.is_none(), "it types in a field");
    assert_eq!(form(&harness).identifier.value(), "!");
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Char('!'));
    assert!(harness.app.shared.help.is_some());
    harness.press(KeyCode::Esc);
    harness.press(KeyCode::Tab);
    harness.press(KeyCode::F(1));
    assert!(harness.app.shared.help.is_some(), "F1 from a field too");
}

#[test]
fn the_login_screen_shows_the_hero_fields_and_buttons_and_masks_the_password() {
    let mut harness = Harness::new();
    harness.type_text("scotty");
    harness.press(KeyCode::Tab);
    harness.type_text("hunter2");
    let screen = screen_text(&harness.app);
    for text in [
        "Decks created",
        "Everybody",
        "Username or email",
        "scotty",
        "•••••••",
        "Show",
        "Log in",
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
    crate::test_support::assert_key_bar_fits(&Harness::new().app, "Esc done");
    crate::test_support::assert_key_bar_fits(&focused(Some(LoginField::LogIn)).app, "Esc done");
    crate::test_support::assert_key_bar_fits(&focused(None).app, "q quit");
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
                text.contains('█'),
                "the logo stays at {width}x{height}:\n{text}"
            );
        }
    }
}

#[test]
fn log_in_has_no_accent_at_rest_and_takes_it_on_its_edges_and_label_when_focused() {
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    for focus in [Some(LoginField::Identifier), Some(LoginField::LogIn)] {
        let mut harness = focused(focus);
        harness.app.shared.palette = palette;
        for (width, height) in crate::test_support::BUTTON_SIZES {
            let terminal = render(&harness.app, width, height);
            let buffer = terminal.backend().buffer();
            let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
            let [(x, y)] = crate::test_support::find_all(buffer, "Log in")[..] else {
                panic!("one Log in at {width}x{height}:\n{rows}");
            };
            let log_in = &buffer[(x, y)];
            assert_eq!(
                buffer[(x - 2, y)].symbol(),
                "│",
                "Log in boxes at {width}x{height}, the logo kept:\n{rows}"
            );
            if focus == Some(LoginField::LogIn) {
                for inside in x - 1..x + 7 {
                    assert!(
                        !buffer[(inside, y)].modifier.contains(Modifier::REVERSED),
                        "cell {inside} stays unfilled at {width}x{height}"
                    );
                }
                assert!(log_in.modifier.contains(Modifier::BOLD));
                assert_eq!(log_in.fg, palette.accent.fg.unwrap());
                assert_eq!(
                    buffer[(x - 2, y)].fg,
                    palette.accent.fg.unwrap(),
                    "the edge takes the accent at {width}x{height}"
                );
            } else {
                assert!(!log_in.modifier.contains(Modifier::REVERSED));
                assert!(
                    !log_in.modifier.contains(Modifier::BOLD),
                    "no primary weight"
                );
                assert_eq!(
                    log_in.fg,
                    palette.base.fg.unwrap(),
                    "no accent at {width}x{height}"
                );
                assert_eq!(buffer[(x - 2, y)].fg, palette.border.fg.unwrap());
            }
        }
    }
}

#[test]
fn log_in_sits_right_under_the_password_field() {
    let harness = Harness::new();
    for (width, height) in [(120, 40), (80, 24)] {
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let rows = crate::test_support::rows(&harness.app, width, height).join("\n");
        let [(_, label)] = crate::test_support::find_all(buffer, " Password ")[..] else {
            panic!("one Password at {width}x{height}:\n{rows}");
        };
        let [(_, log_in)] = crate::test_support::find_all(buffer, "Log in")[..] else {
            panic!("one Log in at {width}x{height}:\n{rows}");
        };
        // The password box's bottom border is two rows under its label, and Log in's top border a row over its label.
        let gap = (log_in - 1) - (label + 2) - 1;
        assert!(
            gap <= 1,
            "{gap} rows between Password and Log in at {width}x{height}:\n{rows}"
        );
    }
}

#[test]
fn empty_fields_show_no_placeholder_and_labels_sit_centered_at_every_size() {
    let mut harness = Harness::new();
    for (theme, dark) in [("gruvbox", true), ("ayu", false)] {
        let palette = crate::domain::theme::Palette::for_account(theme, dark);
        harness.app.shared.palette = palette;
        for (width, height) in crate::test_support::BUTTON_SIZES {
            let terminal = render(&harness.app, width, height);
            let buffer = terminal.backend().buffer();
            for label in ["Username or email", "Password"] {
                let padded = format!(" {label} ");
                let found = crate::test_support::find_all(buffer, &padded);
                let [(x, y)] = found[..] else {
                    panic!("{label} once, as the label, at {width}x{height}: {found:?}");
                };
                let (left, right) = crate::test_support::label_margins(buffer, (x, y), &padded);
                assert!(
                    left.abs_diff(right) <= 1,
                    "centered at {width}x{height}: {left} {right}"
                );
                let corner = x - left - 1;
                let inside: String = (corner + 1..corner + 30)
                    .map(|c| buffer[(c, y + 1)].symbol())
                    .collect();
                assert!(
                    inside.trim().is_empty(),
                    "{label} draws empty at {width}x{height}: {inside:?}"
                );
            }
        }
    }
}

#[test]
fn the_grid_reaches_the_counter_tiles_and_the_fields() {
    let mut harness = Harness::new();
    let palette = crate::domain::theme::Palette::for_account("gruvbox", true);
    harness.app.shared.palette = palette;
    let terminal = render(&harness.app, 120, 40);
    let buffer = terminal.backend().buffer();
    let tiles = crate::inbound::widgets::grid::shades(&palette, buffer.area, None);
    let tile = |x: u16, y: u16| tiles[usize::from(y) * 120 + usize::from(x)].unwrap();
    let base = palette.base.bg.unwrap();
    let rows = crate::test_support::rows(&harness.app, 120, 40).join("\n");
    let [(x, y)] = crate::test_support::find_all(buffer, "Decks created")[..] else {
        panic!("the counters:\n{rows}");
    };
    let tile_left = (0..x)
        .rev()
        .find(|&c| buffer[(c, y)].symbol() == "│")
        .unwrap();
    let [(x, y_field)] = crate::test_support::find_all(buffer, " Username or email ")[..] else {
        panic!("the field:\n{rows}");
    };
    let field_left = (0..x)
        .rev()
        .find(|&c| buffer[(c, y_field)].symbol() == "╭")
        .unwrap();
    for (what, (bx, by)) in [
        ("counter tile", (tile_left, y)),
        ("field", (field_left, y_field + 1)),
    ] {
        assert_ne!(tile(bx, by), base, "a tile that shows beside the {what}");
        assert_eq!(
            buffer[(bx - 1, by)].bg,
            tile(bx - 1, by),
            "the cell beside the {what}"
        );
        assert_eq!(buffer[(bx, by)].bg, tile(bx, by), "the {what}'s own border");
        assert_eq!(
            buffer[(bx + 1, by)].bg,
            base,
            "inside the {what} stays solid"
        );
    }
}
