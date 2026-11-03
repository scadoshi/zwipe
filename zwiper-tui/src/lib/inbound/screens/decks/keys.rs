//! The deck list's keys: Tab and the arrows move between the chip rows and the list, Enter presses, c creates, m opens More.

use crossterm::event::KeyCode;
use zwipe_ui::deck::list::GroupBy;
use zwipe_ui::screens::deck_list::{Action, MoreItem};

use super::{DeckList, Focus, Row, dispatch, more_menu, reload, reopen_hint, verified};
use crate::inbound::app::Context;
use crate::inbound::screens;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::help;
use crate::inbound::widgets::menu::MenuKey;

pub(crate) fn handle(list: &mut DeckList, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(menu) = &mut list.more {
        match menu.handle_key(code) {
            MenuKey::Stay => {}
            MenuKey::Close => list.more = None,
            MenuKey::Pressed(index) => {
                list.more = None;
                if let Some(item) = MoreItem::ALL.get(index) {
                    dispatch(list, Action::More(*item), cx);
                }
            }
        }
        return;
    }
    match code {
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => dispatch(list, Action::Back, cx),
        KeyCode::Char('r') => reload(list, cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        KeyCode::Char('c') => {
            let verified = verified(cx);
            dispatch(list, Action::Create { verified }, cx);
        }
        KeyCode::Char('m') => list.more = Some(more_menu()),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Tab => list.step_area(Toward::Next),
        KeyCode::BackTab => list.step_area(Toward::Previous),
        other => match list.focus {
            Focus::List => handle_list(list, other, cx),
            Focus::GroupBy | Focus::Show | Focus::Tags => handle_chips(list, other, cx),
        },
    }
}

fn handle_list(list: &mut DeckList, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Down | KeyCode::Char('j') => {
            list.selected = list
                .selected
                .saturating_add(1)
                .min(list.rows().len().saturating_sub(1));
        }
        KeyCode::Up | KeyCode::Char('k') => match list.selected.checked_sub(1) {
            Some(above) => list.selected = above,
            None => list.step_area(Toward::Previous),
        },
        KeyCode::Enter | KeyCode::Char(' ') => match list.rows().get(list.selected).cloned() {
            Some(Row::Header { group, .. }) => dispatch(list, Action::ToggleGroup(group.key()), cx),
            Some(Row::Deck(profile)) => dispatch(list, Action::OpenDeck(profile.id), cx),
            None => {}
        },
        _ => {}
    }
}

fn handle_chips(list: &mut DeckList, code: KeyCode, cx: &mut Context<'_>) {
    let count = match list.focus {
        Focus::GroupBy => GroupBy::ALL.len(),
        Focus::Show => list.model.show_chips().len(),
        Focus::Tags => list.model.tags().len(),
        Focus::List => return,
    };
    let focus = match list.focus {
        Focus::GroupBy => &mut list.group_focus,
        Focus::Show => &mut list.show_focus,
        Focus::Tags | Focus::List => &mut list.tag_focus,
    };
    match code {
        KeyCode::Left | KeyCode::Char('h') => *focus = focus.saturating_sub(1),
        KeyCode::Right | KeyCode::Char('l') => {
            *focus = focus.saturating_add(1).min(count.saturating_sub(1));
        }
        KeyCode::Down | KeyCode::Char('j') => list.step_area(Toward::Next),
        KeyCode::Up | KeyCode::Char('k') => list.step_area(Toward::Previous),
        KeyCode::Enter | KeyCode::Char(' ') => {
            let action = match list.focus {
                Focus::GroupBy => DeckList::group_by_action(list.group_focus),
                Focus::Show => list.show_action(),
                Focus::Tags => list.tag_action(),
                Focus::List => None,
            };
            if let Some(action) = action {
                dispatch(list, action, cx);
            }
        }
        _ => {}
    }
}
