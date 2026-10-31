//! Remove's keys: h keeps, l removes, m moves between boards, u undoes (or the arrows), 1 to 4 pick the board, f filters, r refreshes, e opens details, t flips, ? shows the hint, Esc goes back.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::HINT_REMOVE_DECK_CARDS;

use super::{BoardFilter, RemoveView};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets;
use crate::inbound::screens::card_sheets::details::{DetailsOutcome, DetailsSheet};
use crate::inbound::screens::card_sheets::filter::SheetOutcome;
use crate::inbound::screens::card_sheets::printing::{PrintingOutcome, PrintingSheet};
use crate::inbound::widgets::{help, hint};

pub(crate) fn handle(view: &mut RemoveView, code: KeyCode, cx: &mut Context<'_>) {
    if overlay_key(view, code, cx) {
        return;
    }
    view.stack.cancel_entering();
    match code {
        KeyCode::Esc => view.leave(cx),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Left | KeyCode::Char('h') => view.keep(cx),
        KeyCode::Right | KeyCode::Char('l') => view.remove(cx),
        KeyCode::Up | KeyCode::Char('m') => view.move_board(cx),
        KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace => view.undo(cx),
        KeyCode::Char(c @ '1'..='4') => {
            let at = usize::from(u8::try_from(c).unwrap_or(b'1').saturating_sub(b'1'));
            if let Some(board) = BoardFilter::ALL.get(at) {
                view.set_board(*board);
            }
        }
        KeyCode::Char('f') if view.loaded => view.open_filter(cx),
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('e') => {
            if let Some(card) = view.current().cloned() {
                view.details = Some(DetailsSheet::new(card, true));
            }
        }
        KeyCode::Char('t') => view.face = view.face.saturating_add(1),
        KeyCode::Char('?') => {
            hint::reopen(cx, HINT_REMOVE_DECK_CARDS);
        }
        _ => {}
    }
}

/// Hands the key to an open sheet. Returns whether one took it.
fn overlay_key(view: &mut RemoveView, code: KeyCode, cx: &mut Context<'_>) -> bool {
    if let Some(sheet) = &mut view.sheet {
        match card_sheets::filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => view.sheet = None,
            SheetOutcome::Applied(filter) => {
                view.sheet = None;
                view.filter = *filter;
                view.rerun();
            }
        }
        return true;
    }
    if let Some(printing) = &mut view.printing {
        if printing.handle_key(code, cx) != PrintingOutcome::Open {
            view.printing = None;
        }
        return true;
    }
    if let Some(details) = &mut view.details {
        match details.handle_key(code, &cx.shared.palette) {
            DetailsOutcome::Open => {}
            DetailsOutcome::Closed => view.details = None,
            DetailsOutcome::Printings => {
                let card = details.card.clone();
                view.printing = Some(PrintingSheet::open(cx, card, true, "Printing saved"));
            }
        }
        return true;
    }
    false
}
