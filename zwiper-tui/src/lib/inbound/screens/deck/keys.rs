//! The deck screen's keys. Tab and Shift+Tab move between the panes that show, j and k move through the focused pane's roles, tags and fixes or scroll a pane that has none, Space presses one, and Enter zooms the focused pane, then presses while zoomed. Esc unzooms before it goes back. h and l switch Budget's currency or step Draw odds' turn, as do - and +, and i explains the focused section. e is Edit, c Cards, m More and s swipe, and each sheet and dialog over the screen has its own keys.

use crossterm::event::KeyCode;
use zwipe_ui::screens::deck_view::{self, Action, MoreItem, Section};

use super::draw::CURRENCIES;
use super::layout::Pane;
use super::{CloneDialog, DeckView, Dialog, Item, dispatch, info, reload, reopen_hint};
use crate::inbound::app::Context;
use crate::inbound::screens::otag_examples;
use crate::inbound::widgets::alert::AlertChoice;
use crate::inbound::widgets::menu::MenuKey;
use crate::inbound::widgets::{browser, help};

pub(crate) fn handle(view: &mut DeckView, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(dialog) = view.dialog.take() {
        let kept = handle_dialog(view, dialog, code, cx);
        // A dialog the press opened, such as the Share sheet's, stays over the one that closed.
        if view.dialog.is_none() {
            view.dialog = kept;
        }
        return;
    }
    if let Some(menu) = &mut view.more {
        match menu.handle_key(code) {
            MenuKey::Stay => {}
            MenuKey::Close => view.more = None,
            MenuKey::Pressed(index) => press_more(view, index, cx),
        }
        return;
    }
    let draw_odds = view.focused() == Pane::Section(Section::DrawOdds);
    match code {
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc if view.zoomed => view.zoomed = false,
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Char('e') => dispatch(view, Action::Edit, cx),
        KeyCode::Char('c') => dispatch(view, Action::Cards, cx),
        KeyCode::Char('m') => view.more = Some(view.more_menu()),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('i') => {
            if let Pane::Section(section) = view.focused() {
                view.dialog = Some(Dialog::Info(info(section)));
            }
        }
        KeyCode::Char('r') => reload(view, cx),
        KeyCode::Char('s') => dispatch(view, Action::AddCards, cx),
        KeyCode::Char('+' | '=') if draw_odds => view.later_turn(),
        KeyCode::Char('-') if draw_odds => view.earlier_turn(),
        KeyCode::Tab => view.next_pane(),
        KeyCode::BackTab => view.previous_pane(),
        KeyCode::Down | KeyCode::Char('j') => view.move_down(),
        KeyCode::Up | KeyCode::Char('k') => view.move_up(),
        KeyCode::Left | KeyCode::Char('h') => step(view, Step::Back, cx),
        KeyCode::Right | KeyCode::Char('l') => step(view, Step::Forward, cx),
        KeyCode::Enter if !view.zoomed => view.zoomed = true,
        KeyCode::Enter | KeyCode::Char(' ') => press(view, cx),
        _ => {}
    }
}

/// Which way h and l step.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Step {
    Back,
    Forward,
}

/// h and l on Budget pick the previous or next currency once there are cards to price, and on Draw odds step a turn earlier or later.
fn step(view: &mut DeckView, step: Step, cx: &mut Context<'_>) {
    match view.focused() {
        Pane::Section(Section::Budget) if view.model.currency_chips_shown() => {
            let here = CURRENCIES
                .iter()
                .position(|c| *c == view.model.currency())
                .unwrap_or(0);
            let next = match step {
                Step::Forward => here.saturating_add(1),
                Step::Back => here.saturating_add(CURRENCIES.len().saturating_sub(1)),
            }
            .checked_rem(CURRENCIES.len())
            .unwrap_or(0);
            if let Some(currency) = CURRENCIES.get(next) {
                dispatch(view, Action::PickCurrency(*currency), cx);
            }
        }
        Pane::Section(Section::DrawOdds) => match step {
            Step::Forward => view.later_turn(),
            Step::Back => view.earlier_turn(),
        },
        _ => {}
    }
}

