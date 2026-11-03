//! Builders for tests, starting from one real card served by the API.

use ratatui::Terminal;
use ratatui::backend::TestBackend;
use ratatui::buffer::Cell;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, DeckEntry, DeckProfile, Quantity};
use zwipe_core::domain::user::User;
use zwipe_ui::generation::{Generation, Ticket};

use crate::inbound::app::App;

/// A one-request HTTP server on localhost.
pub(crate) mod server;

const CARD_JSON: &str = include_str!("card.json");

/// A real card renamed to `name`, with fresh printing and oracle IDs, no faces, and no cost, rules, stats or flavor text.
pub(crate) fn card(name: &str) -> Card {
    let mut card: Card = serde_json::from_str(CARD_JSON).unwrap();
    let id = Uuid::new_v4();
    card.card_profile.scryfall_data_id = id;
    let data = &mut card.scryfall_data;
    data.id = id;
    data.oracle_id = Some(Uuid::new_v4());
    data.name = name.to_owned();
    data.card_faces = None;
    data.mana_cost = None;
    data.type_line = None;
    data.oracle_text = None;
    data.power = None;
    data.toughness = None;
    data.loyalty = None;
    data.defense = None;
    data.flavor_text = None;
    card
}

/// An empty deck named `name` with no commander and no format.
pub(crate) fn deck_profile(name: &str) -> DeckProfile {
    serde_json::from_value(serde_json::json!({
        "id": Uuid::new_v4(),
        "name": name,
        "commander_id": null,
        "partner_commander_id": null,
        "background_id": null,
        "signature_spell_id": null,
        "format": null,
        "land_target": null,
        "price_target": null,
        "price_target_currency": null,
        "user_id": Uuid::new_v4(),
        "card_count": 0,
        "commander_name": null,
        "partner_commander_name": null,
        "background_name": null,
        "signature_spell_name": null,
    }))
    .unwrap()
}

/// A deck holding one copy of each card on the main board.
pub(crate) fn deck(deck_profile: DeckProfile, cards: Vec<Card>) -> Deck {
    let deck_id = deck_profile.id;
    let entries = cards
        .into_iter()
        .map(|card| DeckEntry {
            deck_card: DeckCard {
                deck_id,
                scryfall_data_id: card.scryfall_data.id,
                oracle_id: card.scryfall_data.oracle_id.unwrap(),
                quantity: Quantity::new(1).unwrap(),
                board: Board::default(),
                mvp_at: None,
            },
            card,
        })
        .collect();
    Deck {
        deck_profile,
        entries,
        warnings: Vec::new(),
        command_zone_cards: Vec::new(),
    }
}

/// The commander Create opened with, when the screen on top is Create in the Commander format.
pub(crate) fn create_opened_with(
    harness: &crate::inbound::event_loop::harness::Harness,
) -> Option<&str> {
    use zwipe_core::domain::deck::Format;
    use zwipe_ui::screens::deck_form::Mode;

    use crate::inbound::screens::Screen;

    match harness.screen() {
        Screen::DeckForm(form)
            if form.model.mode() == Mode::Create
                && form.model.values().format == Some(Format::Commander) =>
        {
            form.model
                .values()
                .commander
                .as_ref()
                .map(|c| c.scryfall_data.name.as_str())
        }
        _ => None,
    }
}

/// A user named scotty, with the email verified or not, who has seen home's welcome hint.
pub(crate) fn user(verified: bool) -> User {
    let mut user: User = serde_json::from_value(serde_json::json!({
        "id": "00000000-0000-0000-0000-000000000001",
        "username": "scotty",
        "email": "scotty@zwipe.net",
        "email_verified_at": null,
        "hints_shown": {"first_login": true},
    }))
    .unwrap();
    if verified {
        user.email_verified_at = Some(chrono::Utc::now());
    }
    user
}

/// A session for `user` in `preferences`, its tokens good for decades.
pub(crate) fn session(
    user: &User,
    preferences: &zwipe_core::domain::user::UserPreferences,
) -> zwipe_core::domain::auth::models::session::Session {
    serde_json::from_value(serde_json::json!({
        "user": user,
        "access_token": {"value": "aGVhZA.cGF5bG9hZA.c2ln", "expires_at": "2099-01-01T00:00:00Z"},
        "refresh_token": {"value": "r", "expires_at": "2099-01-01T00:00:00Z"},
        "preferences": preferences,
    }))
    .unwrap()
}

