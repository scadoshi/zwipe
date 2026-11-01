//! What every screen draws around its content: no title row, a key bar that names Help & feedback and the hint key once, and the grid backdrop showing up to loose content while boxes stay solid.

use crossterm::event::KeyCode;
use ratatui::style::Color;
use uuid::Uuid;

use super::{
    Screen, account, activity, changelog, commander_maybeboard, config, deck, deck_cards,
    deck_export, deck_form, deck_import, decks, forgot_password, oracle_tags, otag_examples,
    playstyle, privacy, register, remove_cards,
};
use crate::domain::theme::Palette;
use crate::inbound::app::{App, Context};
use crate::inbound::event_loop::harness::Harness;
use crate::inbound::widgets::grid;
use crate::test_support::{BUTTON_SIZES, card, deck_profile, render, rows, user};

/// A screen as the tests open it: the title its header used to show, the key that opens its hint as the bar names it, and the screen itself.
struct Case {
    title: &'static str,
    hint: Option<&'static str>,
    harness: Harness,
}

/// Each screen opened over home, its one-time hint closed.
fn cases() -> Vec<Case> {
    let on = |open: Box<dyn FnOnce(&mut Context<'_>)>| {
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.user = Some(user(true));
        harness.open(open);
        harness.app.shared.hint = None;
        harness
    };
    let from_login = |open: fn(&mut Context<'_>)| {
        let mut harness = Harness::new();
        harness.open(open);
        harness
    };
    let id = Uuid::new_v4();
    let case = |title, hint, harness| Case {
        title,
        hint,
        harness,
    };
    let mut form_on_format = on(Box::new(deck_form::open_create));
    form_on_format.press(KeyCode::Tab);
    let mut import_on_chips = on(Box::new(move |cx| deck_import::open(cx, id)));
    import_on_chips.press(KeyCode::Tab);
    vec![
        case("Home", None, Harness::on(Screen::Home(Box::default()))),
        case("Decks", Some("?"), on(Box::new(decks::open))),
        case(
            "Deck",
            Some("?"),
            on(Box::new(|cx| deck::open(cx, deck_profile("Pile")))),
        ),
        case(
            "Export",
            Some("?"),
            on(Box::new(move |cx| deck_export::open(cx, id))),
        ),
        case(
            "Import",
            None,
            on(Box::new(move |cx| deck_import::open(cx, id))),
        ),
        case("Import", Some("?"), import_on_chips),
        case(
            "Deck cards",
            Some("?"),
            on(Box::new(move |cx| deck_cards::open(cx, id))),
        ),
        case(
            "Remove deck cards",
            Some("?"),
            on(Box::new(move |cx| remove_cards::open(cx, id))),
        ),
        case("Create deck", None, on(Box::new(deck_form::open_create))),
        case("Create deck", Some("?"), form_on_format),
        case(
            "Add deck cards",
            Some("?"),
            Harness::swiping(vec![card("Krenko")]).0,
        ),
        case("Account", Some("?"), on(Box::new(account::open))),
        case("Config", Some("?"), on(Box::new(config::open))),
        case("Activity", Some("H"), on(Box::new(activity::open))),
        case("Playstyle", Some("H"), on(Box::new(playstyle::open))),
        case(
            "Commander maybeboard",
            Some("H"),
            on(Box::new(commander_maybeboard::open)),
        ),
        case("Oracle tags", Some("?"), on(Box::new(oracle_tags::open))),
        case(
            "Examples",
            Some("?"),
            on(Box::new(|cx| otag_examples::open(cx, "ramp"))),
        ),
        case("Privacy policy", None, on(Box::new(privacy::open))),
        case("Changelog", None, on(Box::new(changelog::open))),
        case("Login", None, Harness::new()),
        case("Create profile", None, from_login(register::open)),
        case("Reset password", None, from_login(forgot_password::open)),
    ]
}

/// Whether `row`, `width` cells wide, holds `title` centered across it, where a title row would put it.
fn centered_title(row: &str, title: &str, width: u16) -> bool {
    let cells: Vec<char> = row.chars().collect();
    let want: Vec<char> = title.chars().collect();
    let len = want.len();
    let middle = (usize::from(width.saturating_sub(2)).saturating_sub(len) / 2).saturating_add(1);
    cells
        .windows(len.max(1))
        .enumerate()
        .any(|(x, window)| window == want.as_slice() && x.abs_diff(middle) <= 1)
}

#[test]
fn no_screen_draws_its_title_on_the_first_row() {
    for Case { title, harness, .. } in cases() {
        for (width, height) in BUTTON_SIZES {
            let rows = rows(&harness.app, width, height);
            let first = rows.first().unwrap();
            assert!(
                !centered_title(first, title, width),
                "{title} still heads the first row at {width}x{height}:\n{}",
                rows.join("\n")
            );
        }
    }
}

#[test]
fn the_key_bar_names_f1_help_on_every_screen() {
    let help = "F1 help";
    for Case { title, harness, .. } in cases() {
        for (width, height) in BUTTON_SIZES {
            let rows = rows(&harness.app, width, height);
            let bar = rows.iter().rev().take(4).any(|row| row.contains(help));
            assert!(
                bar,
                "{title}'s key bar has no {help:?} at {width}x{height}:\n{}",
                rows.join("\n")
            );
            let count: usize = rows.iter().map(|row| row.matches(" help").count()).sum();
            assert_eq!(count, 1, "{title} shows one help hint at {width}x{height}");
        }
    }
}

