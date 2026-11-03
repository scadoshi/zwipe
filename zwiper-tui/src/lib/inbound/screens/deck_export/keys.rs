//! Export's keys: h/l and Space for the board chips, j/k to scroll, y to copy, Esc to go back.

use crossterm::event::KeyCode;
use zwipe_ui::screens::deck_export::{Action, Board, Status};

use super::{ExportView, dispatch, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut ExportView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('y' | 'c') => dispatch(view, Action::Copy, cx),
        KeyCode::Left | KeyCode::Char('h') => view.focus = view.focus.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => {
            view.focus = view
                .focus
                .saturating_add(1)
                .min(Board::ALL.len().saturating_sub(1));
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            // The chips draw only over a loaded decklist.
            if view.model.status() == Status::Loaded
                && let Some(board) = Board::ALL.get(view.focus)
            {
                dispatch(view, Action::Toggle(*board), cx);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => view.scroll = view.scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => view.scroll = view.scroll.saturating_sub(1),
        _ => {}
    }
}
