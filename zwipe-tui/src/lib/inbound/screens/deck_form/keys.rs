//! The deck form's keys. Tab, Shift-Tab and the arrows move between fields; text fields take typing; F2 saves, F8 clears the focused field, F1 explains it, Esc goes back. On a command-zone field, F3 toggles Filter and F4 opens Swipe.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckOtherTag, PowerLevel};

use super::pickers::{self, FormatPicker, OraclePicker, TagPicker};
use super::{DeckForm, Dropdown, Field, Overlay, reopen_hint, swipe_pick};
use crate::domain::deck::form::{self, Slot, Step};
use crate::inbound::app::Context;
use crate::inbound::widgets::info;

/// The currencies in the app's chip order.
const CURRENCIES: [PriceCurrency; 3] = [PriceCurrency::Usd, PriceCurrency::Eur, PriceCurrency::Tix];

pub(crate) fn handle(form: &mut DeckForm, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(overlay) = form.overlay.take() {
        form.overlay = match overlay {
            Overlay::Info(popup) => (!info::closes_on(code)).then_some(Overlay::Info(popup)),
            Overlay::Format(picker) => pickers::handle_format(form, picker, code, cx),
            Overlay::DeckTags(picker) => pickers::handle_tags(form, picker, code, cx),
            Overlay::OracleTags(picker) => pickers::handle_oracle(form, picker, code, cx),
            Overlay::Swipe(picker) => swipe_pick::handle(form, picker, code, cx),
        };
        return;
    }
    if form.saving {
        return;
    }
    match code {
        KeyCode::Esc => return cx.back(),
        KeyCode::F(2) => return form.save(cx),
        KeyCode::F(1) => return form.overlay = Some(Overlay::Info(form.focus.info())),
        KeyCode::F(8) => return clear(form, cx),
        KeyCode::Tab => return form.step_focus(false),
        KeyCode::BackTab => return form.step_focus(true),
        _ => {}
    }
    if form.loading || form.load_error.is_some() {
        if code == KeyCode::Char('q') {
            cx.quit();
        }
        return;
    }
    match form.focus {
        Field::Name => match code {
            KeyCode::Enter => form.save(cx),
            KeyCode::Up => form.step_focus(true),
            KeyCode::Down => form.step_focus(false),
            other => {
                if form.name.handle_key(other) {
                    form.check_name();
                }
            }
        },
        Field::Card(slot) => handle_card(form, slot, code, cx),
        Field::Price => match code {
            KeyCode::Enter => form.save(cx),
            KeyCode::Up => form.step_focus(true),
            KeyCode::Down => form.step_focus(false),
            KeyCode::Char(c) if !form::price_char(c) => {}
            other => {
                form.price.handle_key(other);
            }
        },
        other => handle_plain(form, other, code, cx),
    }
}

/// Keys on a field that takes no typing, where letters act as keys.
fn handle_plain(form: &mut DeckForm, field: Field, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => return cx.quit(),
        KeyCode::Char('?') => return reopen_hint(cx, form.mode),
        KeyCode::Up | KeyCode::Char('k') => return form.step_focus(true),
        KeyCode::Down | KeyCode::Char('j') => return form.step_focus(false),
        _ => {}
    }
    let left = matches!(code, KeyCode::Left | KeyCode::Char('h'));
    let right = matches!(code, KeyCode::Right | KeyCode::Char('l'));
    let press = matches!(code, KeyCode::Enter | KeyCode::Char(' '));
    let remove = matches!(code, KeyCode::Backspace | KeyCode::Delete);
    match field {
        Field::Format if press => {
            form.overlay = Some(Overlay::Format(Box::new(FormatPicker::open(form))));
        }
        Field::Format if remove => form.set_format(None),
        Field::Power => {
            let count = PowerLevel::all().len();
            step(&mut form.power_focus, count, left, right);
            if press {
                let focused = PowerLevel::all().get(form.power_focus).copied();
                form.power = if form.power == focused { None } else { focused };
            } else if remove {
                form.power = None;
            }
        }
        Field::Land => {
            if left || code == KeyCode::Char('-') {
                form.land_target = Some(form::step_land_target(
                    form.land_target,
                    form.format,
                    Step::Down,
                ));
            } else if right || matches!(code, KeyCode::Char('+' | '=')) {
                form.land_target = Some(form::step_land_target(
                    form.land_target,
                    form.format,
                    Step::Up,
                ));
            } else if remove {
                form.land_target = None;
            }
        }
        Field::Currency => {
            let mut at = CURRENCIES
                .iter()
                .position(|c| *c == form.currency)
                .unwrap_or(0);
            step(&mut at, CURRENCIES.len(), left, right);
            form.currency = CURRENCIES.get(at).copied().unwrap_or_default();
        }
        Field::DeckTags => {
            step(&mut form.tag_focus, form.tags.len(), left, right);
            if press {
                form.overlay = Some(Overlay::DeckTags(TagPicker::open(form)));
            } else if remove && form.tag_focus < form.tags.len() {
                form.tags.remove(form.tag_focus);
                form.tag_focus = form.tag_focus.min(form.tags.len().saturating_sub(1));
            }
        }
        Field::OracleTags => {
            step(&mut form.oracle_focus, form.oracle_tags.len(), left, right);
            if press {
                form.overlay = Some(Overlay::OracleTags(OraclePicker::open(form)));
            } else if remove && form.oracle_focus < form.oracle_tags.len() {
                form.oracle_tags.remove(form.oracle_focus);
                form.oracle_focus = form
                    .oracle_focus
                    .min(form.oracle_tags.len().saturating_sub(1));
            }
        }
        Field::OtherTags => {
            let all = DeckOtherTag::all();
            step(&mut form.other_focus, all.len(), left, right);
            if press && let Some(tag) = all.get(form.other_focus).copied() {
                if let Some(at) = form.other_tags.iter().position(|t| *t == tag) {
                    form.other_tags.remove(at);
                } else {
                    form.other_tags.push(tag);
                }
            }
        }
        Field::Name | Field::Format | Field::Card(_) | Field::Price => {}
    }
}

