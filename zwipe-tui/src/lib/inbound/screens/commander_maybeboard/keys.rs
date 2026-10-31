//! The Commander maybeboard's keys. j/k and the up and down arrows move the focus through the Quick add box, its result chips, the Show chips and the saved rows; `/` types into Quick add from anywhere; `f` is Filter, `s` Swipe, `M` More, `H` shows the hint again, and Esc goes back.
//!
//! Typing: typed keys edit the query, Enter or Down hands the keys back (onto the result chips when there are any), and Esc clears the query, the app's "×", and hands the keys back too. On the box, Enter types and `x` clears a query. On a chip row h/l and the left and right arrows move the cursor and Enter or Space picks: a result chip saves that commander, a Show pip toggles its color and All clears them. The saved rows take the card rows' own keys; an expanded row adds `c` Create deck and, with an oracle id, `r` Remove.

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use zwipe_core::domain::card::scryfall_data::colors::Color;

use super::{
    CLEAR, CLEAR_BODY, Focus, MORE_TITLE, MaybeboardView, Overlay, Request, draw, on_sheet,
    open_filter, reopen_hint,
};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::commander_swipe::{self, Deal};
use crate::inbound::screens::playstyle;
use crate::inbound::widgets::alert::{AlertChoice, AlertDialog};
use crate::inbound::widgets::card_rows::CardRow;
use crate::inbound::widgets::sheet::{Sheet, SheetKey};

pub(crate) fn handle(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    if view.sheet.is_some() {
        on_sheet(view, code, cx);
        return;
    }
    if view.overlay.is_some() {
        on_overlay(view, code, cx);
        return;
    }
    if view.quick.typing {
        on_typing(view, code, cx);
        return;
    }
    match code {
        KeyCode::Esc => return cx.back(),
        KeyCode::Char('H') => return reopen_hint(cx),
        KeyCode::Char('f') => return open_filter(view, cx),
        KeyCode::Char('s') => return swipe(view, cx),
        KeyCode::Char('M') => {
            view.overlay = Some(Overlay::More(more_sheet(&cx.shared.palette)));
            return;
        }
        KeyCode::Char('/') if !view.is_loading => {
            view.focus = Focus::Input;
            view.quick.typing = true;
            return;
        }
        _ => {}
    }
    if view.is_loading {
        return;
    }
    match view.focus {
        Focus::Input => on_input(view, code, cx),
        Focus::Results => on_results(view, code, cx),
        Focus::Show => on_show(view, code),
        Focus::List => on_list(view, code, cx),
    }
}

/// Opens the commander swipe in its Commander mode, leaving out the commanders already saved.
fn swipe(view: &MaybeboardView, cx: &mut Context<'_>) {
    let limit = view.deck_limit(cx);
    commander_swipe::open(
        cx,
        Deal::Commander {
            exclude: view.saved(),
        },
        limit,
    );
}

/// The More sheet: its one danger button, Clear maybeboard, pressed with `c` or Enter.
pub(crate) fn more_sheet(palette: &Palette) -> Sheet {
    let button = Line::from(vec![
        Span::raw("c ").bold(),
        Span::styled(format!(" {CLEAR} "), palette.error.reversed()),
    ]);
    Sheet::new(MORE_TITLE, vec![button]).with_footer("Esc", "Close")
}

/// The Clear maybeboard dialog: Cancel, then the danger Clear.
pub(crate) fn clear_dialog() -> AlertDialog {
    AlertDialog::new(CLEAR, CLEAR_BODY)
        .with_button("Cancel", Some('n'))
        .with_danger_button("Clear", Some('y'))
}

