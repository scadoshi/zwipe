//! The deck form's keys. Tab, Shift-Tab and the arrows move between fields; text fields take typing, and Enter in one saves nothing, as in the app, whose Save sits outside the form; F2 saves, F8 clears the focused field, Esc goes back. On a field that takes no typing, `i` opens its About and `?` the form's hint. On a command-zone field, F3 toggles Filter and F4 opens Swipe.

use crossterm::event::KeyCode;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckOtherTag, PowerLevel};
use zwipe_ui::deck::form::{self, Slot, Step};
use zwipe_ui::deck::pickers::{Picker, TagPicker};
use zwipe_ui::screens::deck_form::{Action, Results, Status};

use super::{DeckForm, Field, Overlay, dispatch, pickers, reopen_hint, swipe_pick};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::hint;

/// The currencies in the app's chip order.
const CURRENCIES: [PriceCurrency; 3] = [PriceCurrency::Usd, PriceCurrency::Eur, PriceCurrency::Tix];

pub(crate) fn handle(form: &mut DeckForm, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(overlay) = form.overlay.take() {
        form.overlay = match overlay {
            Overlay::Format(picker) => pickers::handle_format(form, picker, code, cx),
            Overlay::Tags(which, grid) => pickers::handle_tags(form, which, grid, code, cx),
            Overlay::Swipe(picker) => swipe_pick::handle(form, picker, code, cx),
        };
        return;
    }
    if form.model.is_saving() {
        return;
    }
    match code {
        KeyCode::Esc => return dispatch(form, Action::Back, cx),
        KeyCode::F(2) => return save(form, cx),
        KeyCode::F(8) => return clear(form, cx),
        KeyCode::Tab => return form.step_focus(Toward::Next),
        KeyCode::BackTab => return form.step_focus(Toward::Previous),
        _ => {}
    }
    if form.model.status() != Status::Ready {
        if code == KeyCode::Char('q') {
            cx.quit();
        }
        return;
    }
    match form.focus {
        Field::Name => match code {
            KeyCode::Enter => {}
            KeyCode::Up => form.step_focus(Toward::Previous),
            KeyCode::Down => form.step_focus(Toward::Next),
            other => {
                if form.name.handle_key(other) {
                    let name = form.name.value().to_owned();
                    dispatch(form, Action::SetName(name), cx);
                }
            }
        },
        Field::Card(slot) => handle_card(form, slot, code, cx),
        Field::Price => match code {
            KeyCode::Enter => {}
            KeyCode::Up => form.step_focus(Toward::Previous),
            KeyCode::Down => form.step_focus(Toward::Next),
            KeyCode::Char(c) if !form::price_char(c) => {}
            other => {
                if form.price.handle_key(other) {
                    let text = form.price.value().to_owned();
                    dispatch(form, Action::TypePrice(text), cx);
                }
            }
        },
        other => handle_plain(form, other, code, cx),
    }
}

/// Saves, while the save button shows. A name the model refuses takes the focus.
fn save(form: &mut DeckForm, cx: &mut Context<'_>) {
    if form.model.save_label().is_none() {
        return;
    }
    dispatch(form, Action::Save, cx);
    if !form.model.is_saving() && form.model.name_error().is_some() {
        form.focus = Field::Name;
    }
}

/// Keys on a field that takes no typing, where letters act as keys.
fn handle_plain(form: &mut DeckForm, field: Field, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => return cx.quit(),
        KeyCode::Char('?') => return reopen_hint(cx, form),
        KeyCode::Char('i') => return hint::topic(cx, field.topic()),
        KeyCode::Up | KeyCode::Char('k') => return form.step_focus(Toward::Previous),
        KeyCode::Down | KeyCode::Char('j') => return form.step_focus(Toward::Next),
        _ => {}
    }
    let left = matches!(code, KeyCode::Left | KeyCode::Char('h'));
    let right = matches!(code, KeyCode::Right | KeyCode::Char('l'));
    let press = matches!(code, KeyCode::Enter | KeyCode::Char(' '));
    let remove = matches!(code, KeyCode::Backspace | KeyCode::Delete);
    match field {
        Field::Format if press => pickers::open(form, Picker::Format, cx),
        Field::Format if remove => dispatch(form, Action::ClearFormat, cx),
        Field::Power => {
            let count = PowerLevel::all().len();
            step(&mut form.power_focus, count, left, right);
            if press && let Some(level) = PowerLevel::all().get(form.power_focus).copied() {
                dispatch(form, Action::TogglePowerLevel(level), cx);
            } else if remove {
                dispatch(form, Action::ClearPowerLevel, cx);
            }
        }
        Field::Land => {
            if left || code == KeyCode::Char('-') {
                dispatch(form, Action::StepLand(Step::Down), cx);
            } else if right || matches!(code, KeyCode::Char('+' | '=')) {
                dispatch(form, Action::StepLand(Step::Up), cx);
            } else if remove {
                dispatch(form, Action::ClearLand, cx);
            }
        }
        Field::Currency => {
            let currency = form.model.values().currency;
            let mut at = CURRENCIES.iter().position(|c| *c == currency).unwrap_or(0);
            step(&mut at, CURRENCIES.len(), left, right);
            let picked = CURRENCIES.get(at).copied().unwrap_or_default();
            if picked != currency {
                dispatch(form, Action::PickCurrency(picked), cx);
            }
        }
        Field::DeckTags => tag_field(form, TagPicker::DeckTags, (left, right), code, cx),
        Field::OracleTags => tag_field(form, TagPicker::OracleTags, (left, right), code, cx),
        Field::OtherTags => {
            let all = DeckOtherTag::all();
            step(&mut form.other_focus, all.len(), left, right);
            if press && let Some(tag) = all.get(form.other_focus).copied() {
                dispatch(form, Action::ToggleOtherTag(tag), cx);
            }
        }
        Field::Name | Field::Format | Field::Card(_) | Field::Price => {}
    }
}