/// Moves a chip focus left or right within `count` chips.
fn step(focus: &mut usize, count: usize, left: bool, right: bool) {
    if left {
        *focus = focus.saturating_sub(1);
    } else if right {
        *focus = focus.saturating_add(1).min(count.saturating_sub(1));
    }
}

/// Keys on a command-zone field: typing searches, Up and Down move through the results, Enter picks one.
fn handle_card(form: &mut DeckForm, slot: Slot, code: KeyCode, cx: &mut Context<'_>) {
    let hits = form.card(slot).hits().len();
    match code {
        KeyCode::F(3) => {
            let field = form.card_mut(slot);
            field.filter_on = !field.filter_on;
            form.search(slot, cx);
        }
        KeyCode::F(4) => swipe_pick::open(form, slot, cx),
        KeyCode::Down if hits > 0 => {
            let field = form.card_mut(slot);
            field.highlighted = field
                .highlighted
                .saturating_add(1)
                .min(hits.saturating_sub(1));
        }
        KeyCode::Up if hits > 0 => {
            let field = form.card_mut(slot);
            field.highlighted = field.highlighted.saturating_sub(1);
        }
        KeyCode::Up => form.step_focus(true),
        KeyCode::Down => form.step_focus(false),
        KeyCode::Enter => {
            let field = form.card(slot);
            match field.hits().get(field.highlighted).cloned() {
                Some(card) => form.pick_card(slot, card, cx),
                None if field.dropdown == Dropdown::Hidden => form.save(cx),
                None => {}
            }
        }
        other => {
            // A chosen card shows its name; editing it starts a new search, as tapping the box does in the app.
            if form.card(slot).card.is_some()
                && matches!(
                    other,
                    KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
                )
            {
                form.set_card(slot, None);
                if !matches!(other, KeyCode::Char(_)) {
                    return;
                }
            }
            let edited = matches!(
                other,
                KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
            );
            if form.card_mut(slot).input.handle_key(other) && edited {
                form.search(slot, cx);
            }
        }
    }
}

/// Clears the focused field, as its "×" does in the app.
fn clear(form: &mut DeckForm, cx: &mut Context<'_>) {
    match form.focus {
        Field::Name => {
            form.name.set_value("");
            form.check_name();
        }
        Field::Format => form.set_format(None),
        Field::Card(slot) => {
            let field = form.card(slot);
            if field.input.is_empty() || field.card.is_some() {
                form.set_card(slot, None);
            } else {
                // Clears the query only, leaving any chosen card, as the input's own "×" does.
                let field = form.card_mut(slot);
                field.input.set_value("");
                form.search(slot, cx);
            }
        }
        Field::Power => form.power = None,
        Field::Land => form.land_target = None,
        Field::Currency | Field::Price => form.price.set_value(""),
        Field::DeckTags => form.tags.clear(),
        Field::OracleTags => form.oracle_tags.clear(),
        Field::OtherTags => form.other_tags.clear(),
    }
}
