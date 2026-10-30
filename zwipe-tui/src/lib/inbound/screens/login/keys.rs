//! The sign-in form's keys: typing, switching fields, and submitting.

use crossterm::event::KeyCode;

use super::{LoginField, LoginForm, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::Password;

pub(crate) fn handle(form: &mut LoginForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.quit(),
        KeyCode::Tab | KeyCode::BackTab | KeyCode::Up | KeyCode::Down => {
            form.focus = form.focus.other();
        }
        KeyCode::Enter if form.focus == LoginField::Identifier => {
            form.focus = LoginField::Password;
        }
        KeyCode::Enter => submit(form, cx),
        other => {
            form.focused_mut().handle_key(other);
        }
    }
}

fn submit(form: &mut LoginForm, cx: &mut Context<'_>) {
    if form.busy {
        return;
    }
    let identifier = form.identifier.value().trim();
    if identifier.is_empty() || form.password.is_empty() {
        form.error = Some("Enter your username or email and your password.".to_owned());
        return;
    }
    form.busy = true;
    form.error = None;
    cx.request(
        Request::SignIn {
            identifier: identifier.to_owned(),
            password: Password(form.password.value().to_owned()),
        }
        .into(),
    );
}
