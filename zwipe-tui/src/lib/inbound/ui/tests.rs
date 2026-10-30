//! Tests for drawing: each screen rendered into a test backend.

use std::collections::HashSet;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

use super::*;
use crate::domain::swipe::Stack;
use crate::inbound::app::{DeckList, LoginField, LoginForm, SwipeView};
use crate::test_support::{card, deck_profile};

fn render(app: &App, width: u16, height: u16) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| draw(f, app)).unwrap();
    terminal
}

fn text(app: &App) -> String {
    render(app, 100, 40)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect()
}

fn swipe_app(card: zwipe_core::domain::card::Card) -> App {
    let mut stack = Stack::new(HashSet::default());
    stack.push_page(HttpDeckCardSearch {
        cards: vec![card],
        synergy_applied: true,
        related_from: None,
    });
    let mut view = SwipeView::opening(deck_profile("Simic Ramp"));
    view.stack = Some(stack);
    view.query = None::<CardQuery>;
    let mut app = App::new(Palette::terminal());
    app.palette = Palette::for_account("gruvbox", true);
    app.screen = Screen::Swipe(Box::new(view));
    app
}

#[test]
fn the_login_screen_masks_the_password() {
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Login(LoginForm {
        identifier: "scotty".to_owned(),
        password: "hunter2".to_owned(),
        focus: LoginField::Password,
        error: Some("invalid credentials".to_owned()),
        busy: false,
    });
    let screen = text(&app);
    assert!(screen.contains("Username or email"));
    assert!(screen.contains("scotty"));
    assert!(screen.contains("•••••••"));
    assert!(!screen.contains("hunter2"));
    assert!(screen.contains("invalid credentials"));
}

#[test]
fn the_deck_list_shows_names_counts_and_the_selection() {
    let mut first = deck_profile("Simic Ramp");
    first.card_count = 42;
    first.commander_name = Some("Tatyova".to_owned());
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Decks(DeckList {
        decks: vec![first, deck_profile("Mono Red")],
        ..DeckList::default()
    });
    let screen = text(&app);
    assert!(screen.contains("> Simic Ramp"));
    assert!(screen.contains("Tatyova"));
    assert!(screen.contains("42 cards"));
    assert!(screen.contains("Mono Red"));
}

#[test]
fn the_card_shows_its_text_and_colors_its_mana_by_theme() {
    let mut elves = card("Llanowar Elves");
    elves.scryfall_data.mana_cost = Some("{G}".to_owned());
    elves.scryfall_data.type_line = Some("Creature — Elf Druid".to_owned());
    elves.scryfall_data.oracle_text = Some("{T}: Add {G}.".to_owned());
    elves.scryfall_data.power = Some("1".to_owned());
    elves.scryfall_data.toughness = Some("1".to_owned());
    let app = swipe_app(elves);

    let terminal = render(&app, 100, 40);
    let buffer = terminal.backend().buffer();
    let screen: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    for expected in [
        "Simic Ramp",
        "Llanowar Elves",
        "Creature — Elf Druid",
        "{T}: Add {G}.",
        "1/1",
        "1 left",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }

    let green = app.palette.mana[4].fg.unwrap();
    let cost = buffer
        .content()
        .iter()
        .position(|c| c.symbol() == "{" && c.fg == green)
        .is_some();
    assert!(
        cost,
        "the {{G}} cost is drawn in the theme's green mana color"
    );
}

#[test]
fn a_theme_paints_every_cell() {
    let app = swipe_app(card("Anything"));
    let background = Color::Rgb(0x28, 0x28, 0x28);
    assert!(
        render(&app, 80, 30)
            .backend()
            .buffer()
            .content()
            .iter()
            .all(|c| c.bg == background)
    );
}

#[test]
fn an_exhausted_deck_says_so() {
    let mut app = swipe_app(card("Last"));
    if let Screen::Swipe(view) = &mut app.screen {
        let stack = view.stack.as_mut().unwrap();
        stack.swipe(crate::domain::swipe::Swipe::Pass);
        stack.push_page(HttpDeckCardSearch {
            cards: Vec::new(),
            synergy_applied: true,
            related_from: None,
        });
    }
    assert!(text(&app).contains("No more cards match this deck."));
}

#[test]
fn the_update_notice_gives_the_install_command() {
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::UpdateRequired;
    let screen = text(&app);
    assert!(screen.contains("Update required"));
    assert!(screen.contains("zwipe-tui"));
}

#[test]
fn every_screen_fits_a_tiny_terminal() {
    let mut app = swipe_app(card("Tiny"));
    render(&app, 12, 5);
    app.screen = Screen::Login(LoginForm::default());
    render(&app, 12, 5);
    app.screen = Screen::Decks(DeckList::default());
    render(&app, 12, 5);
    app.screen = Screen::UpdateRequired;
    render(&app, 12, 5);
}
