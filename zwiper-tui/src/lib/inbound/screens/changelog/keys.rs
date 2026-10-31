//! Changelog's keys: h/l across a chip row, Tab between rows, Enter to press a chip, j/k to scroll the cards.

use crossterm::event::KeyCode;

use super::ChangelogView;
use crate::domain::changelog::{self, ChangelogCache};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut ChangelogView, code: KeyCode, cx: &mut Context<'_>) {
    let ChangelogCache::Loaded(log) = &cx.shared.changelog else {
        match code {
            KeyCode::Esc => cx.back(),
            KeyCode::Char('q') => cx.quit(),
            KeyCode::Char('!') => help::open(cx),
            _ => {}
        }
        return;
    };
    let listed = changelog::listed(log);
    let rows = view.rows(&listed);
    let width = rows.get(view.row).map_or(0, Vec::len);
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Left | KeyCode::Char('h') => view.chip = view.chip.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => {
            view.chip = view.chip.saturating_add(1).min(width.saturating_sub(1));
        }
        KeyCode::Tab => {
            view.row = view.row.saturating_add(1).min(rows.len().saturating_sub(1));
            view.chip = 0;
        }
        KeyCode::BackTab => {
            view.row = view.row.saturating_sub(1);
            view.chip = 0;
        }
        KeyCode::Enter | KeyCode::Char(' ') => view.press(&listed),
        KeyCode::Down | KeyCode::Char('j') => view.scroll = view.scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => view.scroll = view.scroll.saturating_sub(1),
        KeyCode::PageDown => view.scroll = view.scroll.saturating_add(10),
        KeyCode::PageUp => view.scroll = view.scroll.saturating_sub(10),
        _ => {}
    }
}
