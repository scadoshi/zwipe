//! The dictionary's keys.
//!
//! Browsing: Esc goes back, `/` starts a search, Left/Right or h/l move along the letter rail, j/k or Up/Down move the selection, `u` or Enter uses the selected tag, `x` opens its examples, and `?` shows the hint again. Letters are commands here, so a search starts only with `/`.
//!
//! Searching: typed keys edit the search, Up/Down still move the selection, Enter keeps the search and returns the keys to the list, Esc clears the search and does the same.

use crossterm::event::KeyCode;
use zwipe_ui::screens::oracle_tag_dictionary::Action;

use super::{OracleTagsView, Step, dispatch, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(view: &mut OracleTagsView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(tags) = cx.shared.caches.oracle_tags.tags() else {
        match code {
            KeyCode::Esc => dispatch(view, Action::Back, cx),
            KeyCode::Char('!') => help::open(cx),
            KeyCode::Char('?') => reopen_hint(cx),
            _ => {}
        }
        return;
    };
    let rows = view.rows(tags);
    let count = rows.len();
    let slug = rows.into_iter().nth(view.selected).map(|t| t.slug);
    let rail: Vec<char> = view
        .model
        .rail(tags)
        .into_iter()
        .map(|chip| chip.letter)
        .collect();
    if view.typing {
        match code {
            KeyCode::Enter => view.typing = false,
            KeyCode::Esc => {
                view.search.set_value("");
                view.typing = false;
                view.selected = 0;
                dispatch(view, Action::Search(String::new()), cx);
            }
            KeyCode::Down => view.step_row(count, Step::Forward),
            KeyCode::Up => view.step_row(count, Step::Back),
            other => {
                if view.search.handle_key(other) {
                    let before = view.model.query().trim().to_lowercase();
                    let value = view.search.value().to_owned();
                    dispatch(view, Action::Search(value), cx);
                    if view.model.query().trim().to_lowercase() != before {
                        view.selected = 0;
                    }
                }
            }
        }
        return;
    }
    match code {
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('/') => view.typing = true,
        KeyCode::Left | KeyCode::Char('h') => view.step_letter(&rail, Step::Back, cx),
        KeyCode::Right | KeyCode::Char('l') => view.step_letter(&rail, Step::Forward, cx),
        KeyCode::Down | KeyCode::Char('j') => view.step_row(count, Step::Forward),
        KeyCode::Up | KeyCode::Char('k') => view.step_row(count, Step::Back),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('u') | KeyCode::Enter => {
            if let Some(slug) = slug {
                dispatch(view, Action::Use(slug), cx);
            }
        }
        KeyCode::Char('x') => {
            if let Some(slug) = slug {
                dispatch(view, Action::OpenExamples(slug), cx);
            }
        }
        _ => {}
    }
}
