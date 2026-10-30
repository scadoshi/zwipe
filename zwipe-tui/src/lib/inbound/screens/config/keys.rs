//! The config screen's keys: the rows, the theme list that previews as it moves, and the exceptions list.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::preferences::PipStyle;

use super::{ConfigRow, ConfigView, ExceptionsEditor, PreferenceChange, Request, ThemePicker};
use crate::domain::theme::Palette;
use crate::inbound::app::Context;

pub(crate) fn handle(view: &mut ConfigView, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(mut picker) = view.picker.take() {
        if handle_picker(view, &mut picker, code, cx) {
            view.picker = Some(picker);
        }
        return;
    }
    if let Some(editor) = view.exceptions.take() {
        view.exceptions = handle_exceptions(view, editor, code, cx);
        return;
    }
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Down | KeyCode::Char('j') => view.next_row(),
        KeyCode::Up | KeyCode::Char('k') => view.previous_row(),
        KeyCode::Enter | KeyCode::Char(' ') => activate(view, cx),
        _ => {}
    }
}

fn activate(view: &mut ConfigView, cx: &mut Context<'_>) {
    let Some(shown) = view.shown.clone() else {
        return;
    };
    let change = match view.selected() {
        ConfigRow::Theme => {
            view.picker = Some(ThemePicker::opening(&shown.theme));
            return;
        }
        ConfigRow::Exceptions => {
            if view.exceptions_open() {
                view.exceptions = Some(ExceptionsEditor::opening(
                    &shown.universes_beyond_exceptions,
                ));
                cx.request(Request::LoadFranchises.into());
            }
            return;
        }
        ConfigRow::Mode => PreferenceChange::DarkMode {
            theme: shown.theme.clone(),
            dark_mode: !shown.dark_mode,
        },
        ConfigRow::Pips => PreferenceChange::Pips(match shown.pip_style {
            PipStyle::Theme => PipStyle::Natural,
            PipStyle::Natural => PipStyle::Theme,
        }),
        ConfigRow::UniversesBeyond => {
            PreferenceChange::HideUniversesBeyond(!shown.exclude_universes_beyond)
        }
    };
    change_preference(view, change, cx);
}

/// Shows `change` at once and sends it; a failure puts the last saved preferences back.
fn change_preference(view: &mut ConfigView, change: PreferenceChange, cx: &mut Context<'_>) {
    let Some(seq) = view.begin(&change) else {
        return;
    };
    if let Some(shown) = &view.shown {
        cx.shared.palette = Palette::for_preferences(shown);
    }
    cx.request(Request::SavePreferences { seq, change }.into());
}

/// Keys in the theme list. Moving previews the theme; Enter saves it; Esc puts the old one back. Returns whether the list stays open.
fn handle_picker(
    view: &mut ConfigView,
    picker: &mut ThemePicker,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> bool {
    let Some(shown) = view.shown.clone() else {
        return false;
    };
    match code {
        KeyCode::Down | KeyCode::Char('j') => picker.down(),
        KeyCode::Up | KeyCode::Char('k') => picker.up(),
        KeyCode::Enter => {
            if let Some(theme) = picker.theme()
                && theme != picker.original
            {
                change_preference(
                    view,
                    PreferenceChange::Theme {
                        theme: theme.to_owned(),
                        dark_mode: shown.dark_mode,
                    },
                    cx,
                );
            }
            return false;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            cx.shared.palette = Palette::for_preferences(&shown);
            if picker.theme() != Some(picker.original.as_str()) {
                "Theme unchanged".clone_into(&mut cx.shared.status);
            }
            return false;
        }
        _ => {}
    }
    if let Some(theme) = picker.theme() {
        let mut preferences = shown;
        theme.clone_into(&mut preferences.theme);
        cx.shared.palette = Palette::for_preferences(&preferences);
    }
    true
}

/// Keys in the exceptions list. Returns the editor while it stays open.
fn handle_exceptions(
    view: &mut ConfigView,
    mut editor: ExceptionsEditor,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<ExceptionsEditor> {
    match code {
        KeyCode::Down | KeyCode::Char('j') => editor.down(),
        KeyCode::Up | KeyCode::Char('k') => editor.up(),
        KeyCode::Char(' ') => editor.toggle(),
        KeyCode::Char('x') => editor.draft.clear(),
        KeyCode::Enter => {
            if editor.changed() {
                change_preference(view, PreferenceChange::Exceptions(editor.draft), cx);
            }
            return None;
        }
        KeyCode::Esc | KeyCode::Char('q') => {
            if editor.changed() {
                "Exceptions unchanged".clone_into(&mut cx.shared.status);
            }
            return None;
        }
        _ => {}
    }
    Some(editor)
}
