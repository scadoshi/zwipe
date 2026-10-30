//! The update notice's keys: only quitting.

use crossterm::event::KeyCode;

use crate::inbound::app::Context;

pub(crate) fn handle(code: KeyCode, cx: &mut Context<'_>) {
    if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
        cx.quit();
    }
}
