//! Draws whichever screen is up, in the account's theme.

mod account;
mod charts;
mod config;
mod deck;
mod decks;
mod home;
mod login;
mod swipe;
mod update;

use ratatui::Frame;
use ratatui::layout::{Constraint, Position, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Clear, Padding};

use super::app::{App, Screen};
use crate::domain::theme::Palette;

/// A border on each side.
const BORDERS: u16 = 2;

pub(crate) fn draw(f: &mut Frame<'_>, app: &App) {
    let palette = &app.palette;
    f.render_widget(Block::new().style(palette.base), f.area());
    match &app.screen {
        Screen::Login(form) => login::draw(f, palette, form),
        Screen::Home(view) => home::draw(f, palette, app, view),
        Screen::Decks(list) => decks::draw(f, palette, app, list),
        Screen::Deck(view) => deck::draw(f, palette, app, view),
        Screen::Swipe(view) => swipe::draw(f, palette, app, view),
        Screen::Account(view) => account::draw(f, palette, app, view),
        Screen::Config(view) => config::draw(f, palette, app, view),
        Screen::UpdateRequired => update::draw(f, palette),
    }
}

/// Clears and frames a centered panel of the given inner size, and returns the area inside its border and padding.
fn panel(f: &mut Frame<'_>, palette: &Palette, area: Rect, width: u16, height: u16) -> Rect {
    let outer = area.centered(
        Constraint::Max(width.saturating_add(BORDERS).saturating_add(BORDERS)),
        Constraint::Length(height.saturating_add(BORDERS)),
    );
    let block = Block::bordered()
        .border_style(palette.border)
        .style(palette.base)
        .padding(Padding::horizontal(1));
    let inner = block.inner(outer);
    f.render_widget(Clear, outer);
    f.render_widget(block, outer);
    inner
}

/// Key hints: each key in bold, its action muted.
fn hints(palette: &Palette, keys: &[(&str, &str)]) -> Line<'static> {
    let spans: Vec<Span<'static>> = keys
        .iter()
        .flat_map(|&(key, action)| {
            [
                key.to_owned().bold(),
                Span::styled(format!(" {action}  "), palette.muted),
            ]
        })
        .collect();
    Line::from(spans)
}

/// Where the cursor sits after `text` in a bordered one-line box.
fn cursor_after(text: &str, area: Rect) -> Position {
    let column = u16::try_from(text.chars().count()).unwrap_or(u16::MAX);
    Position {
        x: area
            .x
            .saturating_add(1)
            .saturating_add(column)
            .min(area.right().saturating_sub(BORDERS)),
        y: area.y.saturating_add(1),
    }
}

fn width_of(text: &str) -> u16 {
    u16::try_from(text.chars().count()).unwrap_or(u16::MAX)
}

fn height_of(lines: usize) -> u16 {
    u16::try_from(lines).unwrap_or(u16::MAX)
}

#[cfg(test)]
mod tests;
