//! The deck cards view's keys.
//!
//! j and k (or the arrows) move through the list; Enter opens a group or a row. On an open row: - and + change the quantity, p changes the printing, s stars, d, m and b move to the main board, maybeboard or sideboard, t flips and i previews. / types in quick add. g steps Group by, 1 2 3 toggle the boards, T C A toggle tokens, the command zone and art. f filters, x resets the filter, u undoes, a adds, r removes, ? shows the hint, Esc goes back.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::user::hints::HINT_DECK_CARDS;

use super::{DeckCardsView, Item, Kind, PrintingTarget, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets;
use crate::inbound::screens::card_sheets::filter::SheetOutcome;
use crate::inbound::screens::card_sheets::printing::PrintingOutcome;
use crate::inbound::widgets::{card_info, help, hint};

pub(crate) fn handle(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    if overlay_key(view, code, cx) {
        return;
    }
    if view.quick.focused {
        quick_key(view, code, cx);
        return;
    }
    match code {
        KeyCode::Esc => view.leave(cx),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => {
            view.flush_all(cx);
            cx.quit();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let last = view.items().len().saturating_sub(1);
            view.cursor = view.cursor.saturating_add(1).min(last);
        }
        KeyCode::Up | KeyCode::Char('k') => view.cursor = view.cursor.saturating_sub(1),
        KeyCode::Enter | KeyCode::Char(' ') => toggle(view),
        KeyCode::Char('/') if view.loaded => view.quick.focused = true,
        KeyCode::Char('g') if view.loaded => {
            let options = GroupByOption::all();
            let at = options
                .iter()
                .position(|o| *o == view.group_by)
                .unwrap_or_default();
            if let Some(next) = options.get(at.saturating_add(1) % options.len().max(1)) {
                view.group_by = *next;
            }
            view.rerun_collapsed();
        }
        KeyCode::Char('1') => toggle_board(view, Board::Deck),
        KeyCode::Char('2') => toggle_board(view, Board::Maybeboard),
        KeyCode::Char('3') => toggle_board(view, Board::Sideboard),
        KeyCode::Char('T') => view.show_tokens = !view.show_tokens,
        KeyCode::Char('C') => view.show_zone = !view.show_zone,
        KeyCode::Char('A') => view.show_art = !view.show_art,
        KeyCode::Char('f') if view.loaded => view.open_filter(cx),
        KeyCode::Char('x') if !view.filter.is_empty() => view.reset_filter(cx),
        KeyCode::Char('u') => view.undo(cx),
        KeyCode::Char('a') => view.open_add(cx),
        KeyCode::Char('r') => view.open_remove(cx),
        KeyCode::Char('?') => hint::reopen(cx, HINT_DECK_CARDS),
        _ => row_key(view, code, cx),
    }
}

/// Whether Maybe or Side can be turned on: only with cards on that board, unless already on.
fn board_enabled(view: &DeckCardsView, board: Board) -> bool {
    let on = match board {
        Board::Deck => return true,
        Board::Maybeboard => view.boards.maybe,
        Board::Sideboard => view.boards.side,
    };
    on || view.entries.iter().any(|e| e.deck_card.board == board)
}

fn toggle_board(view: &mut DeckCardsView, board: Board) {
    if board_enabled(view, board) {
        view.boards.toggle(board);
        view.rerun_collapsed();
    }
}

/// Enter on a group header collapses it; on a card row it opens or closes the row.
fn toggle(view: &mut DeckCardsView) {
    match view.selected() {
        Some(Item::Header {
            key,
            collapsible: true,
            ..
        }) => {
            if !view.collapsed.remove(&key) {
                view.collapsed.insert(key);
            }
        }
        Some(Item::Card { card, .. }) => {
            let id = card.scryfall_data.id;
            view.expanded = if view.expanded == Some(id) {
                None
            } else {
                Some(id)
            };
            view.face = 0;
        }
        _ => {}
    }
}

/// A key on the open row's actions.
fn row_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    let Some((card, kind)) = view.open_row() else {
        return;
    };
    let id = card.scryfall_data.id;
    let board = view.entry(id).map(|e| e.deck_card.board);
    match (code, kind) {
        (KeyCode::Char('t'), _) if card_info::face_count(&card) > 1 => {
            view.face = view.face.saturating_add(1) % card_info::face_count(&card);
        }
        (KeyCode::Char('i'), _) => view.preview = Some((Box::new(card), view.face)),
        (KeyCode::Char('-'), Kind::Main | Kind::Board(_)) => view.change_quantity(cx, id, -1, true),
        (KeyCode::Char('+' | '='), Kind::Main | Kind::Board(_)) => {
            view.change_quantity(cx, id, 1, true);
        }
        (KeyCode::Char('p'), Kind::Main | Kind::Board(_)) => {
            view.open_printing(cx, card, PrintingTarget::Deck);
        }
        (KeyCode::Char('p'), Kind::Zone(slot)) => {
            view.open_printing(cx, card, PrintingTarget::Zone(slot));
        }
        (KeyCode::Char('s'), Kind::Main) => view.toggle_mvp(cx, id),
        (KeyCode::Char(c @ ('d' | 'm' | 'b')), Kind::Main | Kind::Board(_)) => {
            let target = match c {
                'd' => Board::Deck,
                'm' => Board::Maybeboard,
                _ => Board::Sideboard,
            };
            if board.is_some_and(|b| b != target) {
                view.move_to_board(cx, id, target, true);
            }
        }
        _ => {}
    }
}

