//! The deck screen's keys: tabs, currency, draw-odds turn, reload, and starting to swipe.

use crossterm::event::KeyCode;

use super::{DeckView, Step};
use crate::inbound::app::Context;
use crate::inbound::screens::swipe;

pub(crate) fn handle(view: &mut DeckView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Tab | KeyCode::Right | KeyCode::Char('l') => view.step_section(Step::Next),
        KeyCode::BackTab | KeyCode::Left | KeyCode::Char('h') => {
            view.step_section(Step::Previous);
        }
        KeyCode::Char('c') => view.next_currency(),
        KeyCode::Char('+' | '=') => view.later_turn(),
        KeyCode::Char('-') => view.earlier_turn(),
        KeyCode::Char('r') => view.reload(cx),
        KeyCode::Char('s') => {
            // The deck view reloads when the swipe screen closes, so it hands its copy over.
            if let Some(deck) = view.deck.take() {
                view.metrics = None;
                swipe::open(cx, &deck);
            }
        }
        _ => {}
    }
}