fn on_overlay(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(overlay) = &mut view.overlay else {
        return;
    };
    match overlay {
        Overlay::More(sheet) => {
            if matches!(code, KeyCode::Char('c') | KeyCode::Enter) {
                view.overlay = Some(Overlay::Clear(clear_dialog()));
            } else if sheet.handle_key(code) == SheetKey::Close {
                view.overlay = None;
            }
        }
        Overlay::Clear(dialog) => match dialog.handle_key(code) {
            Some(AlertChoice::Pressed(1)) => {
                view.overlay = None;
                cx.request(Request::Clear.into());
            }
            Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => {
                view.overlay = Some(Overlay::More(more_sheet(&cx.shared.palette)));
            }
            None => {}
        },
    }
}

fn on_typing(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Enter | KeyCode::Down => {
            view.quick.typing = false;
            if view.stops().contains(&Focus::Results) {
                view.focus = Focus::Results;
            }
        }
        KeyCode::Esc => {
            view.quick.typing = false;
            clear_query(view, cx);
        }
        other => {
            let before = view.quick.query.value().to_owned();
            view.quick.query.handle_key(other);
            if view.quick.query.value() != before {
                view.query_changed(cx);
            }
        }
    }
}

/// Empties the query, as the app's "×".
fn clear_query(view: &mut MaybeboardView, cx: &mut Context<'_>) {
    if !view.quick.query.is_empty() {
        view.quick.query.set_value("");
        view.query_changed(cx);
    }
}

/// j/k and the up and down arrows move between stops. Returns whether `code` was one of them.
fn step(view: &mut MaybeboardView, code: KeyCode) -> bool {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            view.step(true);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            view.step(false);
        }
        _ => return false,
    }
    true
}

fn on_input(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Enter => view.quick.typing = true,
        KeyCode::Char('x') => clear_query(view, cx),
        _ => {
            step(view, code);
        }
    }
}

/// Moves a chip cursor over `count` chips, or picks the chip under it. Returns the chip picked.
fn on_chips(cursor: &mut usize, count: usize, code: KeyCode) -> Option<usize> {
    match code {
        KeyCode::Left | KeyCode::Char('h') => *cursor = cursor.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => {
            *cursor = cursor.saturating_add(1).min(count.saturating_sub(1));
        }
        KeyCode::Enter | KeyCode::Char(' ') => return Some(*cursor),
        _ => {}
    }
    None
}

fn on_results(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    if step(view, code) {
        return;
    }
    let count = view.shown_results().len();
    if let Some(index) = on_chips(&mut view.quick.cursor, count, code) {
        view.add_result(index, cx);
    }
}

fn on_show(view: &mut MaybeboardView, code: KeyCode) {
    if step(view, code) {
        return;
    }
    let count = Color::all().len().saturating_add(1);
    if let Some(index) = on_chips(&mut view.show_cursor, count, code) {
        view.pick_show(index);
    }
}

fn on_list(view: &mut MaybeboardView, code: KeyCode, cx: &mut Context<'_>) {
    let filtered: Vec<_> = view.filtered().into_iter().cloned().collect();
    let last = filtered.len().saturating_sub(1);
    if leaves_rows(view.rows.focused(), last, code) {
        step(view, code);
        return;
    }
    let rows: Vec<CardRow<'_>> = filtered.iter().map(draw::entry_row).collect();
    if view.rows.handle_key(&rows, code, cx) {
        return;
    }
    let Some(card) = view
        .rows
        .expanded()
        .and_then(|id| filtered.iter().find(|c| c.scryfall_data.id == id))
    else {
        return;
    };
    match code {
        KeyCode::Char('c') => {
            let limit = view.deck_limit(cx);
            playstyle::create_deck_with(cx, card, limit);
        }
        KeyCode::Char('r') => {
            if let Some(oracle_id) = card.scryfall_data.oracle_id {
                cx.request(Request::Remove(oracle_id).into());
            }
        }
        _ => {}
    }
}

/// Whether `code` steps out of the rows: up from the first row or down from the last.
fn leaves_rows(focused: usize, last: usize, code: KeyCode) -> bool {
    match code {
        KeyCode::Up | KeyCode::Char('k') => focused == 0,
        KeyCode::Down | KeyCode::Char('j') => focused >= last,
        _ => false,
    }
}
