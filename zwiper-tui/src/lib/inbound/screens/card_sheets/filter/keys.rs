//! The filter sheet's keys.
//!
//! Up and Down move between section headers and the open section's rows, everywhere. In a text box letters type; Esc leaves the box. Elsewhere Enter or Space opens a section, flips a mode chip or taps a chip, h and l move along a chip row, - and + step a stepper, x clears, a applies, r resets, ? shows the hint, and Esc closes.
//!
//! In the Oracle tags section, `i` shows the oracle tags explainer, and `d` on a label (F3 in its search box) opens the oracle tag dictionary for that list.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::HINT_FILTER;

use super::{FilterSheet, PickerInput, SheetOutcome, Target};
use crate::domain::filter::sections::{self, Clear, Row, Section};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint::{self, HintTopic};

pub(crate) fn handle(sheet: &mut FilterSheet, code: KeyCode, cx: &mut Context<'_>) -> SheetOutcome {
    match code {
        KeyCode::Up => {
            sheet.move_focus(cx, Toward::Previous);
            return SheetOutcome::Open;
        }
        KeyCode::Down => {
            sheet.move_focus(cx, Toward::Next);
            return SheetOutcome::Open;
        }
        _ => {}
    }
    if sheet.typing() {
        if code == KeyCode::F(3)
            && let Some(Target::Row(row)) = sheet.focused()
            && let Some(field) = sections::dictionary_for(row)
        {
            sheet.open_dictionary(field, cx);
            return SheetOutcome::Open;
        }
        type_key(sheet, code, cx);
        return SheetOutcome::Open;
    }
    match code {
        KeyCode::Char('k') => sheet.move_focus(cx, Toward::Previous),
        KeyCode::Char('j') => sheet.move_focus(cx, Toward::Next),
        KeyCode::Esc => return sheet.back(cx),
        KeyCode::Char('a') => return sheet.apply(cx),
        KeyCode::Char('r') => sheet.reset(cx),
        KeyCode::Char('?') => hint::reopen(cx, HINT_FILTER),
        KeyCode::Char('i') if in_oracle_tags(sheet) => hint::topic(cx, HintTopic::OracleTags),
        KeyCode::Char('d') => {
            if let Some(Target::Row(row)) = sheet.focused()
                && let Some(field) = sections::dictionary_for(row)
            {
                sheet.open_dictionary(field, cx);
            }
        }
        _ => act(sheet, code, cx),
    }
    SheetOutcome::Open
}

/// Whether the focus is on the Oracle tags header or one of its rows, where `i` explains oracle tags.
fn in_oracle_tags(sheet: &FilterSheet) -> bool {
    match sheet.focused() {
        Some(Target::Header(section)) => section == Section::OracleTags,
        Some(Target::Row(_)) => sheet.open == Some(Section::OracleTags),
        None => false,
    }
}

/// A key on the focused header, label, chip row or stepper.
fn act(sheet: &mut FilterSheet, code: KeyCode, cx: &mut Context<'_>) {
    let Some(target) = sheet.focused() else {
        return;
    };
    let toggle = matches!(code, KeyCode::Enter | KeyCode::Char(' '));
    match target {
        Target::Header(section) => {
            if toggle {
                sheet.toggle_section(section);
            } else if code == KeyCode::Char('x') && section.is_active(&sheet.draft) {
                section.clear(&mut sheet.draft);
                sheet.sync_inputs();
                sheet.clamp_focus();
            }
        }
        Target::Row(Row::Label { mode, clear, .. }) => {
            if toggle && let Some(chip) = mode {
                sheet.modes.flip(&mut sheet.draft, chip);
                sheet.sync_inputs();
            } else if code == KeyCode::Char('x')
                && let Some(clear) = clear
            {
                clear_label(sheet, clear);
            }
        }
        Target::Row(Row::Chips(set)) => {
            let count = sheet.chips(set, &cx.shared.cards.catalog).len();
            let focus = sheet.chip_focus.entry(set).or_default();
            match code {
                KeyCode::Left | KeyCode::Char('h') => *focus = focus.saturating_sub(1),
                KeyCode::Right | KeyCode::Char('l') => {
                    *focus = focus.saturating_add(1).min(count.saturating_sub(1));
                }
                KeyCode::Enter | KeyCode::Char(' ') => {
                    let at = *focus;
                    sheet.tap_chip(set, at, &cx.shared.cards.catalog);
                }
                _ => {}
            }
        }
        Target::Row(Row::Stepper(row)) => match code {
            KeyCode::Char('-') => sheet.step(row, -1),
            KeyCode::Char('+' | '=') => sheet.step(row, 1),
            KeyCode::Left | KeyCode::Char('h') => sheet.step_max = false,
            KeyCode::Right | KeyCode::Char('l') => sheet.step_max = true,
            _ => {}
        },
        Target::Row(_) => {}
    }
}

