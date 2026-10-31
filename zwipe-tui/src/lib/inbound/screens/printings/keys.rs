//! The printings' keys: j/k or Up/Down move, Enter picks in pick mode, `i` shows the selected printing's image, Esc goes back.

use crossterm::event::KeyCode;

use super::PrintingsView;
use crate::inbound::app::Context;
use crate::inbound::screens::image_preview;

pub(crate) fn handle(view: &mut PrintingsView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => view.close(cx),
        KeyCode::Down | KeyCode::Char('j') => view.next(),
        KeyCode::Up | KeyCode::Char('k') => view.previous(),
        KeyCode::Enter => view.pick(cx),
        KeyCode::Char('i') => {
            if let Some(printing) = view.selected_printing() {
                image_preview::open(cx, printing.clone(), 0);
            }
        }
        _ => {}
    }
}
