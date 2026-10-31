//! Privacy policy's keys: j/k and the page keys scroll, Esc goes back.

use crossterm::event::KeyCode;

use super::PrivacyView;
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut PrivacyView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Down | KeyCode::Char('j') => view.scroll = view.scroll.saturating_add(1),
        KeyCode::Up | KeyCode::Char('k') => view.scroll = view.scroll.saturating_sub(1),
        KeyCode::PageDown | KeyCode::Char(' ') => view.scroll = view.scroll.saturating_add(10),
        KeyCode::PageUp => view.scroll = view.scroll.saturating_sub(10),
        KeyCode::Home => view.scroll = 0,
        _ => {}
    }
}
