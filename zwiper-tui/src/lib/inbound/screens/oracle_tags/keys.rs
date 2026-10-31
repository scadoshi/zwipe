//! The dictionary's keys.
//!
//! Browsing: Esc goes back, `/` starts a search, Left/Right or h/l move along the letter rail, j/k or Up/Down move the selection, `u` or Enter uses the selected tag, `x` opens its examples, and `?` shows the hint again. Letters are commands here, so a search starts only with `/`.
//!
//! Searching: typed keys edit the search, Up/Down still move the selection, Enter keeps the search and returns the keys to the list, Esc clears the search and does the same.

use crossterm::event::KeyCode;

use super::{OracleTagsView, Step, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::screens::otag_examples;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut OracleTagsView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(tags) = cx.shared.caches.oracle_tags.tags() else {
        match code {
            KeyCode::Esc => cx.back(),
            KeyCode::Char('!') => help::open(cx),
            KeyCode::Char('?') => reopen_hint(cx),
            _ => {}
        }
        return;
    };
    let rows = view.rows(tags);
    let count = rows.len();
    let slug = rows.into_iter().nth(view.selected).map(|t| t.slug);
    if view.typing {
        match code {
            KeyCode::Enter => view.typing = false,
            KeyCode::Esc => {
                view.search.set_value("");
                view.typing = false;
                view.selected = 0;
            }
            KeyCode::Down => view.step_row(count, Step::Forward),
            KeyCode::Up => view.step_row(count, Step::Back),
            other => {
                let before = view.query();
                view.search.handle_key(other);
                if view.query() != before {
                    view.selected = 0;
                }
            }
        }
        return;
    }
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('/') => view.typing = true,
        KeyCode::Left | KeyCode::Char('h') => view.step_letter(tags, Step::Back),
        KeyCode::Right | KeyCode::Char('l') => view.step_letter(tags, Step::Forward),
        KeyCode::Down | KeyCode::Char('j') => view.step_row(count, Step::Forward),
        KeyCode::Up | KeyCode::Char('k') => view.step_row(count, Step::Back),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('u') | KeyCode::Enter => {
            if let Some(slug) = slug {
                cx.shared.caches.handoff.oracle_tag = Some(slug);
                cx.back();
            }
        }
        KeyCode::Char('x') => {
            if let Some(slug) = slug {
                otag_examples::open(cx, &slug);
            }
        }
        _ => {}
    }
}
