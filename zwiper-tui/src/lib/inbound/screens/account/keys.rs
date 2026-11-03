//! Account's keys: the row buttons, the action bar's letters, and the sheets and dialogs over the screen. Presses the model has an action for go through [`dispatch`]; moving the focus and opening the More sheet stay here.

use crossterm::event::KeyCode;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::screens::account::{Action, Field, MoreItem, Password, Typed};

use super::{AccountView, DeleteFocus, Overlay, Stop, dispatch};
use crate::inbound::app::Context;
use crate::inbound::widgets::alert::AlertChoice;
use crate::inbound::widgets::help;
use crate::inbound::widgets::hint::{self, HintTopic};

pub(crate) fn handle(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    match &mut view.overlay {
        Some(Overlay::Form(_)) => handle_form(view, code, cx),
        Some(Overlay::ConfirmPassword { alert, .. }) => match alert.handle_key(code) {
            Some(AlertChoice::Pressed(1)) => {
                back_to_sheet(view);
                dispatch(view, Action::ConfirmPassword, cx);
            }
            Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => back_to_sheet(view),
            None => {}
        },
        Some(Overlay::LogOut(alert)) => match alert.handle_key(code) {
            Some(AlertChoice::Pressed(1)) => {
                view.overlay = None;
                dispatch(view, Action::ConfirmLogOut, cx);
            }
            Some(AlertChoice::Pressed(_) | AlertChoice::Dismissed) => view.overlay = None,
            None => {}
        },
        Some(Overlay::More(Some(_))) => handle_delete(view, code, cx),
        Some(Overlay::More(None)) => match code {
            KeyCode::Esc | KeyCode::Char('q') => view.overlay = None,
            KeyCode::Enter | KeyCode::Char('d') => {
                dispatch(view, Action::More(MoreItem::DeleteAccount), cx);
            }
            _ => {}
        },
        None => handle_screen(view, code, cx),
    }
}

/// Closes the password confirm, leaving its sheet open under it.
fn back_to_sheet(view: &mut AccountView) {
    view.overlay = match view.overlay.take() {
        Some(Overlay::ConfirmPassword { form, .. }) => Some(Overlay::Form(form)),
        other => other,
    };
}

/// Keys on the screen itself.
fn handle_screen(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Esc => dispatch(view, Action::Back, cx),
        KeyCode::Down | KeyCode::Char('j') | KeyCode::Tab => view.next(),
        KeyCode::Up | KeyCode::Char('k') | KeyCode::BackTab => view.previous(),
        KeyCode::Enter => {
            if let Some(stop) = view.focused() {
                dispatch(view, stop.action(), cx);
            }
        }
        KeyCode::Char('i') => {
            if let Some(topic) = view.focused().and_then(Stop::topic) {
                hint::topic(cx, topic);
            }
        }
        KeyCode::Char('?') => hint::reopen(cx, OneTimeHint::Profile.key()),
        KeyCode::Char('!') => help::open(cx),
        KeyCode::Char('L') => dispatch(view, Action::LogOut, cx),
        KeyCode::Char('m') => view.overlay = Some(Overlay::More(None)),
        _ => {}
    }
}

/// Keys in a Change sheet. It takes none while its change is out.
fn handle_form(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(Overlay::Form(form)) = &mut view.overlay else {
        return;
    };
    let sheet = form.sheet;
    if view.model.is_saving(sheet) {
        return;
    }
    match code {
        KeyCode::Esc => view.overlay = None,
        KeyCode::Tab | KeyCode::Down => form.next_field(),
        KeyCode::BackTab | KeyCode::Up => form.previous_field(),
        KeyCode::F(2) if form.field() == Some(Field::NewPassword) => {
            hint::topic(cx, HintTopic::Password);
        }
        KeyCode::Enter if !form.on_last_field() => form.next_field(),
        KeyCode::Enter => dispatch(view, Action::Save(sheet), cx),
        other => {
            let field = form.field();
            if let Some(input) = form.inputs.get_mut(form.focus)
                && input.handle_key(other)
                && let Some(field) = field
            {
                let value = Typed::new(input.value());
                dispatch(
                    view,
                    Action::Edit {
                        sheet,
                        field,
                        value,
                    },
                    cx,
                );
            }
        }
    }
}

/// Keys in the Delete account dialog. It takes none while the delete is out.
fn handle_delete(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    let Some(Overlay::More(Some(delete))) = &mut view.overlay else {
        return;
    };
    if view.model.is_deleting() {
        return;
    }
    let order = [
        DeleteFocus::Password,
        DeleteFocus::ShowPassword,
        DeleteFocus::Cancel,
        DeleteFocus::Delete,
    ];
    let at = order.iter().position(|f| *f == delete.focus).unwrap_or(0);
    match code {
        KeyCode::Esc => close_delete(view, cx),
        KeyCode::Tab | KeyCode::Down => {
            delete.focus = order
                .get(at.saturating_add(1).checked_rem(order.len()).unwrap_or(0))
                .copied()
                .unwrap_or(DeleteFocus::Password);
        }
        KeyCode::BackTab | KeyCode::Up => {
            let back = at
                .checked_sub(1)
                .unwrap_or_else(|| order.len().saturating_sub(1));
            delete.focus = order.get(back).copied().unwrap_or(DeleteFocus::Password);
        }
        KeyCode::Enter => match delete.focus {
            DeleteFocus::Cancel => close_delete(view, cx),
            DeleteFocus::ShowPassword => {
                let masked = delete.password.is_masked();
                delete.password.set_masked(!masked);
            }
            DeleteFocus::Password | DeleteFocus::Delete => {
                // The model's countdown alone gates it; an empty password is the server's to refuse.
                let password = Password::new(delete.password.value());
                dispatch(view, Action::Delete { password }, cx);
            }
        },
        other if delete.focus == DeleteFocus::Password => {
            delete.password.handle_key(other);
        }
        _ => {}
    }
}

/// Closes the Delete account dialog back to the More sheet, stopping its countdown.
fn close_delete(view: &mut AccountView, cx: &mut Context<'_>) {
    view.overlay = Some(Overlay::More(None));
    dispatch(view, Action::CancelDelete, cx);
}
