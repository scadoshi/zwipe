//! Activity's keys. j/k and the up and down arrows move the focus through the section headers, the heatmap, the Show chips, the chart rows and the Most skipped rows; Enter or Space opens and closes what is focused; `?` shows the focused section's topic while it is open; `H` shows the screen's hint again; Esc goes back.
//!
//! Inside the heatmap the arrows and hjkl move the day cursor and show that day's counts; stepping up past the top row or down past the bottom one leaves the heatmap. On the Show chips h/l and the left and right arrows pick the rate, which switches both charts. On a revealed role, Tab and `BackTab` open its tags' definitions and `x` opens the open tag's examples; on a revealed tag `x` opens its examples. The Most skipped rows take the card rows' own keys.

use crossterm::event::KeyCode;

use super::{ActivityView, Focus, Load, Section, pass_row, passes_topic, reopen_hint};
use crate::domain::activity::RateKind;
use crate::inbound::app::Context;
use crate::inbound::screens::otag_examples;
use crate::inbound::widgets::card_rows::CardRow;
use crate::inbound::widgets::info;

pub(crate) fn handle(view: &mut ActivityView, code: KeyCode, cx: &mut Context<'_>) {
    if view.overlay.is_some() {
        if info::closes_on(code) {
            view.overlay = None;
        }
        return;
    }
    match code {
        KeyCode::Esc => return cx.back(),
        KeyCode::Char('H') => return reopen_hint(cx),
        KeyCode::Char('?') => {
            view.overlay = topic(view);
            return;
        }
        _ => {}
    }
    match view.focus {
        Focus::Header(section) => on_header(view, section, code),
        Focus::Heat => on_heat(view, code),
        Focus::Chips(_) => on_chips(view, code),
        Focus::Bar(section, index) => on_bar(view, section, index, code, cx),
        Focus::Passes => on_passes(view, code, cx),
    }
}

/// The focused section's topic while it is open, or the Most skipped group's.
fn topic(view: &ActivityView) -> Option<info::InfoPopup> {
    match view.focused_section() {
        Some(section) => view.is_open(section).then(|| section.topic()),
        None => Some(passes_topic()),
    }
}

/// j/k and the up and down arrows move between stops. Returns whether `code` was one of them.
fn step(view: &mut ActivityView, code: KeyCode) -> bool {
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

fn on_header(view: &mut ActivityView, section: Section, code: KeyCode) {
    if matches!(code, KeyCode::Enter | KeyCode::Char(' ')) {
        view.toggle(section);
        view.settle_focus();
    } else {
        step(view, code);
    }
}

fn on_heat(view: &mut ActivityView, code: KeyCode) {
    match code {
        KeyCode::Left | KeyCode::Char('h') => {
            view.move_cursor(-1, 0);
        }
        KeyCode::Right | KeyCode::Char('l') => {
            view.move_cursor(1, 0);
        }
        KeyCode::Up | KeyCode::Char('k') => {
            if !view.move_cursor(0, -1) {
                view.step(false);
            }
        }
        KeyCode::Down | KeyCode::Char('j') => {
            if !view.move_cursor(0, 1) {
                view.step(true);
            }
        }
        KeyCode::Enter | KeyCode::Char(' ') => {
            view.move_cursor(0, 0);
        }
        _ => {}
    }
}

fn on_chips(view: &mut ActivityView, code: KeyCode) {
    let at = RateKind::ALL
        .iter()
        .position(|&r| r == view.rate)
        .unwrap_or(0);
    let next = match code {
        KeyCode::Left | KeyCode::Char('h') => at.checked_sub(1),
        KeyCode::Right | KeyCode::Char('l') => at.checked_add(1),
        _ => {
            step(view, code);
            return;
        }
    };
    if let Some(&rate) = next.and_then(|i| RateKind::ALL.get(i)) {
        view.pick_rate(rate);
    }
}

fn on_bar(
    view: &mut ActivityView,
    section: Section,
    index: usize,
    code: KeyCode,
    cx: &mut Context<'_>,
) {
    match code {
        KeyCode::Enter | KeyCode::Char(' ') => {
            let Some(bar) = view.bars(section).into_iter().nth(index) else {
                return;
            };
            if !bar.revealable {
                return;
            }
            match section {
                Section::Roles => view.toggle_role(&bar.key),
                Section::Tags => view.toggle_tag(&bar.key),
                Section::Activity => {}
            }
        }
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Char('x') if section == Section::Roles => {
            view.role_tags.handle_key(code, cx);
        }
        KeyCode::Char('x') if section == Section::Tags => {
            if let Some(slug) = &view.open_tag {
                otag_examples::open(cx, slug);
            }
        }
        _ => {
            step(view, code);
        }
    }
}

fn on_passes(view: &mut ActivityView, code: KeyCode, cx: &mut Context<'_>) {
    if view.rows.focused() == 0 && matches!(code, KeyCode::Up | KeyCode::Char('k')) {
        view.step(false);
        return;
    }
    let Load::Loaded(loaded) = &view.load else {
        return;
    };
    let palette = cx.shared.palette;
    let rows: Vec<CardRow<'_>> = loaded
        .activity
        .passes
        .iter()
        .map(|p| pass_row(&palette, p))
        .collect();
    view.rows.handle_key(&rows, code, cx);
}
