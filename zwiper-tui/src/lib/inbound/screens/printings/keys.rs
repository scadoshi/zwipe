//! The printings' keys: j/k or Up/Down move, Enter picks in pick mode, `i` shows the selected printing's image, Esc goes back.

use crossterm::event::KeyCode;
use zwipe_ui::screens::printing_sheet::Action;

use super::{PrintingsView, dispatch};
use crate::inbound::app::Context;
use crate::inbound::screens::image_preview;

pub(crate) fn handle(view: &mut PrintingsView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => {
            let at = view.selected;
            dispatch(view, Action::Back { at }, cx);
        }
        KeyCode::Down | KeyCode::Char('j') => view.next(),
        KeyCode::Up | KeyCode::Char('k') => view.previous(),
        KeyCode::Enter => {
            let at = view.selected;
            dispatch(view, Action::Save { at }, cx);
        }
        KeyCode::Char('i') => {
            if let Some(printing) = view.selected_printing() {
                image_preview::open(cx, printing.clone(), 0);
            }
        }
        _ => {}
    }
}
