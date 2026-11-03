//! Config's keys: the row buttons, the Themes sheet that previews as it moves, and the Exceptions sheet of franchise chips. Presses the model has an action for go through [`dispatch`]; moving the focus and the theme preview stay here.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::screens::config::Action;

use super::{ConfigView, Sheet, Stop, dispatch, show, topic};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::{help, hint};

pub(crate) fn handle(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    match &view.sheet {
        Some(Sheet::Themes(_)) => handle_themes(view, code, cx),
        Some(Sheet::Exceptions { .. }) => handle_exceptions(view, code, cx),
        None => handle_screen(view, code, cx),
    }
}

fn handle_screen(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    let exceptions = OneTimeHint::UniversesBeyondExceptions.key();
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => view.next(),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => view.previous(),
        KeyCode::Enter | KeyCode::Char(' ') => activate(view, cx),
        KeyCode::Char('i') => match topic(view.focused()) {
            Some(topic) => hint::topic(cx, topic),
            None => hint::reopen(cx, exceptions),
        },
        KeyCode::Char('?') => hint::reopen(cx, OneTimeHint::Config.key()),
        KeyCode::Char('!') => help::open(cx),
        _ => {}
    }
}

/// Presses the focused button. The model ignores the two Universes Beyond buttons until the preferences load.
fn activate(view: &mut ConfigView, cx: &mut Context<'_>) {
    let action = match view.focused() {
        Stop::Mode => Action::ToggleDarkMode {
            shown: view.theme(),
        },
        Stop::Theme => Action::OpenThemes,
        Stop::Pips => Action::TogglePips {
            shown: view.shown.pip_style,
        },
        Stop::UniversesBeyond => Action::ToggleUniversesBeyond,
        Stop::Exceptions => Action::OpenExceptions,
    };
    dispatch(view, action, cx);
}

/// Keys in the Themes sheet: moving previews, Enter keeps a new pick, Esc puts the original back.
fn handle_themes(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(Sheet::Themes(sheet)) = &mut view.sheet else {
        return;
    };
    match code {
        KeyCode::Down | KeyCode::Char('j') => sheet.step(Toward::Next),
        KeyCode::Up | KeyCode::Char('k') => sheet.step(Toward::Previous),
        KeyCode::Enter => {
            let Some(name) = sheet.theme().filter(|_| sheet.picked()) else {
                return;
            };
            // The pick stays on screen; a failed save keeps it, as the app does.
            view.sheet = None;
            name.clone_into(&mut view.shown.theme);
            let theme = view.theme();
            dispatch(view, Action::KeepTheme(theme), cx);
            return;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            let picked = sheet.picked();
            view.sheet = None;
            show(cx, &view.shown);
            dispatch(view, Action::DiscardTheme { picked }, cx);
            return;
        }
        _ => return,
    }
    preview(view, cx);
}

/// Draws the app in the Themes sheet's selection, in the mode shown now.
fn preview(view: &ConfigView, cx: &mut Context<'_>) {
    let Some(Sheet::Themes(sheet)) = &view.sheet else {
        return;
    };
    if let Some(name) = sheet.theme() {
        let mut preview = view.shown.clone();
        name.clone_into(&mut preview.theme);
        show(cx, &preview);
    }
}

/// Keys in the Exceptions sheet: arrows across the chips, Space picks, x clears, Enter saves a changed draft, Esc discards.
fn handle_exceptions(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(Sheet::Exceptions { chip }) = &mut view.sheet else {
        return;
    };
    let count = view.franchises.len();
    match code {
        KeyCode::Right | KeyCode::Down | KeyCode::Char('l' | 'j') => {
            *chip = chip.saturating_add(1).min(count.saturating_sub(1));
        }
        KeyCode::Left | KeyCode::Up | KeyCode::Char('h' | 'k') => {
            *chip = chip.saturating_sub(1);
        }
        KeyCode::Char(' ') => {
            if let Some(slug) = view.franchises.get(*chip).map(|f| f.slug.clone()) {
                dispatch(view, Action::ToggleException(slug), cx);
            }
        }
        KeyCode::Char('x') => dispatch(view, Action::ClearExceptions, cx),
        KeyCode::Char('?') => hint::reopen(cx, OneTimeHint::UniversesBeyondExceptions.key()),
        // Save is off while the draft picks the saved set.
        KeyCode::Enter if !view.model.draft_unchanged() => {
            dispatch(view, Action::SaveExceptions, cx);
        }
        KeyCode::Esc | KeyCode::Char('q') => dispatch(view, Action::DiscardExceptions, cx),
        _ => {}
    }
}
