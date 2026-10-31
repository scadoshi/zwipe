//! The deck screen's keys: j/k through the accordion, Enter to open or press, e Edit, c Cards, m More, and the keys of each sheet and dialog over it.

use crossterm::event::KeyCode;

use super::{
    CloneDialog, DeckView, Dialog, Request, Section, Target, clear_skips_dialog, copy,
    delete_dialog, open_deck_cards_stub, open_tag_examples_stub, reopen_hint, run_fix,
};
use crate::domain::deck::warning;
use crate::inbound::app::Context;
use crate::inbound::screens::{deck_export, deck_form, deck_import, swipe};
use crate::inbound::widgets::alert::AlertChoice;
use crate::inbound::widgets::menu::MenuKey;
use crate::inbound::widgets::{browser, info};

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
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => cx.back(),
        KeyCode::Char('e') => deck_form::open_edit(cx, view.profile.id),
        KeyCode::Char('c') => open_deck_cards_stub(cx, view.profile.id),
        KeyCode::Char('m') => view.more = Some(view.more_menu()),
        KeyCode::Char('?') => reopen_hint(cx),
        KeyCode::Char('i') => {
            if let Some(section) = view.focused_section() {
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
        KeyCode::Char('+' | '=') if view.open == Some(Section::DrawOdds) => view.later_turn(),
        KeyCode::Char('-') if view.open == Some(Section::DrawOdds) => view.earlier_turn(),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => {
            view.focus = view
                .focus
                .saturating_add(1)
                .min(view.targets().len().saturating_sub(1));
        }
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => {
            view.focus = view.focus.saturating_sub(1);
        }
        KeyCode::Left | KeyCode::Char('h') => step_accessory(view, false),
        KeyCode::Right | KeyCode::Char('l') => step_accessory(view, true),
        KeyCode::Enter | KeyCode::Char(' ') => press(view, cx),
        _ => {}
    }
}

/// Left and right on an open Budget switch the currency, and on an open Draw odds step the turn.
fn step_accessory(view: &mut DeckView, forward: bool) {
    match (view.focused_section(), view.open) {
        (Some(Section::Budget), Some(Section::Budget)) if view.has_cards() => {
            if forward {
                view.next_currency();
            } else {
                view.previous_currency();
            }
        }
        (Some(Section::DrawOdds), Some(Section::DrawOdds)) => {
            if forward {
                view.later_turn();
            } else {
                view.earlier_turn();
            }
        }
        _ => {}
    }
}

/// Presses the focused target.
fn press(view: &mut DeckView, cx: &mut Context<'_>) {
    match view.focused() {
        Some(Target::Header(section)) => view.toggle(section),
        Some(Target::Role(role)) => {
            view.tag_open = None;
            view.role_open = if view.role_open.as_deref() == Some(role.as_str()) {
                None
            } else {
                Some(role)
            };
        }
        Some(Target::Tag(tag)) => {
            view.tag_open = if view.tag_open.as_deref() == Some(tag.as_str()) {
                None
            } else {
                Some(tag)
            };
        }
        Some(Target::Examples(slug)) => open_tag_examples_stub(cx, &slug),
        Some(Target::Fix {
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
            Some(AlertChoice::Pressed(0)) => {
                view.more = None;
                cx.request(Request::ClearSkips(deck_id).into());
                None
            }
            Some(_) => None,
        },
        Dialog::Delete(mut alert) => match alert.handle_key(code) {
            None => Some(Dialog::Delete(alert)),
            Some(AlertChoice::Pressed(0)) => {
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
