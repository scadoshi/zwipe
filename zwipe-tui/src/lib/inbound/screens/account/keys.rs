//! The account screen's keys: the rows, and the forms and dialogs over them.

use crossterm::event::KeyCode;

use super::{AccountChange, AccountRow, AccountView, Dialog, Form, FormKind, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::{self, Password};

pub(crate) fn handle(view: &mut AccountView, code: KeyCode, cx: &mut Context<'_>) {
    match view.dialog.take() {
        Some(Dialog::Form(form)) => view.dialog = handle_form(form, code, cx),
        Some(Dialog::ConfirmPassword(mut form)) => match code {
            KeyCode::Enter | KeyCode::Char('y') => {
                form.busy = true;
                cx.request(
                    Request::Change(AccountChange::Password {
                        current: Password(form.value(0).to_owned()),
                        new: Password(form.value(1).to_owned()),
                    })
                    .into(),
                );
                view.dialog = Some(Dialog::ConfirmPassword(form));
            }
            KeyCode::Esc | KeyCode::Char('n') => view.dialog = Some(Dialog::Form(form)),
            _ => view.dialog = Some(Dialog::ConfirmPassword(form)),
        },
        Some(Dialog::LogOut) => match code {
            KeyCode::Enter | KeyCode::Char('y') => screens::sign_out(cx, "Logging out..."),
            KeyCode::Esc | KeyCode::Char('n') => {}
            _ => view.dialog = Some(Dialog::LogOut),
        },
        None => match code {
            KeyCode::Char('q') => cx.quit(),
            KeyCode::Esc => cx.back(),
            KeyCode::Down | KeyCode::Char('j') => view.next_row(),
            KeyCode::Up | KeyCode::Char('k') => view.previous_row(),
            KeyCode::Enter => {
                if let Some(row) = view.selected() {
                    activate(view, row, cx);
                }
            }
            _ => {}
        },
    }
}

fn activate(view: &mut AccountView, row: AccountRow, cx: &mut Context<'_>) {
    match row {
        AccountRow::Username => view.dialog = Some(Dialog::Form(Form::new(FormKind::Username))),
        AccountRow::Email => view.dialog = Some(Dialog::Form(Form::new(FormKind::Email))),
        AccountRow::Password => view.dialog = Some(Dialog::Form(Form::new(FormKind::Password))),
        AccountRow::Delete => view.dialog = Some(Dialog::Form(Form::new(FormKind::Delete))),
        AccountRow::LogOut => view.dialog = Some(Dialog::LogOut),
        AccountRow::Resend => {
            if view.resend_wait().is_none() {
                view.start_resend_cooldown();
                "Sending...".clone_into(&mut cx.shared.status);
                cx.request(Request::Resend.into());
            }
        }
        AccountRow::Check => {
            "Checking...".clone_into(&mut cx.shared.status);
            cx.request(Request::CheckVerification.into());
        }
    }
}

/// Keys in an open form. Returns what stays open: the form, its password confirmation, or nothing.
fn handle_form(mut form: Form, code: KeyCode, cx: &mut Context<'_>) -> Option<Dialog> {
    if form.busy {
        return Some(Dialog::Form(form));
    }
    match code {
        KeyCode::Esc => return None,
        KeyCode::Tab | KeyCode::Down => form.next_field(),
        KeyCode::BackTab | KeyCode::Up => form.previous_field(),
        KeyCode::Backspace => {
            if let Some(field) = form.focused_mut() {
                field.pop();
            }
        }
        KeyCode::Char(c) => {
            if let Some(field) = form.focused_mut() {
                field.push(c);
            }
        }
        KeyCode::Enter if !form.on_last_field() => form.next_field(),
        KeyCode::Enter => return Some(submit(form, cx)),
        _ => {}
    }
    Some(Dialog::Form(form))
}

fn submit(mut form: Form, cx: &mut Context<'_>) -> Dialog {
    if form.delete_wait().is_some() {
        return Dialog::Form(form);
    }
    if let Some(problem) = form.problem() {
        form.error = Some(problem);
        return Dialog::Form(form);
    }
    form.error = None;
    let password = |field: usize| Password(form.value(field).to_owned());
    let change = match form.kind {
        FormKind::Password => return Dialog::ConfirmPassword(form),
        FormKind::Username => AccountChange::Username {
            username: form.value(0).trim().to_owned(),
            password: password(1),
        },
        FormKind::Email => AccountChange::Email {
            email: form.value(0).trim().to_owned(),
            password: password(1),
        },
        FormKind::Delete => AccountChange::Delete {
            password: password(0),
        },
    };
    form.busy = true;
    cx.request(Request::Change(change).into());
    Dialog::Form(form)
}