/// Presses the focused item: a role opens or closes its tags, a tag its definition, Examples its overlay, and a fix runs.
fn press(view: &mut DeckView, cx: &mut Context<'_>) {
    match view.focused_item() {
        Some(Item::Role(role)) => {
            view.tag_open = None;
            view.role_open = if view.role_open.as_deref() == Some(role.as_str()) {
                None
            } else {
                Some(role)
            };
        }
        Some(Item::Tag(tag)) => {
            view.tag_open = if view.tag_open.as_deref() == Some(tag.as_str()) {
                None
            } else {
                Some(tag)
            };
        }
        Some(Item::Examples(slug)) => otag_examples::open(cx, &slug),
        Some(Item::Fix {
            warning: index,
            button,
        }) => {
            let action = view
                .model
                .warnings()
                .get(index)
                .map(deck_view::fix_buttons)
                .and_then(|buttons| buttons.into_iter().nth(button))
                .map(|(_, action)| action);
            if let Some(action) = action {
                dispatch(view, action, cx);
            }
        }
        None => {}
    }
}

/// Presses the More sheet's item at `index`. Share, Clear skips and Delete open their dialogs over the sheet; the rest close it.
fn press_more(view: &mut DeckView, index: usize, cx: &mut Context<'_>) {
    let Some(item) = MoreItem::ALL.get(index).copied() else {
        return;
    };
    if !matches!(
        item,
        MoreItem::ShareDeck | MoreItem::ClearSkips | MoreItem::DeleteDeck
    ) {
        view.more = None;
    }
    dispatch(view, Action::More(item), cx);
}

/// Keys in an open dialog. Returns the dialog when it stays open.
fn handle_dialog(
    view: &mut DeckView,
    dialog: Dialog,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Dialog> {
    match dialog {
        Dialog::Info(popup) => {
            (!crate::inbound::widgets::info::closes_on(code)).then_some(Dialog::Info(popup))
        }
        Dialog::Clone(clone) => handle_clone(view, clone, code, cx),
        Dialog::Buy(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::Buy(alert)),
            Some(AlertChoice::Pressed(link @ (1 | 2))) => {
                if let Some((tcg, ck)) = view.model.buy_links() {
                    browser::open(cx, if link == 1 { &tcg } else { &ck });
                }
                None
            }
            Some(_) => None,
        },
        Dialog::Share(mut alert) => {
            let shared = view.model.share_token().is_some();
            let action = match (alert.handle_key(code), shared) {
                (None, _) => return Some(Dialog::Share(alert)),
                (Some(AlertChoice::Pressed(1)), false) => Action::Share,
                (Some(AlertChoice::Pressed(1)), true) => Action::CopyLink,
                (Some(AlertChoice::Pressed(2)), true) => Action::StopShare,
                (Some(_), _) => return None,
            };
            view.more = None;
            dispatch(view, action, cx);
            None
        }
        Dialog::ClearSkips(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::ClearSkips(alert)),
            Some(AlertChoice::Pressed(1)) => {
                view.more = None;
                dispatch(view, Action::ClearSkips, cx);
                None
            }
            Some(_) => None,
        },
        Dialog::Delete(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::Delete(alert)),
            Some(AlertChoice::Pressed(1)) => {
                view.more = None;
                dispatch(view, Action::Delete, cx);
                None
            }
            Some(_) => None,
        },
    }
}

/// Keys in the Clone dialog: typing names the copy, Tab and the arrows move between Cancel and Save, Enter presses, Esc cancels.
fn handle_clone(
    view: &mut DeckView,
    mut clone: CloneDialog,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Dialog> {
    if clone.busy {
        return Some(Dialog::Clone(clone));
    }
    match code {
        KeyCode::Esc => return None,
        KeyCode::Tab | KeyCode::BackTab => clone.focused = 1_usize.saturating_sub(clone.focused),
        KeyCode::Enter if clone.focused == 0 => return None,
        KeyCode::Enter => {
            let name = clone.name.value().to_owned();
            if deck_view::can_clone(&name) {
                clone.busy = true;
                dispatch(view, Action::Clone { name }, cx);
            }
        }
        other => {
            clone.name.handle_key(other);
        }
    }
    Some(Dialog::Clone(clone))
}