/// A key in quick add.
fn quick_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    let count = match &view.quick.results {
        super::Dropdown::Results(results) => results.len(),
        _ => 0,
    };
    match code {
        KeyCode::Esc | KeyCode::Up | KeyCode::Down => view.quick.focused = false,
        KeyCode::Tab => {
            view.quick.highlight = view
                .quick
                .highlight
                .saturating_add(1)
                .min(count.saturating_sub(1));
        }
        KeyCode::BackTab => view.quick.highlight = view.quick.highlight.saturating_sub(1),
        KeyCode::Enter => view.quick_pick(cx),
        KeyCode::Delete if !view.quick.input.is_empty() => {
            view.quick.input.set_value("");
            view.quick_edit(cx);
        }
        _ => {
            let before = view.quick.input.value().to_owned();
            view.quick.input.handle_key(code);
            if view.quick.input.value() != before {
                view.quick_edit(cx);
            }
        }
    }
}

/// Hands the key to an open sheet or preview. Returns whether one took it.
fn overlay_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) -> bool {
    if let Some(sheet) = &mut view.sheet {
        match card_sheets::filter::keys::handle(sheet, code, cx) {
            SheetOutcome::Open => {}
            SheetOutcome::Closed => view.sheet = None,
            SheetOutcome::Applied(filter) => {
                view.sheet = None;
                view.filter = *filter;
                view.rerun_collapsed();
            }
        }
        return true;
    }
    if let Some(printing) = &mut view.printing {
        match printing.handle_key(code, cx) {
            PrintingOutcome::Open => {}
            PrintingOutcome::Closed => view.printing = None,
            PrintingOutcome::Saved(new_card) => {
                let old_card = Box::new(printing.card.clone());
                view.printing = None;
                let request = match view.printing_target {
                    PrintingTarget::Deck => Request::Printing {
                        deck_id: view.deck_id,
                        old_card,
                        new_card,
                    },
                    PrintingTarget::Zone(slot) => Request::ZonePrinting {
                        deck_id: view.deck_id,
                        slot,
                        new_card,
                        old_card: Some(old_card),
                    },
                };
                cx.request(request.into());
            }
        }
        return true;
    }
    if let Some((card, face)) = &mut view.preview {
        match code {
            KeyCode::Char('t') if card_info::face_count(card) > 1 => {
                *face = face.saturating_add(1) % card_info::face_count(card);
            }
            _ => view.preview = None,
        }
        return true;
    }
    false
}
