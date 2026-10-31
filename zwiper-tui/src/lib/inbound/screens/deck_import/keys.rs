//! Import's keys: Tab moves between From, Mode, Board and the input; the arrows pick in a chip row; the input takes typing; F2 imports; Esc goes back.

use crossterm::event::KeyCode;
use zwipe_core::domain::deck::ImportMode;

use super::{Focus, ImportView, Source, Target, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut ImportView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::F(2) => view.submit(cx),
        KeyCode::Tab => view.step_focus(Toward::Next),
        KeyCode::BackTab => view.step_focus(Toward::Previous),
        other => match view.focus {
            Focus::Input => match (view.source, other) {
                (Source::Archidekt, KeyCode::Enter) => view.submit(cx),
                (Source::Archidekt, _) => {
                    view.url.handle_key(other);
                }
                (Source::Text, _) => {
                    view.text.handle_key(other);
                }
            },
            Focus::From | Focus::Mode | Focus::Board => match other {
                KeyCode::Char('!') => help::open(cx),
                KeyCode::Char('q') => cx.quit(),
                KeyCode::Char('?') => reopen_hint(cx),
                KeyCode::Down | KeyCode::Char('j') => view.step_focus(Toward::Next),
                KeyCode::Up | KeyCode::Char('k') => view.step_focus(Toward::Previous),
                KeyCode::Left | KeyCode::Char('h') => step_chip(view, Toward::Previous),
                KeyCode::Right | KeyCode::Char('l') => step_chip(view, Toward::Next),
                _ => {}
            },
        },
    }
}

/// Picks the next or previous chip in the focused row; each row picks exactly one.
fn step_chip(view: &mut ImportView, toward: Toward) {
    match view.focus {
        Focus::From => {
            view.source = if toward == Toward::Next {
                Source::Archidekt
            } else {
                Source::Text
            };
        }
        Focus::Mode => {
            view.mode = if toward == Toward::Next {
                ImportMode::Replace
            } else {
                ImportMode::Add
            };
        }
        Focus::Board => {
            let here = Target::ALL
                .iter()
                .position(|t| *t == view.target)
                .unwrap_or(0);
            let next = toward.clamped(here, Target::ALL.len());
            view.target = Target::ALL.get(next).copied().unwrap_or_default();
        }
        Focus::Input => {}
    }
}