/// `app` drawn into a test terminal of the given size.
pub(crate) fn render(app: &App, width: u16, height: u16) -> Terminal<TestBackend> {
    let mut terminal = Terminal::new(TestBackend::new(width, height)).unwrap();
    terminal.draw(|f| app.draw(f)).unwrap();
    terminal
}

/// The foreground color of each of `words` in the first row of `app`, drawn at 100 by 40, that holds `row`; panics when no row does.
pub(crate) fn word_colors(
    app: &App,
    row: &str,
    words: &[&str],
) -> Vec<Option<ratatui::style::Color>> {
    let terminal = render(app, 100, 40);
    let buffer = terminal.backend().buffer();
    let rows: Vec<Vec<&Cell>> = (0..40)
        .map(|y| (0..100).map(|x| buffer.cell((x, y)).unwrap()).collect())
        .collect();
    let text = |cells: &[&Cell]| cells.iter().map(|c| c.symbol()).collect::<String>();
    let line = rows
        .iter()
        .find(|cells| text(cells).contains(row))
        .unwrap_or_else(|| panic!("no row holds {row:?}"));
    words
        .iter()
        .map(|word| {
            let at = (0..line.len())
                .find(|&x| text(&line[x..]).starts_with(&format!(" {word} ")))
                .unwrap_or_else(|| panic!("{word:?} is not on the row"));
            Some(line[at.checked_add(1).unwrap()].fg)
        })
        .collect()
}

/// Every cell's symbol of `app` drawn at 100 by 40, row after row.
pub(crate) fn screen_text(app: &App) -> String {
    render(app, 100, 40)
        .backend()
        .buffer()
        .content()
        .iter()
        .map(Cell::symbol)
        .collect::<String>()
        .replace('\u{a0}', " ")
}

/// The sizes the button and label tests draw at: a roomy terminal and the two of [`NARROW`].
pub(crate) const BUTTON_SIZES: [(u16, u16); 3] = [(120, 40), (80, 24), (60, 20)];

/// Every place `text` starts in `buffer`, as its column and row, top to bottom.
pub(crate) fn find_all(buffer: &ratatui::buffer::Buffer, text: &str) -> Vec<(u16, u16)> {
    let area = buffer.area;
    let mut found = Vec::new();
    for y in 0..area.height {
        let row: Vec<&str> = (0..area.width).map(|x| buffer[(x, y)].symbol()).collect();
        let want: Vec<String> = text.chars().map(String::from).collect();
        for (x, window) in row.windows(want.len().max(1)).enumerate() {
            if window.iter().zip(&want).all(|(a, b)| a == b) {
                found.push((u16::try_from(x).unwrap(), y));
            }
        }
    }
    found
}

/// The dashes on each side of the label starting at `(x, y)` on a box's top border, as far as the corners.
pub(crate) fn label_margins(
    buffer: &ratatui::buffer::Buffer,
    (x, y): (u16, u16),
    label: &str,
) -> (u16, u16) {
    let width = u16::try_from(label.chars().count()).unwrap();
    let left = (0..x)
        .rev()
        .take_while(|&c| buffer[(c, y)].symbol() == "─")
        .count();
    let right = (x.saturating_add(width)..buffer.area.width)
        .take_while(|&c| buffer[(c, y)].symbol() == "─")
        .count();
    (u16::try_from(left).unwrap(), u16::try_from(right).unwrap())
}

/// The small terminal sizes every screen must fit: the common 80 by 24 and the narrowest supported, 60 by 20.
pub(crate) const NARROW: [(u16, u16); 2] = [(80, 24), (60, 20)];

/// The rows of `app` drawn at `width` by `height`, each as text.
pub(crate) fn rows(app: &App, width: u16, height: u16) -> Vec<String> {
    let terminal = render(app, width, height);
    let buffer = terminal.backend().buffer();
    (0..height)
        .map(|y| {
            (0..width)
                .map(|x| buffer.cell((x, y)).unwrap().symbol())
                .collect::<String>()
                .replace('\u{a0}', " ")
        })
        .collect()
}

/// Draws `app` at each [`NARROW`] size and asserts the key bar's last hint, `last` (like `"Esc back"`), shows whole in the bottom rows; prints the screen when it does not.
pub(crate) fn assert_key_bar_fits(app: &App, last: &str) {
    for (width, height) in NARROW {
        let rows = rows(app, width, height);
        let shown = rows.iter().rev().take(4).any(|row| row.contains(last));
        assert!(
            shown,
            "{last:?} is not in the key bar at {width}x{height}:\n{}",
            rows.join("\n")
        );
    }
}

