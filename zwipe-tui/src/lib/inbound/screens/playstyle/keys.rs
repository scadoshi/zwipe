//! Playstyle's keys. j/k and the up and down arrows move the focus through the section headers, the Card roles rows, the Oracle tags table, the Show chips, the suggestion rows, the Staples chips and the staple rows; Enter or Space opens and closes what is focused; `?` shows the focused piece's topic; `M` opens More; `H` shows the screen's hint again; Esc goes back.
//!
//! On a chip row h/l and the left and right arrows move the chip cursor and Enter or Space picks it; picking the selected chip again returns to All. On the table's header h/l pick the column whose "?" shows. On a revealed role Tab and `BackTab` open its tags' definitions and `x` opens the open tag's examples; on a revealed table tag `x` opens its examples. The suggestion and staple rows take the card rows' own keys; an expanded suggestion adds `c` Create deck, `m` Maybeboard, `r` Remove and `w` to fold its reasons.

use crossterm::event::KeyCode;
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};

use super::{
    CLEAR_BODY, CLEAR_SKIPS, Focus, MORE_TITLE, Overlay, PlaystyleView, Request, Section,
    create_deck_with, draw, reopen_hint,
};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;
use crate::inbound::screens::otag_examples;
use crate::inbound::widgets::alert::{AlertChoice, AlertDialog};
use crate::inbound::widgets::card_rows::CardRow;
use crate::inbound::widgets::info;
use crate::inbound::widgets::sheet::{Sheet, SheetKey};

pub(crate) fn handle(view: &mut PlaystyleView, code: KeyCode, cx: &mut Context<'_>) {
    if view.overlay.is_some() {
        on_overlay(view, code, cx);
        return;
    }
    match code {
        KeyCode::Esc => return cx.back(),
        KeyCode::Char('H') => return reopen_hint(cx),
        KeyCode::Char('M') => {
            view.overlay = Some(Overlay::More(more_sheet(&cx.shared.palette)));
            return;
        }
        KeyCode::Char('?') => {
            view.overlay = view.topic().map(Overlay::Info);
            return;
        }
        _ => {}
    }
    match view.focus {
        Focus::Header(section) => on_header(view, section, code),
        Focus::Role(index) => on_role(view, index, code, cx),
        Focus::TagHead => on_tag_head(view, code),
        Focus::Tag(index) => on_tag(view, index, code, cx),
        Focus::Show => on_show(view, code),
        Focus::List => on_list(view, code, cx),
        Focus::Types => on_types(view, code),
        Focus::Colors => on_colors(view, code),
        Focus::Staples => on_staples(view, code, cx),
    }
}

/// The More sheet: its one danger button, Clear skips, pressed with `c` or Enter.
pub(crate) fn more_sheet(palette: &Palette) -> Sheet {
    let button = Line::from(vec![
        Span::raw("c ").bold(),
        Span::styled(format!(" {CLEAR_SKIPS} "), palette.error.reversed()),
    ]);
    Sheet::new(MORE_TITLE, vec![button]).with_footer("Esc", "Close")
}

/// The Clear skips dialog: Cancel, then the danger Clear.
pub(crate) fn clear_dialog() -> AlertDialog {
    AlertDialog::new(CLEAR_SKIPS, CLEAR_BODY)
        .with_button("Cancel", Some('n'))
        .with_danger_button("Clear", Some('y'))
}

fn on_overlay(view: &mut PlaystyleView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(overlay) = &mut view.overlay else {
        return;
    };
    match overlay {
        Overlay::Info(_) => {
            if info::closes_on(code) {
                view.overlay = None;
            }
        }
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
                cx.request(Request::ClearSkips.into());
            }
            Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => {
                view.overlay = Some(Overlay::More(more_sheet(&cx.shared.palette)));
            }
            None => {}
        },
    }
}

/// j/k and the up and down arrows move between stops. Returns whether `code` was one of them.
fn step(view: &mut PlaystyleView, code: KeyCode) -> bool {
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

fn on_header(view: &mut PlaystyleView, section: Section, code: KeyCode) {
    if matches!(code, KeyCode::Enter | KeyCode::Char(' ')) {
        view.toggle(section);
        view.settle_focus();
    } else {
        step(view, code);
    }
}

fn on_role(view: &mut PlaystyleView, index: usize, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(bar) = view.role_bars().into_iter().nth(index)
                && bar.revealable
            {
                view.toggle_role(&bar.key);
            }
        }
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('x') => {
            view.role_tags.handle_key(code, cx);
        }
        _ => {
            step(view, code);
        }
    }
}

