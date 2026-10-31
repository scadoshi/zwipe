//! The examples' keys: Left or h for the next card, Down or j to step back one, `d` for the details sheet, `?` for the hint, Esc to close.
//!
//! While the details sheet is open it takes the keys: `p` opens the card's printings read only, Esc or q closes it, and the rest scroll it.

use crossterm::event::KeyCode;

use super::{ExamplesView, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::screens::printings::{self, Mode};
use crate::inbound::widgets::help;
use crate::inbound::widgets::sheet::SheetKey;

pub(crate) fn handle(view: &mut ExamplesView, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(sheet) = &mut view.details {
        if code == KeyCode::Char('p') {
            if let Some(card) = view.current() {
                printings::open(cx, card.clone(), Mode::ReadOnly);
            }
        } else if sheet.handle_key(code) == SheetKey::Close {
            view.details = None;
        }
        return;
    }
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Left | KeyCode::Char('h') => view.next(cx),
        KeyCode::Down | KeyCode::Char('j') => view.back_one(cx),
        KeyCode::Char('d') => view.open_details(&cx.shared.palette),
        KeyCode::Char('?') => reopen_hint(cx),
        _ => {}
    }
}