/// A generation that has handed out `n` tickets, as a screen has after `n` searches.
pub(crate) fn generation_at(n: u64) -> Generation {
    let mut generation = Generation::default();
    for _ in 0..n {
        generation.next_ticket();
    }
    generation
}

/// The `n`th ticket a generation hands out; `ticket(0)` is the one before any.
pub(crate) fn ticket(n: u64) -> Ticket {
    generation_at(n).current()
}

/// Asserts that a bullet in `rows` wraps, and that no wrapped bullet's next row carries its sentence on left of its text, two columns right of the `•`; prints the rows when one does.
pub(crate) fn assert_bullets_hang(rows: &[String]) {
    let mut hung = 0_usize;
    for (y, row) in rows.iter().enumerate() {
        let Some(bullet) = row.chars().position(|c| c == '•') else {
            continue;
        };
        let Some(next) = rows
            .get(y.saturating_add(1))
            .filter(|next| !next.contains('•'))
        else {
            continue;
        };
        let under: Vec<char> = next.chars().skip(bullet).take(3).collect();
        // A lowercase letter is a sentence carrying on; a capital is a paragraph after the list.
        assert!(
            !under.first().is_some_and(|c| c.is_lowercase()),
            "the bullet on row {y} wraps to its margin:\n{}",
            rows.join("\n")
        );
        if under.get(2).is_some_and(|c| c.is_alphanumeric()) {
            hung = hung.saturating_add(1);
        }
    }
    assert!(hung > 0, "no bullet wraps:\n{}", rows.join("\n"));
}

/// Key hints whose key is `?`, which stay bare in key bars and sheet footers.
const QUESTION_KEYS: [&str; 3] = ["? hint", "? Hint", "? About"];

/// Every bare About marker in `rows` as its column and row: a `?` right after a space, outside the `?` key hints. A `?` closing a sentence touches its word and the `[?]` button opens with a bracket, so neither counts.
pub(crate) fn bare_about_markers(rows: &[String]) -> Vec<(usize, usize)> {
    let mut found = Vec::new();
    for (y, row) in rows.iter().enumerate() {
        let mut row = row.replace('\u{a0}', " ");
        for key in QUESTION_KEYS {
            row = row.replace(key, &" ".repeat(key.chars().count()));
        }
        let cells: Vec<char> = row.chars().collect();
        for (x, pair) in cells.windows(2).enumerate() {
            if pair == [' ', '?'] {
                found.push((x.saturating_add(1), y));
            }
        }
    }
    found
}

/// Asserts the `[?]` starting at `(x, y)` in `buffer` draws as the About button in `palette`: no cell reversed or filled with a marker color, brackets and `?` muted at rest, and brackets and `?` bold in the accent when `focused`.
pub(crate) fn assert_about_marker(
    buffer: &ratatui::buffer::Buffer,
    palette: &crate::domain::theme::Palette,
    (x, y): (u16, u16),
    focused: bool,
    context: &str,
) {
    use ratatui::style::Modifier;
    let symbols: String = (x..x.saturating_add(3))
        .map(|c| buffer[(c, y)].symbol())
        .collect();
    assert_eq!(symbols, "[?]", "{context}");
    let fills = [palette.accent.fg, palette.border.fg, palette.muted.fg];
    for c in x..x.saturating_add(3) {
        let cell = &buffer[(c, y)];
        assert!(
            !cell.modifier.contains(Modifier::REVERSED),
            "{context}: reversed at {c}"
        );
        assert!(!fills.contains(&Some(cell.bg)), "{context}: filled at {c}");
        if focused {
            assert_eq!(
                cell.fg,
                palette.accent.fg.unwrap(),
                "{context}: accent at {c}"
            );
            assert!(
                cell.modifier.contains(Modifier::BOLD),
                "{context}: bold at {c}"
            );
        }
    }
    if !focused {
        for c in x..x.saturating_add(3) {
            let cell = &buffer[(c, y)];
            assert_eq!(
                cell.fg,
                palette.muted.fg.unwrap(),
                "{context}: muted at {c}"
            );
            assert!(
                !cell.modifier.contains(Modifier::BOLD),
                "{context}: not bold at rest at {c}"
            );
        }
    }
}
