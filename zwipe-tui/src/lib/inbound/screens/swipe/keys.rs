//! The swipe screen's keys: add, pass, maybe, undo, and the image toggle.

use crossterm::event::KeyCode;

use super::{Request, SwipeView};
use crate::domain::swipe::Swipe;
use crate::inbound::app::Context;

/// Shown when the commander's synergy picks run out and the server moves on to cards like the deck.
const RELATED_NOTICE: &str = "That's every synergy pick. Showing cards like your deck.";

pub(crate) fn handle(view: &mut SwipeView, code: KeyCode, cx: &mut Context<'_>) {
    if matches!(code, KeyCode::Esc | KeyCode::Char('q')) {
        cx.back();
        return;
    }
    if code == KeyCode::Char('i') {
        view.show_image = !view.show_image;
        return;
    }
    let deck_id = view.deck.id;
    let Some(stack) = &mut view.stack else {
        return;
    };
    let swiped = match code {
        KeyCode::Right | KeyCode::Char('l') => Some(Swipe::Add),
        KeyCode::Left | KeyCode::Char('h') => Some(Swipe::Pass),
        KeyCode::Up | KeyCode::Char('m') => Some(Swipe::Maybe),
        _ => None,
    };
    if let Some(swiped) = swiped {
        if stack.current().is_none() {
            return;
        }
        if swiped == Swipe::Add {
            view.added = view.added.saturating_add(1);
        }
        if let Some(effect) = stack.swipe(swiped) {
            cx.request(Request::Send { deck_id, effect }.into());
        }
    } else if matches!(
        code,
        KeyCode::Down | KeyCode::Char('u') | KeyCode::Backspace
    ) {
        let Some((undone, effect)) = stack.undo() else {
            return;
        };
        if undone == Swipe::Add {
            view.added = view.added.saturating_sub(1);
        }
        if let Some(effect) = effect {
            cx.request(Request::Send { deck_id, effect }.into());
        }
    } else {
        return;
    }

    cx.shared.status = if stack.at_related_start() {
        RELATED_NOTICE.to_owned()
    } else {
        String::new()
    };
    view.refill(cx);
    view.sync_images(cx);
}