fn on_tag_head(view: &mut PlaystyleView, code: KeyCode) {
    let columns = super::Column::ALL;
    let at = columns.iter().position(|&c| c == view.column).unwrap_or(0);
    let next = match code {
        KeyCode::Left | KeyCode::Char('h') => at.checked_sub(1),
        KeyCode::Right | KeyCode::Char('l') => at.checked_add(1),
        _ => {
            step(view, code);
            return;
        }
    };
    if let Some(&column) = next.and_then(|i| columns.get(i)) {
        view.column = column;
    }
}

fn on_tag(view: &mut PlaystyleView, index: usize, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            if let Some(row) = view.tag_rows().into_iter().nth(index) {
                view.toggle_tag(&row.slug);
            }
        }
        KeyCode::Char('x') => {
            if let Some(slug) = &view.open_tag {
                otag_examples::open(cx, slug);
            }
        }
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

fn is_chip_key(code: KeyCode) -> bool {
    matches!(
        code,
        KeyCode::Left | KeyCode::Right | KeyCode::Char('h' | 'l' | ' ') | KeyCode::Enter
    )
}

fn on_show(view: &mut PlaystyleView, code: KeyCode) {
    if !is_chip_key(code) {
        step(view, code);
        return;
    }
    let count = view.identities().len().saturating_add(1);
    if let Some(index) = on_chips(&mut view.show_cursor, count, code) {
        view.pick_identity(index);
    }
}

fn on_types(view: &mut PlaystyleView, code: KeyCode) {
    if !is_chip_key(code) {
        step(view, code);
        return;
    }
    let count = view.staple_types().len().saturating_add(1);
    if let Some(index) = on_chips(&mut view.type_cursor, count, code) {
        view.pick_type(index);
    }
}

fn on_colors(view: &mut PlaystyleView, code: KeyCode) {
    if !is_chip_key(code) {
        step(view, code);
        return;
    }
    let count = view.staple_colors().len().saturating_add(1);
    if let Some(index) = on_chips(&mut view.color_cursor, count, code) {
        view.pick_color(index);
    }
}

fn on_list(view: &mut PlaystyleView, code: KeyCode, cx: &mut Context<'_>) {
    let shown = view.shown();
    let last = shown.len().saturating_sub(1);
    if leaves_rows(view.rows.focused(), last, code) {
        step(view, code);
        return;
    }
    let palette = cx.shared.palette;
    let rows: Vec<CardRow<'_>> = shown
        .iter()
        .map(|s| draw::suggestion_row(&palette, s, view.deck_count, view.why_open))
        .collect();
    let was = view.rows.expanded();
    if view.rows.handle_key(&rows, code, cx) {
        if view.rows.expanded() != was && view.rows.expanded().is_some() {
            view.staple_rows.collapse();
        }
        return;
    }
    let Some(suggestion) = view
        .rows
        .expanded()
        .and_then(|id| shown.iter().find(|s| s.card.scryfall_data.id == id))
    else {
        return;
    };
    let card = &suggestion.card;
    let oracle_id = card.scryfall_data.oracle_id;
    match code {
        KeyCode::Char('c') => {
            let limit = view.deck_limit(cx);
            create_deck_with(cx, card, limit);
        }
        KeyCode::Char('m') => {
            if let Some(oracle_id) = oracle_id {
                cx.request(Request::Maybeboard(oracle_id).into());
            }
        }
        KeyCode::Char('r') => {
            if let Some(oracle_id) = oracle_id {
                cx.request(Request::Dismiss(oracle_id).into());
            }
        }
        KeyCode::Char('w') => {
            let id = card.scryfall_data.id;
            view.why_open = if view.why_open == Some(id) {
                None
            } else {
                Some(id)
            };
        }
        _ => {}
    }
}

fn on_staples(view: &mut PlaystyleView, code: KeyCode, cx: &mut Context<'_>) {
    let staples = view.matching_staples();
    let last = staples.len().saturating_sub(1);
    if leaves_rows(view.staple_rows.focused(), last, code) {
        step(view, code);
        return;
    }
    let palette = cx.shared.palette;
    let rows: Vec<CardRow<'_>> = staples
        .iter()
        .map(|s| draw::staple_row(&palette, s, view.deck_count))
        .collect();
    let was = view.staple_rows.expanded();
    if view.staple_rows.handle_key(&rows, code, cx)
        && view.staple_rows.expanded() != was
        && view.staple_rows.expanded().is_some()
    {
        view.rows.collapse();
    }
}

/// Whether `code` steps out of a group of rows: up from its first row or down from its last.
fn leaves_rows(focused: usize, last: usize, code: KeyCode) -> bool {
    match code {
        KeyCode::Up | KeyCode::Char('k') => focused == 0,
        KeyCode::Down | KeyCode::Char('j') => focused >= last,
        _ => false,
    }
}
