//! The deck screen's keys. Tab and Shift+Tab move between the panes that show, j and k move through the focused pane's roles, tags and fixes or scroll a pane that has none, Space presses one, and Enter zooms the focused pane, then presses while zoomed. Esc unzooms before it goes back. h and l switch Budget's currency or step Draw odds' turn, as do - and +, and i explains the focused section. e is Edit, c Cards, m More and s swipe, and each sheet and dialog over the screen has its own keys.

use crossterm::event::KeyCode;

use super::layout::Pane;
use super::{
    CloneDialog, DeckView, Dialog, Item, Request, Section, clear_skips_dialog, copy, delete_dialog,
    open_tag_examples_stub, reopen_hint, run_fix,
};
use crate::domain::deck::warning;
use crate::inbound::app::Context;
use crate::inbound::screens::{deck_cards, deck_export, deck_form, deck_import, swipe};
use crate::inbound::widgets::alert::AlertChoice;
use crate::inbound::widgets::menu::MenuKey;
use crate::inbound::widgets::{browser, help, info};

pub(crate) fn handle(view: &mut DeckView, code: KeyCode, cx: &mut Context<'_>) {
    if let Some(dialog) = view.dialog.take() {
        view.dialog = handle_dialog(view, dialog, code, cx);
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
        KeyCode::Esc => cx.back(),
        KeyCode::Char('e') => deck_form::open_edit(cx, view.profile.id),
        KeyCode::Char('c') => deck_cards::open(cx, view.profile.id),
        KeyCode::Char('m') => view.more = Some(view.more_menu()),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('i') => {
            if let Pane::Section(section) = view.focused() {
                view.dialog = Some(Dialog::Info(section.info()));
            }
        }
        KeyCode::Char('r') => view.reload(cx),
        KeyCode::Char('s') => {
            // The deck view reloads when the swipe screen closes, so it hands its copy over.
            if let Some(deck) = view.deck.take() {
                view.metrics = None;
                swipe::open(cx, &deck);
            }
        }
        KeyCode::Char('+' | '=') if draw_odds => view.later_turn(),
        KeyCode::Char('-') if draw_odds => view.earlier_turn(),
        KeyCode::Tab => view.next_pane(),
        KeyCode::BackTab => view.previous_pane(),
        KeyCode::Down | KeyCode::Char('j') => view.move_down(),
        KeyCode::Up | KeyCode::Char('k') => view.move_up(),
        KeyCode::Left | KeyCode::Char('h') => step_back(view),
        KeyCode::Right | KeyCode::Char('l') => step_forward(view),
        KeyCode::Enter if !view.zoomed => view.zoomed = true,
        KeyCode::Enter | KeyCode::Char(' ') => press(view, cx),
        _ => {}
    }
}

/// Right on Budget switches to the next currency once there are cards to price, and on Draw odds steps a turn later.
fn step_forward(view: &mut DeckView) {
    match view.focused() {
        Pane::Section(Section::Budget) if view.has_cards() => view.next_currency(),
        Pane::Section(Section::DrawOdds) => view.later_turn(),
        _ => {}
    }
}

/// Left on Budget switches to the previous currency once there are cards to price, and on Draw odds steps a turn earlier.
fn step_back(view: &mut DeckView) {
    match view.focused() {
        Pane::Section(Section::Budget) if view.has_cards() => view.previous_currency(),
        Pane::Section(Section::DrawOdds) => view.earlier_turn(),
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
        Some(Item::Examples(slug)) => open_tag_examples_stub(cx, &slug),
        Some(Item::Fix {
            warning: index,
            button,
        }) => {
            let fix = view
                .deck
                .as_ref()
                .and_then(|d| d.warnings.get(index))
                .and_then(warning::fix);
            if let Some(fix) = fix {
                run_fix(view, fix, button, cx);
            }
        }
        None => {}
    }
}

/// Presses the More sheet's button at `index`. Share, Clear skips and Delete open their dialogs over the sheet.
fn press_more(view: &mut DeckView, index: usize, cx: &mut Context<'_>) {
    let deck_id = view.profile.id;
    match index {
        0 => {
            view.more = None;
            deck_import::open(cx, deck_id);
        }
        1 => {
            view.more = None;
            deck_export::open(cx, deck_id);
        }
        2 => {
            view.more = None;
            view.dialog = Some(Dialog::Buy(view.buy_dialog()));
        }
        3 => {
            view.more = None;
            view.dialog = Some(Dialog::Clone(CloneDialog::new(&view.profile.name)));
        }
        4 => view.dialog = Some(Dialog::Share(view.share_dialog())),
        5 => view.dialog = Some(Dialog::ClearSkips(clear_skips_dialog())),
        6 => view.dialog = Some(Dialog::Delete(delete_dialog())),
        _ => {}
    }
}

/// Keys in an open dialog. Returns the dialog when it stays open.
fn handle_dialog(
    view: &mut DeckView,
    dialog: Dialog,
    code: KeyCode,
    cx: &mut Context<'_>,
) -> Option<Dialog> {
    let deck_id = view.profile.id;
    match dialog {
        Dialog::Info(popup) => (!info::closes_on(code)).then_some(Dialog::Info(popup)),
        Dialog::Clone(clone) => handle_clone(view, clone, code, cx),
        Dialog::Buy(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::Buy(alert)),
            Some(AlertChoice::Pressed(link @ (1 | 2))) => {
                if let Some((tcg, ck)) = view.buy_links() {
                    browser::open(cx, if link == 1 { &tcg } else { &ck });
                }
                None
            }
            Some(_) => None,
        },
        Dialog::Share(mut alert) => {
            let shared = view.share_token.is_some();
            match (alert.handle_key(code), shared) {
                (None, _) => Some(Dialog::Share(alert)),
                (Some(AlertChoice::Pressed(1)), false) => {
                    view.more = None;
                    cx.request(Request::Share(deck_id).into());
                    None
                }
                (Some(AlertChoice::Pressed(1)), true) => {
                    view.more = None;
                    if let Some(link) = view.share_link() {
                        copy(cx, &link, "Share link copied");
                    }
                    None
                }
                (Some(AlertChoice::Pressed(2)), true) => {
                    view.more = None;
                    cx.request(Request::Unshare(deck_id).into());
                    None
                }
                (Some(_), _) => None,
            }
        }
        Dialog::ClearSkips(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::ClearSkips(alert)),
            Some(AlertChoice::Pressed(1)) => {
                view.more = None;
                cx.request(Request::ClearSkips(deck_id).into());
                None
            }
            Some(_) => None,
        },
        Dialog::Delete(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::Delete(alert)),
            Some(AlertChoice::Pressed(1)) => {
                view.more = None;
                cx.request(Request::Delete(deck_id).into());
                None
            }
            Some(_) => None,
        },
    }
}

/// Keys in the Clone dialog: typing names the copy, Tab and the arrows move between Cancel and Save, Enter presses, Esc cancels.
fn handle_clone(
    view: &DeckView,
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
            if let Some(name) = clone.trimmed() {
                clone.busy = true;
                cx.request(
                    Request::Clone {
                        deck_id: view.profile.id,
                        name,
                    }
                    .into(),
                );
            }
        }
        other => {
            clone.name.handle_key(other);
        }
    }
    Some(Dialog::Clone(clone))
}
