//! The deck cards view's keys.
//!
//! j and k (or the arrows) move through the list; Enter opens a group or a row. On an open row: - and + change the quantity, p changes the printing, s stars, d, m and b move to the main board, maybeboard or sideboard, t flips and i previews; Tab and `BackTab` walk its oracle tags, and while one is open `x` opens its example cards. / types in quick add. g steps Group by, 1 2 3 toggle the boards, T C toggle tokens and the command zone. f filters, x resets the filter, u undoes, a adds, r removes, ? shows the hint, Esc goes back.

use chrono::Utc;
use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::user::hints::HINT_DECK_CARDS;
use zwipe_ui::screens::deck_cards::{Action, QuickAddResults};

use super::{DeckCardsView, Item, Kind, dispatch};
use crate::inbound::app::Context;
use crate::inbound::screens::card_sheets::filter::SheetOutcome;
use crate::inbound::screens::card_sheets::printing::PrintingOutcome;
use crate::inbound::screens::{card_sheets, oracle_tags, otag_examples};
use crate::inbound::widgets::{card_info, help, hint, otag};

pub(crate) fn handle(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    if overlay_key(view, code, cx) {
        return;
    }
    if view.quick.focused {
        quick_key(view, code, cx);
        return;
    }
    if tag_key(view, code, cx) {
        return;
    }
    match code {
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => {
            dispatch(view, Action::Leave, cx);
            cx.quit();
        }
        KeyCode::Down | KeyCode::Char('j') => {
            let last = view.items().len().saturating_sub(1);
            view.cursor = view.cursor.saturating_add(1).min(last);
        }
        KeyCode::Up | KeyCode::Char('k') => view.cursor = view.cursor.saturating_sub(1),
        KeyCode::Enter | KeyCode::Char(' ') => toggle(view, cx),
        KeyCode::Char('/') if view.model.is_loaded() => view.quick.focused = true,
        KeyCode::Char('g') if view.model.is_loaded() => {
            let options = GroupByOption::all();
            let at = options
                .iter()
                .position(|o| *o == view.model.group_by())
                .unwrap_or_default();
            if let Some(next) =
                options.get(at.saturating_add(1).checked_rem(options.len()).unwrap_or(0))
            {
                dispatch(view, Action::PickGroupBy(*next), cx);
            }
        }
        KeyCode::Char('1') => toggle_board(view, Board::Deck, cx),
        KeyCode::Char('2') => toggle_board(view, Board::Maybeboard, cx),
        KeyCode::Char('3') => toggle_board(view, Board::Sideboard, cx),
        KeyCode::Char('T') => dispatch(view, Action::ToggleTokens, cx),
        KeyCode::Char('C') => dispatch(view, Action::ToggleCommandZone, cx),
        KeyCode::Char('f') if view.model.is_loaded() => view.open_filter(cx),
        KeyCode::Char('x') if !view.filter.is_empty() => dispatch(view, Action::ResetFilter, cx),
        KeyCode::Char('u') => view.undo(cx),
        KeyCode::Char('a') if view.model.is_loaded() => dispatch(view, Action::OpenAdd, cx),
        KeyCode::Char('r') if view.model.is_loaded() => dispatch(view, Action::OpenRemove, cx),
        KeyCode::Char('?') => hint::reopen(cx, HINT_DECK_CARDS),
        _ => row_key(view, code, cx),
    }
}

/// Turns a Boards chip on or off while the model lets it be tapped.
fn toggle_board(view: &mut DeckCardsView, board: Board, cx: &mut Context<'_>) {
    if view.model.board_enabled(board) {
        dispatch(view, Action::ToggleBoard(board), cx);
    }
}

/// Enter on a group header collapses it; on a card row it opens or closes the row.
fn toggle(view: &mut DeckCardsView, cx: &mut Context<'_>) {
    match view.selected() {
        Some(Item::Header {
            key,
            collapsible: true,
            ..
        }) => dispatch(view, Action::ToggleSection(key), cx),
        Some(Item::Card { card, .. }) => {
            let id = card.scryfall_data.id;
            view.expanded = if view.expanded == Some(id) {
                None
            } else {
                Some(id)
            };
            view.face = 0;
            view.open_tag = None;
        }
        _ => {}
    }
}

