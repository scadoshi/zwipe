//! The Add screen's keys. An open sheet takes every key; otherwise l, h, m and u (or the arrows) swipe right, left, up and undo, and the rest open sheets, switch the source, flip Synergy and refresh.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::HINT_ADD_DECK_CARDS;

use super::{HINT_LINES, HINT_TITLE, Source, SwipeView};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets;
use crate::inbound::screens::card_sheets::details::DetailsOutcome;
use crate::inbound::screens::card_sheets::filter::{self, SheetOutcome};
use crate::inbound::screens::card_sheets::printing::{PrintingOutcome, PrintingSheet};

pub(crate) fn handle(view: &mut SwipeView, code: KeyCode, cx: &mut Context<'_>) {
    if sheet_key(view, code, cx) {
        return;
    }
    view.stack.cancel_entering();
    view.mb_stack.cancel_entering();
    match code {
        KeyCode::Esc => view.leave(cx),
        KeyCode::Char('q') => {
            view.park(cx);
            cx.quit();
        }
        KeyCode::Right | KeyCode::Char('l') => view.swipe_right(cx),
        KeyCode::Left | KeyCode::Char('h') => view.swipe_left(cx),
        KeyCode::Up | KeyCode::Char('m') => view.swipe_up(cx),
        KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace => view.undo(cx),
        KeyCode::Char('f') => view.open_filter(cx),
        KeyCode::Char('r') => view.refresh(cx),
        KeyCode::Char('e') => view.open_details(),
        KeyCode::Char('y') => view.toggle_synergy(cx),
        KeyCode::Tab => {
            let to = match view.source {
                Source::Search => Source::Maybeboard,
                Source::Maybeboard => Source::Search,
            };
            view.switch_source(cx, to);
        }
        KeyCode::Char('t') => {
            view.face = view.face.saturating_add(1);
            view.sync_images(cx);
        }
        KeyCode::Char('i') => view.show_image = !view.show_image,
        KeyCode::Char('?') => {
            card_sheets::show_hint(cx, HINT_ADD_DECK_CARDS, HINT_TITLE, &HINT_LINES);
        }
        _ => {}
    }
}

/// Hands the key to the sheet on top, if any. Returns whether one took it.
fn sheet_key(view: &mut SwipeView, code: KeyCode, cx: &mut Context<'_>) -> bool {
    if let Some(sheet) = &mut view.sheet {
        match filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => view.sheet = None,
            SheetOutcome::Applied(filter) => {
                view.sheet = None;
                view.filter = *filter;
                view.rerun(cx);
            }
        }
        return true;
    }
    if let Some(printing) = &mut view.printing {
        match printing.handle_key(code, cx) {
            PrintingOutcome::Open => {}
            PrintingOutcome::Closed => view.printing = None,
            PrintingOutcome::Saved(card) => {
                view.printing = None;
                view.details = None;
                view.stack.replace_current(*card);
                view.face = 0;
                view.sync_images(cx);
            }
        }
        return true;
    }
    if let Some(details) = &mut view.details {
        match details.handle_key(code, &cx.shared.palette) {
            DetailsOutcome::Open => {}
            DetailsOutcome::Closed => view.details = None,
            DetailsOutcome::Printings => {
                let card = details.card.clone();
                view.printing = Some(PrintingSheet::open(cx, card, false, "Printing swapped"));
            }
        }
        return true;
    }
    false
}
