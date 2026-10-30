//! Tests for drawing: each screen rendered into a test backend.

use std::collections::HashSet;

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::style::Color;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::http::contracts::deck::HttpDeckCardSearch;

use super::*;
use crate::domain::home::Figures;
use crate::domain::swipe::Stack;
use crate::inbound::app::{
    DeckList, DeckView, HomeView, LoginField, LoginForm, Section, SwipeView,
};
use crate::test_support::{card, deck, deck_profile};

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

fn color_map() -> zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary {
    serde_json::from_value(serde_json::json!({
        "deck_count": 2,
        "color_map": {
            "colors": [{"color": "G", "cards": 30}, {"color": "U", "cards": 12}],
            "colorless_cards": 4,
            "pairs": [{"colors": ["U", "G"], "decks": 2}],
        },
    }))
    .unwrap()
}

#[test]
fn home_shows_the_counters_color_map_and_flavor() {
    let mut flavor = card("Llanowar Elves");
    flavor.scryfall_data.flavor_text = Some("One bone broken for every twig snapped.".to_owned());
    let mut app = App::new(Palette::terminal());
    app.username = Some("scotty".to_owned());
    app.screen = Screen::Home(Box::new(HomeView {
        everybody: Some(Figures {
            decks_created: 1_234,
            searches: 56_789,
            cards_swiped: 3,
        }),
        play_style: Some(color_map()),
        flavor: Some(flavor),
        ..HomeView::default()
    }));
    let screen = text(&app);
    for expected in [
        "Home",
        "1,234 Decks created",
        "56,789 Searches run",
        "3 Cards swiped",
        "Everybody",
        "Color map",
        "Most shared: UG 2",
        "Featured flavor",
        "One bone broken",
        "Llanowar Elves",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn home_without_decks_asks_for_one_and_hides_a_card_without_flavor() {
    let mut empty = color_map();
    empty.deck_count = 0;
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Home(Box::new(HomeView {
        play_style: Some(empty),
        flavor: Some(card("No flavor")),
        ..HomeView::default()
    }));
    let screen = text(&app);
    assert!(screen.contains("Build a deck to see your playstyle"));
    assert!(screen.contains("Counting..."));
    assert!(!screen.contains("Featured flavor"));
}

fn loaded_deck_screen(section: Section) -> App {
    let creature = |name: &str, cost: &str, cmc: f64| {
        let mut c = card(name);
        c.scryfall_data.mana_cost = Some(cost.to_owned());
        c.scryfall_data.cmc = Some(cmc);
        c.scryfall_data.type_line = Some("Creature — Elf".to_owned());
        c
    };
    let mut forest = card("Forest");
    forest.scryfall_data.type_line = Some("Basic Land — Forest".to_owned());
    let mut profile = deck_profile("Elves");
    profile.land_target = Some(36);
    let mut view = DeckView::opening(profile.clone());
    view.load(deck(
        profile,
        vec![
            creature("A", "{G}", 1.0),
            creature("B", "{1}{G}", 2.0),
            forest,
        ],
    ));
    view.section = section;
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Deck(Box::new(view));
    app
}

#[test]
fn the_deck_screen_shows_the_profile_and_tabs() {
    let screen = text(&loaded_deck_screen(Section::Budget));
    for expected in [
        "Elves",
        "Profile",
        "Cards",
        "Budget",
        "Distributions",
        "Mana",
        "Draw odds",
        "USD",
        "Total price",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_mana_tab_shows_lands_against_the_target_and_the_charts() {
    let screen = text(&loaded_deck_screen(Section::Mana));
    for expected in [
        "1 / 36",
        "Average mana value",
        "Mana curve",
        "Colors",
        "Mana cost fulfillment",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn the_draw_odds_tab_counts_from_the_opening_hand() {
    let screen = text(&loaded_deck_screen(Section::DrawOdds));
    assert!(screen.contains("Odds of ≥1 in opening hand"));
    assert!(screen.contains("LAND"));
    assert!(
        screen.contains("100%"),
        "every land is drawn when the hand is bigger than the deck"
    );
}

#[test]
fn the_distributions_tab_shows_types_and_average_stats() {
    let screen = text(&loaded_deck_screen(Section::Distributions));
    assert!(screen.contains("Average P/T"));
    assert!(screen.contains("Types"));
    assert!(screen.contains("creat"));
}

#[test]
fn new_screens_fit_a_tiny_terminal() {
    let mut app = loaded_deck_screen(Section::Mana);
    render(&app, 12, 5);
    app.screen = Screen::Home(Box::new(HomeView {
        play_style: Some(color_map()),
        ..HomeView::default()
    }));
    render(&app, 12, 5);
    render(&app, 40, 12);
}

#[test]
fn a_loaded_image_draws_beside_the_card_text_on_a_wide_terminal() {
    use std::cell::RefCell;

    use image::DynamicImage;
    use ratatui_image::picker::Picker;

    use crate::inbound::app::CardImage;

    let elves = card("Llanowar Elves");
    let id = elves.scryfall_data.id;
    let mut app = swipe_app(elves);
    let picker = Picker::halfblocks();
    let red = image::RgbImage::from_pixel(40, 56, image::Rgb([255, 0, 0]));
    let protocol = picker.new_resize_protocol(DynamicImage::ImageRgb8(red));
    app.images = Some(picker);
    if let Screen::Swipe(view) = &mut app.screen {
        view.images
            .insert(id, CardImage::Ready(Box::new(RefCell::new(protocol))));
    }
    let terminal = render(&app, 160, 40);
    let buffer = terminal.backend().buffer();
    let screen: String = buffer
        .content()
        .iter()
        .map(ratatui::buffer::Cell::symbol)
        .collect();
    assert!(screen.contains("Llanowar Elves"), "text beside the image");
    let red = Color::Rgb(255, 0, 0);
    assert!(
        buffer.content().iter().any(|c| c.fg == red || c.bg == red),
        "the image's pixels are drawn"
    );

    let narrow = text(&app);
    assert!(
        narrow.contains("Llanowar Elves"),
        "a narrow terminal shows the text first"
    );

    if let Screen::Swipe(view) = &mut app.screen {
        view.show_image = true;
        view.images.insert(id, CardImage::Loading);
    }
    let narrow_image = text(&app);
    assert!(narrow_image.contains("Loading image..."));
    assert!(
        !narrow_image.contains("Llanowar Elves"),
        "i swaps the text for the image"
    );
}

fn account_user(verified: bool) -> zwipe_core::domain::user::User {
    let mut user: zwipe_core::domain::user::User = serde_json::from_value(serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "username": "scotty",
        "email": "scotty@zwipe.net",
        "email_verified_at": null,
        "hints_shown": {},
    }))
    .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
    }
    user
}

#[test]
fn the_account_screen_shows_details_and_verification_rows() {
    use crate::inbound::app::AccountView;

    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Account(Box::new(AccountView::new(account_user(false))));
    let screen = text(&app);
    for expected in [
        "> Username",
        "scotty",
        "scotty@zwipe.net",
        "Unverified",
        "Verification",
        "Resend",
        "Check",
        "•••••••",
        "Version",
        "Log out",
        "Delete account",
    ] {
        assert!(screen.contains(expected), "missing {expected}");
    }
}

#[test]
fn a_form_masks_passwords_and_the_delete_button_counts_down() {
    use crate::inbound::app::{AccountView, Dialog, Form, FormKind};

    let mut view = AccountView::new(account_user(true));
    let mut form = Form::new(FormKind::Delete);
    form.values = vec!["hunter2".to_owned()];
    view.dialog = Some(Dialog::Form(form));
    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Account(Box::new(view));
    let screen = text(&app);
    assert!(screen.contains("Delete account"));
    assert!(screen.contains("permanently delete"));
    assert!(screen.contains("•••••••"));
    assert!(!screen.contains("hunter2"));
    assert!(screen.contains("Delete (5)"));
}

#[test]
fn the_config_screen_lines_up_every_row_and_marks_the_selection() {
    use zwipe_core::domain::user::preferences::UserPreferences;

    use crate::inbound::app::ConfigView;

    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Config(Box::new(ConfigView {
        saved: Some(UserPreferences::default()),
        shown: Some(UserPreferences::default()),
        ..ConfigView::default()
    }));
    let screen = text(&app);
    for expected in [
        "> Theme",
        "Gruvbox",
        "  Mode",
        "Dark",
        "  Pips",
        "  Universes Beyond",
        "Show",
        "  Exceptions",
    ] {
        assert!(screen.contains(expected), "missing {expected:?}");
    }
}

#[test]
fn the_theme_list_shows_sections_and_names() {
    use zwipe_core::domain::user::preferences::UserPreferences;

    use crate::inbound::app::{ConfigView, ThemePicker};

    let mut app = App::new(Palette::terminal());
    app.screen = Screen::Config(Box::new(ConfigView {
        saved: Some(UserPreferences::default()),
        shown: Some(UserPreferences::default()),
        picker: Some(ThemePicker::opening("gruvbox")),
        ..ConfigView::default()
    }));
    let screen = text(&app);
    assert!(screen.contains("Themes"));
    assert!(screen.contains("Warm & earthy"));
    assert!(screen.contains("> Gruvbox"));
}