/// Tab and `BackTab` on an open row walk its oracle tags, and `x` opens the open tag's examples. Returns whether the key was used.
fn tag_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) -> bool {
    let Some((card, _)) = view.open_row() else {
        return false;
    };
    let tags = card_info::oracle_tags(&card);
    match code {
        KeyCode::Tab if !tags.is_empty() => {
            oracle_tags::ensure_catalog(cx);
            view.open_tag = otag::next_open(view.open_tag, tags.len());
        }
        KeyCode::BackTab if !tags.is_empty() => {
            oracle_tags::ensure_catalog(cx);
            view.open_tag = otag::previous_open(view.open_tag, tags.len());
        }
        KeyCode::Char('x') => {
            let Some(slug) = view.open_tag.and_then(|i| tags.get(i)) else {
                return false;
            };
            otag_examples::open(cx, slug);
        }
        _ => return false,
    }
    true
}

/// A key on the open row's actions.
fn row_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    let Some((card, kind)) = view.open_row() else {
        return;
    };
    let id = card.scryfall_data.id;
    let board = view.board(id);
    match (code, kind) {
        (KeyCode::Char('t'), _) if card_info::face_count(&card) > 1 => {
            view.face = view
                .face
                .saturating_add(1)
                .checked_rem(card_info::face_count(&card))
                .unwrap_or(0);
        }
        (KeyCode::Char('i'), _) => view.preview = Some((Box::new(card), view.face)),
        (KeyCode::Char('-'), Kind::Main | Kind::Board(_)) => {
            dispatch(
                view,
                Action::ChangeQuantity {
                    card_id: id,
                    delta: -1,
                },
                cx,
            );
        }
        (KeyCode::Char('+' | '='), Kind::Main | Kind::Board(_)) => {
            dispatch(
                view,
                Action::ChangeQuantity {
                    card_id: id,
                    delta: 1,
                },
                cx,
            );
        }
        (KeyCode::Char('p'), Kind::Main | Kind::Board(_) | Kind::Zone(_)) => {
            let slot = match kind {
                Kind::Zone(slot) => Some(slot),
                Kind::Main | Kind::Board(_) | Kind::Token => None,
            };
            let card = Box::new(card);
            dispatch(view, Action::OpenPrintings { card, slot }, cx);
        }
        (KeyCode::Char('s'), Kind::Main) => {
            let now = Utc::now();
            dispatch(view, Action::ToggleMvp { card_id: id, now }, cx);
        }
        (KeyCode::Char(c @ ('d' | 'm' | 'b')), Kind::Main | Kind::Board(_)) => {
            let target = match c {
                'd' => Board::Deck,
                'm' => Board::Maybeboard,
                _ => Board::Sideboard,
            };
            if board.is_some_and(|b| b != target) {
                dispatch(
                    view,
                    Action::MoveTo {
                        card_id: id,
                        board: target,
                    },
                    cx,
                );
            }
        }
        _ => {}
    }
}

/// A key in quick add.
fn quick_key(view: &mut DeckCardsView, code: KeyCode, cx: &mut Context<'_>) {
    let count = match view.model.quick_add() {
        Some(QuickAddResults::Found(results)) => results.len(),
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
            dispatch(view, Action::ClearQuickAdd, cx);
        }
        _ => {
            let before = view.quick.input.value().to_owned();
            view.quick.input.handle_key(code);
            let query = view.quick.input.value().to_owned();
            if query != before {
                dispatch(view, Action::Type(query), cx);
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
                view.apply_filter(*filter, cx);
            }
        }
        return true;
    }
    if let Some(printing) = &mut view.printing {
        match printing.handle_key(code, cx) {
            PrintingOutcome::Open => {}
            PrintingOutcome::Closed => view.printing = None,
            PrintingOutcome::Saved(new_card) => {
                view.printing = None;
                dispatch(view, Action::PickPrinting(new_card), cx);
            }
        }
        return true;
    }
    if let Some((card, face)) = &mut view.preview {
        match code {
            KeyCode::Char('t') if card_info::face_count(card) > 1 => {
                *face = face
                    .saturating_add(1)
                    .checked_rem(card_info::face_count(card))
                    .unwrap_or(0);
            }
            _ => view.preview = None,
        }
        return true;
    }
    false
}
