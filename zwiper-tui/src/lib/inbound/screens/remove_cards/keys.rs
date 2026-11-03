//! Remove's keys: h keeps (its label reads skip), l removes, m moves between boards, u undoes (or the arrows), 1 to 4 pick the board, f filters, r refreshes, e opens details, t flips, ? shows the hint, Esc goes back.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::HINT_REMOVE_DECK_CARDS;
use zwipe_ui::screens::remove_cards::{Action, BoardFilter, Status};
use zwipe_ui::stack::Direction;

use super::{RemoveView, dispatch};
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
    view.entering = None;
    match code {
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Left | KeyCode::Char('h') => swipe(view, Direction::Left, Action::Keep, cx),
        KeyCode::Right | KeyCode::Char('l') => swipe(view, Direction::Right, Action::Remove, cx),
        KeyCode::Up | KeyCode::Char('m') => swipe(view, Direction::Up, Action::Move, cx),
        KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace => {
            dispatch(view, Action::Undo, cx);
        }
        KeyCode::Char(c @ '1'..='4') => {
            let at = usize::from(u8::try_from(c).unwrap_or(b'1').saturating_sub(b'1'));
            if let Some(board) = BoardFilter::ALL.get(at) {
                view.motion = None;
                dispatch(view, Action::PickBoard(*board), cx);
            }
        }
        KeyCode::Char('f') if matches!(view.model.status(), Status::Empty | Status::Loaded) => {
            view.open_filter(cx);
        }
        KeyCode::Char('r') => dispatch(view, Action::Refresh, cx),
        KeyCode::Char('e') => {
            if let Some(card) = view.model.current().cloned() {
                view.details = Some(DetailsSheet::new(card, /* printings */ true));
            }
        }
        KeyCode::Char('t') => view.face = view.face.saturating_add(1),
        KeyCode::Char('?') => {
            hint::reopen(cx, HINT_REMOVE_DECK_CARDS);
        }
        _ => {}
    }
}

/// A swipe: the top card flies off by `direction` while it is still the top one, then the model acts.
fn swipe(view: &mut RemoveView, direction: Direction, action: Action, cx: &mut Context<'_>) {
    view.exit(direction);
    dispatch(view, action, cx);
}

/// Hands the key to an open sheet. Returns whether one took it.
fn overlay_key(view: &mut RemoveView, code: KeyCode, cx: &mut Context<'_>) -> bool {
    if let Some(sheet) = &mut view.sheet {
        match card_sheets::filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => view.sheet = None,
            SheetOutcome::Applied(filter) => {
                view.sheet = None;
                view.motion = None;
                dispatch(view, Action::Filter(*filter), cx);
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
        match details.handle_key(code, cx) {
            DetailsOutcome::Open => {}
            DetailsOutcome::Closed => view.details = None,
            DetailsOutcome::Printings => {
                let card = details.card.clone();
                view.printing = Some(PrintingSheet::open(
                    cx,
                    card,
                    /* read_only */ true,
                    "Printing saved",
                ));
            }
        }
        return true;
    }
    false
}
