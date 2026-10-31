//! The preview's keys: `f` flips a multi-faced card, Esc or q closes it.

use crossterm::event::KeyCode;

use super::ImagePreview;
use crate::inbound::app::Context;

pub(crate) fn handle(view: &mut ImagePreview, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc | KeyCode::Char('q') => cx.back(),
        KeyCode::Char('f') => view.flip(cx),
        _ => {}
    }
}
