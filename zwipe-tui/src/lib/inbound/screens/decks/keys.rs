//! The deck list's keys: Tab and the arrows move between the chip rows and the list, Enter presses, c creates, m opens More.

use crossterm::event::KeyCode;

use super::{
    DeckList, Focus, Row, create, more_menu, open_commander_maybeboard_stub, open_playstyle_stub,
    reopen_hint,
};
use crate::domain::deck::list::GroupBy;
use crate::inbound::app::Context;
use crate::inbound::screens::{self, deck};
use crate::inbound::widgets::help;
use crate::inbound::widgets::menu::MenuKey;

pub(crate) fn handle(list: &mut DeckList, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(menu) = &mut list.more {
        match menu.handle_key(code) {
            MenuKey::Stay => {}
            MenuKey::Close => list.more = None,
            MenuKey::Pressed(index) => {
                list.more = None;
                if index == 0 {
                    open_commander_maybeboard_stub(cx);
                } else {
                    open_playstyle_stub(cx);
                }
            }
        }
        return;
    }
    match code {
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Char('r') => list.reload(cx),
        KeyCode::Char('O') => screens::sign_out(cx, "Signing out..."),
        KeyCode::Char('c') => create(list, cx),
        KeyCode::Char('m') => list.more = Some(more_menu()),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Tab => list.step_area(false),
        KeyCode::BackTab => list.step_area(true),
        other => match list.focus {
            Focus::List => handle_list(list, other, cx),
            Focus::GroupBy | Focus::Show | Focus::Tags => handle_chips(list, other),
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
            None => list.step_area(true),
        },
        KeyCode::Enter | KeyCode::Char(' ') => match list.rows().get(list.selected).cloned() {
            Some(Row::Header { group, .. }) => {
                let key = group.key();
                if !list.collapsed.remove(&key) {
                    list.collapsed.insert(key);
                }
                list.clamp();
            }
            Some(Row::Deck(profile)) => deck::open(cx, *profile),
            None => {}
        },
        _ => {}
    }
}

fn handle_chips(list: &mut DeckList, code: KeyCode) {
    let count = match list.focus {
        Focus::GroupBy => GroupBy::ALL.len(),
        Focus::Show => list.show_chips().len(),
        Focus::Tags => list.tags().len(),
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
        KeyCode::Down | KeyCode::Char('j') => list.step_area(false),
        KeyCode::Up | KeyCode::Char('k') => list.step_area(true),
        KeyCode::Enter | KeyCode::Char(' ') => match list.focus {
            Focus::GroupBy => list.pick_group_by(list.group_focus),
            Focus::Show => list.press_show(),
            Focus::Tags => list.press_tag(),
            Focus::List => {}
        },
        _ => {}
    }
}