/// Keys on a tag field: left and right move through its chips, Enter opens its picker, and Backspace takes the focused chip off.
fn tag_field(
    form: &mut DeckForm,
    which: TagPicker,
    (left, right): (bool, bool),
    code: KeyCode,
    cx: &mut Context<'_>,
) {
    let count = form.model.tags(which).len();
    let focus = match which {
        TagPicker::DeckTags => &mut form.tag_focus,
        TagPicker::OracleTags => &mut form.oracle_focus,
    };
    step(focus, count, left, right);
    let at = *focus;
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => pickers::open(form, which.into(), cx),
        KeyCode::Backspace | KeyCode::Delete => {
            let Some(slug) = form.model.tags(which).get(at).cloned() else {
                return;
            };
            dispatch(
                form,
                Action::RemoveTag {
                    picker: which,
                    slug,
                },
                cx,
            );
            let last = form.model.tags(which).len().saturating_sub(1);
            let focus = match which {
                TagPicker::DeckTags => &mut form.tag_focus,
                TagPicker::OracleTags => &mut form.oracle_focus,
            };
            *focus = at.min(last);
        }
        _ => {}
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

/// The results shown under `slot`'s box, when there are some.
fn hits(form: &DeckForm, slot: Slot) -> Vec<Card> {
    match form.model.results(slot) {
        Some(Results::Found(cards)) => cards,
        _ => Vec::new(),
    }
}

/// Keys on a command-zone field: typing searches, Up and Down move through the results, Enter picks one.
fn handle_card(form: &mut DeckForm, slot: Slot, code: KeyCode, cx: &mut Context<'_>) {
    let hits = hits(form, slot);
    match code {
        KeyCode::F(3) => dispatch(form, Action::ToggleFilter(slot), cx),
        KeyCode::F(4) => swipe_pick::open(form, slot, cx),
        KeyCode::Down if !hits.is_empty() => {
            let field = form.inputs.get_mut(slot);
            field.highlighted = field
                .highlighted
                .saturating_add(1)
                .min(hits.len().saturating_sub(1));
        }
        KeyCode::Up if !hits.is_empty() => {
            let field = form.inputs.get_mut(slot);
            field.highlighted = field.highlighted.saturating_sub(1);
        }
        KeyCode::Up => form.step_focus(Toward::Previous),
        KeyCode::Down => form.step_focus(Toward::Next),
        KeyCode::Enter => {
            let highlighted = form.inputs.get(slot).highlighted;
            if let Some(card) = hits.into_iter().nth(highlighted) {
                dispatch(
                    form,
                    Action::PickResult {
                        slot,
                        card: Box::new(card),
                    },
                    cx,
                );
            }
        }
        other => {
            let edits = matches!(
                other,
                KeyCode::Char(_) | KeyCode::Backspace | KeyCode::Delete
            );
            // A chosen card shows its name; editing it starts a new search, as tapping the box does in the app.
            if edits && form.model.values().slot(slot).is_some() {
                dispatch(form, Action::TapInput(slot), cx);
                if !matches!(other, KeyCode::Char(_)) {
                    return;
                }
            }
            let input = &mut form.inputs.get_mut(slot).input;
            let before = input.value().to_owned();
            if input.handle_key(other) && input.value() != before {
                let text = input.value().to_owned();
                dispatch(form, Action::Type { slot, text }, cx);
            }
        }
    }
}

/// Clears the focused field, as its "×" does in the app.
fn clear(form: &mut DeckForm, cx: &mut Context<'_>) {
    let action = match form.focus {
        Field::Name => Action::SetName(String::new()),
        Field::Format => Action::ClearFormat,
        // The box's own "×" clears only the text typed; with none, the label's "×" empties the slot.
        Field::Card(slot) if form.model.has_query(slot) => Action::ClearSearch(slot),
        Field::Card(slot) => Action::ClearSlot(slot),
        Field::Power => Action::ClearPowerLevel,
        Field::Land => Action::ClearLand,
        Field::Currency | Field::Price => Action::ClearPrice,
        Field::DeckTags => Action::ClearTags(TagPicker::DeckTags),
        Field::OracleTags => Action::ClearTags(TagPicker::OracleTags),
        Field::OtherTags => Action::ClearOtherTags,
    };
    dispatch(form, action, cx);
}
