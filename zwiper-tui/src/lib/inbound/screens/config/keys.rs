//! Config's keys: the row buttons, the Themes sheet that previews as it moves, and the Exceptions sheet of franchise chips.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::{HINT_CONFIG, HINT_UNIVERSES_BEYOND_EXCEPTIONS};
use zwipe_core::domain::user::preferences::PipStyle;

use super::{
    ConfigView, ExceptionsSheet, PreferenceChange, Request, Sheet, Stop, ThemeSheet, same_set,
    show, topic,
};
use crate::inbound::app::Context;
use crate::inbound::widgets::toast::{self, ToastKind};
use crate::inbound::widgets::{help, hint};

pub(crate) fn handle(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    match view.sheet.take() {
        Some(Sheet::Themes(sheet)) => {
            view.sheet = handle_themes(view, sheet, code, cx).map(Sheet::Themes);
        }
        Some(Sheet::Exceptions(sheet)) => {
            view.sheet = handle_exceptions(view, sheet, code, cx).map(Sheet::Exceptions);
        }
        None => handle_screen(view, code, cx),
    }
}

fn handle_screen(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => view.next(),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => view.previous(),
        KeyCode::Enter | KeyCode::Char(' ') => activate(view, cx),
        KeyCode::Char('i') => match topic(view.focused()) {
            Some(topic) => hint::topic(cx, topic),
            None => hint::reopen(cx, HINT_UNIVERSES_BEYOND_EXCEPTIONS),
        },
        KeyCode::Char('?') => hint::reopen(cx, HINT_CONFIG),
        KeyCode::Char('!') => help::open(cx),
        _ => {}
    }
}

fn activate(view: &mut ConfigView, cx: &mut Context<'_>) {
    let stop = view.focused();
    if !view.enabled(stop) {
        return;
    }
    let shown = &view.shown;
    let change = match stop {
        Stop::Theme => {
            view.sheet = Some(Sheet::Themes(ThemeSheet::opening(&shown.theme)));
            return;
        }
        Stop::Exceptions => {
            view.sheet = Some(Sheet::Exceptions(ExceptionsSheet {
                draft: shown.universes_beyond_exceptions.clone(),
                chip: 0,
            }));
            if !view.exceptions_opened {
                view.exceptions_opened = true;
                hint::once(cx, HINT_UNIVERSES_BEYOND_EXCEPTIONS);
            }
            return;
        }
        Stop::Mode => PreferenceChange::DarkMode {
            theme: shown.theme.clone(),
            dark_mode: !shown.dark_mode,
        },
        Stop::Pips => PreferenceChange::Pips(match shown.pip_style {
            PipStyle::Theme => PipStyle::Natural,
            PipStyle::Natural => PipStyle::Theme,
        }),
        Stop::UniversesBeyond => {
            PreferenceChange::HideUniversesBeyond(!shown.exclude_universes_beyond)
        }
    };
    send(view, change, cx);
}

/// Sends `change`, showing a toggle at once with the value it replaces kept to put back.
fn send(view: &mut ConfigView, change: PreferenceChange, cx: &mut Context<'_>) {
    let undo = change.undo(&view.shown);
    let seq = view.begin(&change);
    if matches!(
        change,
        PreferenceChange::DarkMode { .. } | PreferenceChange::Pips(_)
    ) {
        let shown = view.shown.clone();
        show(cx, &shown);
    }
    cx.request(Request::Save { seq, change, undo }.into());
}

/// Keys in the Themes sheet: moving previews, Enter saves a new pick, Esc puts the original back. Returns the sheet while it stays open.
fn handle_themes(
    view: &mut ConfigView,
    mut sheet: ThemeSheet,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<ThemeSheet> {
    let mut preview = view.shown.clone();
    match code {
        KeyCode::Down | KeyCode::Char('j') => sheet.step(true),
        KeyCode::Up | KeyCode::Char('k') => sheet.step(false),
        KeyCode::Enter => {
            let Some(theme) = sheet.theme().filter(|_| sheet.picked()) else {
                return Some(sheet);
            };
            // Saved after the sheet closes; a failure keeps the preview, as the app does.
            let change = PreferenceChange::Theme {
                theme: theme.to_owned(),
                dark_mode: view.shown.dark_mode,
            };
            send(view, change, cx);
            return None;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if sheet.picked() {
                show(cx, &preview);
                cx.toast_for(ToastKind::Info, "Theme unchanged".to_owned(), toast::QUICK);
            }
            return None;
        }
        _ => return Some(sheet),
    }
    if let Some(theme) = sheet.theme() {
        theme.clone_into(&mut preview.theme);
        show(cx, &preview);
    }
    Some(sheet)
}

/// Keys in the Exceptions sheet: arrows across the chips, Space picks, x clears, Enter saves a changed draft, Esc discards. Returns the sheet while it stays open.
fn handle_exceptions(
    view: &mut ConfigView,
    mut sheet: ExceptionsSheet,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<ExceptionsSheet> {
    let count = view.franchises.len();
    match code {
        KeyCode::Right | KeyCode::Down | KeyCode::Char('l' | 'j') => {
            sheet.chip = sheet.chip.saturating_add(1).min(count.saturating_sub(1));
        }
        KeyCode::Left | KeyCode::Up | KeyCode::Char('h' | 'k') => {
            sheet.chip = sheet.chip.saturating_sub(1);
        }
        KeyCode::Char(' ') => {
            if let Some(slug) = view.franchises.get(sheet.chip).map(|f| f.slug.clone()) {
                if let Some(at) = sheet.draft.iter().position(|s| *s == slug) {
                    sheet.draft.remove(at);
                } else {
                    sheet.draft.push(slug);
                }
            }
        }
        KeyCode::Char('x') => sheet.draft.clear(),
        KeyCode::Char('?') => hint::reopen(cx, HINT_UNIVERSES_BEYOND_EXCEPTIONS),
        KeyCode::Enter => {
            if same_set(&sheet.draft, &view.shown.universes_beyond_exceptions) {
                return Some(sheet);
            }
            let change = PreferenceChange::Exceptions(sheet.draft);
            let undo = change.undo(&view.shown);
            cx.request(
                Request::Save {
                    seq: 0,
                    change,
                    undo,
                }
                .into(),
            );
            return None;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if !same_set(&sheet.draft, &view.shown.universes_beyond_exceptions) {
                cx.toast_for(
                    ToastKind::Info,
                    "Exceptions unchanged".to_owned(),
                    toast::QUICK,
                );
            }
            return None;
        }
        _ => {}
    }
    Some(sheet)
}