/// Clears what a label's x covers, and the text boxes under it.
fn clear_label(sheet: &mut FilterSheet, clear: Clear) {
    clear.apply(&mut sheet.draft);
    sheet.sync_inputs();
    for picker in sheet.pickers.values_mut() {
        *picker = PickerInput::default();
    }
    sheet.clamp_focus();
}

/// A key in a text box, number box or picker.
fn type_key(sheet: &mut FilterSheet, code: KeyCode, cx: &mut Context<'_>) {
    let Some(Target::Row(row)) = sheet.focused() else {
        return;
    };
    match row {
        Row::Text(field) => {
            if matches!(code, KeyCode::Esc | KeyCode::Enter) {
                sheet.move_focus(
                    cx,
                    if code == KeyCode::Enter {
                        Toward::Next
                    } else {
                        Toward::Previous
                    },
                );
                if code == KeyCode::Esc {
                    focus_header(sheet);
                }
                return;
            }
            if let Some(input) = sheet.texts.get_mut(&field)
                && input.handle_key(code)
            {
                let value = input.value().to_owned();
                field.set(&mut sheet.draft, &value);
            }
        }
        Row::Number(field) => match code {
            KeyCode::Esc => {
                sheet.blur(cx);
                focus_header(sheet);
            }
            KeyCode::Enter => sheet.blur(cx),
            _ => {
                if let Some(input) = sheet.numbers.get_mut(&field) {
                    input.handle_key(code);
                }
            }
        },
        Row::Picker(field) => {
            let results = {
                let query = sheet
                    .pickers
                    .get(&field)
                    .map(|p| p.input.value().to_owned())
                    .unwrap_or_default();
                field.results(
                    &sheet.draft,
                    &query,
                    &sheet.config.sources,
                    &cx.shared.cards.catalog,
                )
            };
            let picker = sheet.pickers.entry(field).or_default();
            match code {
                KeyCode::Esc => focus_header(sheet),
                KeyCode::Tab => {
                    picker.highlight = picker
                        .highlight
                        .saturating_add(1)
                        .min(results.len().saturating_sub(1));
                }
                KeyCode::BackTab => picker.highlight = picker.highlight.saturating_sub(1),
                KeyCode::Enter => {
                    let at = picker.highlight.min(results.len().saturating_sub(1));
                    if let Some(value) = results.get(at) {
                        let modes = sheet.modes.clone();
                        field.pick(&mut sheet.draft, &modes, value);
                    }
                }
                KeyCode::Backspace if picker.input.is_empty() => {
                    let modes = sheet.modes.clone();
                    if let Some(last) = field.picked(&sheet.draft).last() {
                        field.unpick(&mut sheet.draft, &modes, last);
                    }
                }
                _ => {
                    if picker.input.handle_key(code) {
                        picker.highlight = 0;
                    }
                }
            }
        }
        _ => {}
    }
}

/// Moves the focus up to the open section's header.
fn focus_header(sheet: &mut FilterSheet) {
    if let Some(section) = sheet.open {
        sheet.focus = sheet
            .targets()
            .iter()
            .position(|t| *t == Target::Header(section))
            .unwrap_or_default();
    }
}
