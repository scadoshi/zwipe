//! The commander swipe's keys, the deck swipe's convention: Right or `l` picks the card on top, Up or `k` saves it, Left or `h` skips it, Down, `j` or `u` undoes the last swipe. `f` opens the Suggested Filter, `r` refreshes the stack, `?` shows the hint, Esc closes.
//!
//! In the Filter sheet h/l and the arrows move the chip cursor, Enter or Space picks it (the picked one again is All), and Esc closes the sheet.

use crossterm::event::KeyCode;

use super::{CommanderSwipeView, Filter, reopen_hint};
use crate::inbound::app::Context;

pub(crate) fn handle(view: &mut CommanderSwipeView, code: KeyCode, cx: &mut Context<'_>) {
    if view.filter.is_some() {
        let count = view.identities().len().saturating_add(1);
        let Some(filter) = &mut view.filter else {
            return;
        };
        match code {
            KeyCode::Esc | KeyCode::Char('f' | 'q') => view.filter = None,
            KeyCode::Left | KeyCode::Char('h') => filter.cursor = filter.cursor.saturating_sub(1),
            KeyCode::Right | KeyCode::Char('l') => {
                filter.cursor = filter.cursor.saturating_add(1).min(count.saturating_sub(1));
            }
            KeyCode::Enter | KeyCode::Char(' ') => {
                let cursor = filter.cursor;
                view.pick_identity(cursor);
            }
            _ => {}
        }
        return;
    }
    match code {
        KeyCode::Esc | KeyCode::Char('q') => view.close(cx),
        KeyCode::Right | KeyCode::Char('l') => view.select(cx),
        KeyCode::Up | KeyCode::Char('k') => view.save(cx),
        KeyCode::Left | KeyCode::Char('h') => view.skip(cx),
        KeyCode::Down | KeyCode::Char('j' | 'u') => view.undo(cx),
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('?') => reopen_hint(view, cx),
        KeyCode::Char('f') => {
            if let Some(pile) = view.suggested() {
                let cursor = pile
                    .identity
                    .as_ref()
                    .and_then(|picked| view.identities().iter().position(|i| i == picked))
                    .map_or(0, |i| i.saturating_add(1));
                view.filter = Some(Filter { cursor });
            }
        }
        _ => {}
    }
}