#[test]
fn the_hint_key_shows_once_on_screens_with_a_hint_and_never_without() {
    for Case {
        title,
        hint,
        harness,
    } in cases()
    {
        for (width, height) in BUTTON_SIZES {
            let rows = rows(&harness.app, width, height);
            let screen = rows.join("\n");
            for key in ["?", "H"] {
                let label = format!("{key} hint");
                let count: usize = rows.iter().map(|row| row.matches(&label).count()).sum();
                let want = usize::from(hint == Some(key));
                assert_eq!(
                    count, want,
                    "{title} shows {label:?} {count} times at {width}x{height}:\n{screen}"
                );
            }
            if let Some(key) = hint {
                let label = format!("{key} hint");
                assert!(
                    rows.iter().rev().take(4).any(|row| row.contains(&label)),
                    "{title}'s {label:?} is not in its key bar at {width}x{height}:\n{screen}"
                );
            }
        }
    }
}

#[test]
fn a_hint_key_still_opens_its_hint_and_f1_opens_help() {
    for Case {
        title,
        hint,
        mut harness,
    } in cases()
    {
        let Some(key) = hint.and_then(|k| k.chars().next()) else {
            continue;
        };
        harness.press(KeyCode::Char(key));
        assert!(
            harness.app.shared.hint.is_some(),
            "{key} opens {title}'s hint"
        );
        harness.app.shared.hint = None;
        harness.press(KeyCode::F(1));
        assert!(
            harness.app.shared.help.is_some(),
            "F1 opens help on {title}"
        );
    }
}

/// The backgrounds of `app` in a colored theme, row by row, and the theme's plain background.
fn backgrounds(app: &mut App, width: u16, height: u16) -> (Vec<Vec<Color>>, Color) {
    let palette = Palette::for_account("gruvbox", true);
    app.shared.palette = palette;
    let terminal = render(app, width, height);
    let buffer = terminal.backend().buffer();
    let rows = (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).unwrap().bg)
                .collect()
        })
        .collect();
    (rows, palette.base.bg.unwrap())
}

#[test]
fn the_grid_shows_around_loose_content_and_boxes_stay_solid() {
    for (width, height) in BUTTON_SIZES {
        // Account's first rows are loose text: a section title over its rows.
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.user = Some(user(true));
        harness.open(account::open);
        harness.app.shared.hint = None;
        let (rows, base) = backgrounds(&mut harness.app, width, height);
        assert!(
            rows[0].iter().all(|&bg| bg != base),
            "every cell of Account's first row carries a tile at {width}x{height}"
        );
        // The deck list sits in a panel whose inside fills with the plain background.
        let mut harness = Harness::on(Screen::Home(Box::default()));
        harness.app.shared.user = Some(user(true));
        harness.open(decks::open);
        harness.app.shared.hint = None;
        let terminal = {
            harness.app.shared.palette = Palette::for_account("gruvbox", true);
            render(&harness.app, width, height)
        };
        let buffer = terminal.backend().buffer();
        let corner = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .find(|&(x, y)| buffer[(x, y)].symbol() == "╭" && y < height / 2)
            .unwrap_or_else(|| panic!("the deck list's panel at {width}x{height}"));
        let inside = &buffer[(corner.0 + 2, corner.1 + 1)];
        assert_eq!(
            inside.bg, base,
            "a panel's inside stays solid at {width}x{height}"
        );
        assert_ne!(
            buffer[(0, 0)].bg,
            base,
            "the margin beside the deck list carries a tile at {width}x{height}"
        );
    }
}

#[test]
fn the_logo_leaves_no_plain_background_box() {
    for (width, height) in BUTTON_SIZES {
        let mut harness = Harness::new();
        let (rows, base) = backgrounds(&mut harness.app, width, height);
        let terminal = render(&harness.app, width, height);
        let buffer = terminal.backend().buffer();
        let logo: Vec<(u16, u16)> = (0..height)
            .flat_map(|y| (0..width).map(move |x| (x, y)))
            .filter(|&(x, y)| buffer[(x, y)].symbol() == "█")
            .collect();
        if logo.is_empty() {
            assert!(height < 24, "Login shows its logo at {width}x{height}");
            continue;
        }
        let (left, right) = (
            logo.iter().map(|p| p.0).min().unwrap(),
            logo.iter().map(|p| p.0).max().unwrap(),
        );
        let (top, bottom) = (
            logo.iter().map(|p| p.1).min().unwrap(),
            logo.iter().map(|p| p.1).max().unwrap(),
        );
        let tiles = grid::shades(&harness.app.shared.palette, buffer.area, None);
        for y in top..=bottom {
            for x in left..=right {
                if buffer[(x, y)].symbol() != " " {
                    continue;
                }
                let at = usize::from(y) * usize::from(width) + usize::from(x);
                let tile = tiles[at].unwrap_or(base);
                assert_eq!(
                    rows[usize::from(y)][usize::from(x)],
                    tile,
                    "the logo's blank cell at {x},{y} keeps its tile at {width}x{height}"
                );
            }
        }
    }
}

#[test]
fn the_terminals_own_colors_draw_no_tiles_on_a_screen() {
    let harness = Harness::on(Screen::Home(Box::default()));
    let terminal = render(&harness.app, 80, 24);
    assert!(
        terminal
            .backend()
            .buffer()
            .content()
            .iter()
            .all(|cell| cell.bg == Color::Reset),
        "the terminal palette leaves every background as the terminal's"
    );
}
