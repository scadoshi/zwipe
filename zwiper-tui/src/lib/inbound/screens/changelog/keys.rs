//! Changelog's keys: h/l across a chip row, Tab between rows, Enter to press a chip, j/k to scroll the cards. Until the history loads, only Esc, `q` and `!` do anything.

use crossterm::event::KeyCode;
use zwipe_ui::screens::changelog::{Action, Status};

use super::{ChangelogView, dispatch};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut ChangelogView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => return dispatch(view, Action::Back, cx),
        KeyCode::Char('q') => return cx.quit(),
        KeyCode::Char('!') => return help::open(cx),
        _ => {}
    }
    if view.model.status() != Status::Loaded {
        return;
    }
    let rows = view.model.chip_rows();
    let width = rows.get(view.row).map_or(0, Vec::len);
    match code {
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
        KeyCode::Enter | KeyCode::Char(' ') => view.press(cx),
        KeyCode::Down | KeyCode::Char('j') => view.scroll = view.scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => view.scroll = view.scroll.saturating_sub(1),
        KeyCode::PageDown => view.scroll = view.scroll.saturating_add(10),
        KeyCode::PageUp => view.scroll = view.scroll.saturating_sub(10),
        _ => {}
    }
}
