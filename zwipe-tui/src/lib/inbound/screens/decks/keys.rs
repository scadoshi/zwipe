//! The deck list's keys: move, open a deck, reload, sign out.

use crossterm::event::KeyCode;

use super::DeckList;
use crate::inbound::app::Context;
use crate::inbound::screens::{self, deck};

pub(crate) fn handle(list: &mut DeckList, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Down | KeyCode::Char('j') => list.next(),
        KeyCode::Up | KeyCode::Char('k') => list.previous(),
        KeyCode::Char('r') => list.reload(cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        KeyCode::Enter => {
            if let Some(profile) = list.selected_deck().cloned() {
                deck::open(cx, profile);
            }
        }
        _ => {}
    }
}
