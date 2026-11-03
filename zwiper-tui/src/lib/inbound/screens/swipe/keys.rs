//! The Add screen's keys. An open sheet takes every key; otherwise l, h, m and u (or the arrows) swipe right, left, up and undo, and the rest open sheets, switch the source, flip Synergy and refresh.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_ui::screens::add_cards::{Action, PRINTING_SWAPPED, Source};
use zwipe_ui::stack::Direction;

use super::{Entering, SwipeView, dispatch, reopen_hint, scope};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets::details::DetailsOutcome;
use crate::inbound::screens::card_sheets::filter::{self, SheetOutcome};
use crate::inbound::screens::card_sheets::printing::{PrintingOutcome, PrintingSheet};
use crate::inbound::widgets::card_motion::CardMotion;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut SwipeView, code: KeyCode, cx: &mut Context<'_>) {
    if sheet_key(view, code, cx) {
        return;
    }
    view.entering = Entering::default();
    match code {
        KeyCode::Esc => {
            view.park(cx);
            dispatch(view, Action::Back, cx);
        }
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => {
            view.park(cx);
            cx.quit();
        }
        KeyCode::Right | KeyCode::Char('l') => swipe(view, Direction::Right, cx),
        KeyCode::Left | KeyCode::Char('h') => swipe(view, Direction::Left, cx),
        KeyCode::Up | KeyCode::Char('m') => swipe(view, Direction::Up, cx),
        KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace => undo(view, cx),
        KeyCode::Char('f') => view.open_filter(cx),
        KeyCode::Char('r') => dispatch(view, Action::Refresh, cx),
        KeyCode::Char('e') => view.open_details(),
        KeyCode::Char('y') if view.model.shows_synergy() => {
            dispatch(view, Action::ToggleSynergy, cx);
        }
        KeyCode::Tab => switch_source(view, cx),
        KeyCode::Char('t') => {
            view.face = view.face.saturating_add(1);
            view.sync_images(cx);
        }
        KeyCode::Char('i') => view.show_image = !view.show_image,
        KeyCode::Char('?') => reopen_hint(cx),
        _ => {}
    }
}

/// Commits the top card's exit to `direction`, starting its motion while it is still on top.
fn swipe(view: &mut SwipeView, direction: Direction, cx: &mut Context<'_>) {
    view.exit(direction);
    dispatch(view, Action::Swipe(direction), cx);
}

/// Undoes the newest swipe, bringing its card back from the side it left by.
fn undo(view: &mut SwipeView, cx: &mut Context<'_>) {
    dispatch(view, Action::Undo, cx);
    if let Some(side) = view.entering() {
        view.motion = Some(CardMotion::enter(side, Instant::now()));
    }
}

/// Switches between Search and Maybeboard, each keeping its own filter.
fn switch_source(view: &mut SwipeView, cx: &mut Context<'_>) {
    let Some(deck_id) = view.model.deck_id() else {
        return;
    };
    let source = match view.model.source() {
        Source::Search => Source::Maybeboard,
        Source::Maybeboard => Source::Search,
    };
    let remembered = cx.shared.cards.filters.restore(scope(source), deck_id);
    view.motion = None;
    dispatch(view, Action::PickSource { source, remembered }, cx);
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
                dispatch(view, Action::Filter(view.filter.clone()), cx);
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
                dispatch(view, Action::SwapPrinting(card), cx);
            }
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
                    /* read_only */ false,
                    PRINTING_SWAPPED,
                ));
            }
        }
        return true;
    }
    false
}
