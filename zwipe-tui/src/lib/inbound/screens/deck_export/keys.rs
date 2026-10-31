//! Export's keys: h/l and Space for the board chips, j/k to scroll, y to copy, Esc to go back.

use crossterm::event::KeyCode;

use super::{ExportView, reopen_hint};
use crate::inbound::app::Context;

/// The last board chip's index.
const LAST_CHIP: usize = 2;

pub(crate) fn handle(view: &mut ExportView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('y' | 'c') => view.copy(cx),
        KeyCode::Left | KeyCode::Char('h') => view.focus = view.focus.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => {
            view.focus = view.focus.saturating_add(1).min(LAST_CHIP);
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            if view.deck.is_some() {
                view.toggle();
            }
        }
        KeyCode::Down | KeyCode::Char('j') => view.scroll = view.scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => view.scroll = view.scroll.saturating_sub(1),
        _ => {}
    }
}
