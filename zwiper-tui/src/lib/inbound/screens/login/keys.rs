//! Login's keys: typing in the fields, Tab and the arrows across the fields and buttons, Enter on whatever has the focus.

use crossterm::event::KeyCode;

use super::{EMPTY_FORM, LoginField, LoginForm, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::{Password, forgot_password, register};
use crate::inbound::widgets::help;
use crate::inbound::widgets::toast::ToastKind;

pub(crate) fn handle(form: &mut LoginForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.quit(),
        KeyCode::Tab | KeyCode::Down => form.focus = form.focus.next(),
        KeyCode::BackTab | KeyCode::Up => form.focus = form.focus.previous(),
        KeyCode::Enter => press(form, cx),
        KeyCode::Char('!') if form.focused_mut().is_none() => help::open(cx),
        KeyCode::Left | KeyCode::Right if form.focus.button().is_some() => {
            form.focus = if code == KeyCode::Left {
                form.focus.previous()
            } else {
                form.focus.next()
            };
        }
        other => {
            if let Some(field) = form.focused_mut() {
                field.handle_key(other);
            }
        }
    }
}

/// Enter: the identifier moves on to the password, the password submits, and a button does what it says. Buttons do nothing while a sign-in is out.
fn press(form: &mut LoginForm, cx: &mut Context<'_>) {
    match form.focus {
        LoginField::Identifier => form.focus = LoginField::Password,
        LoginField::Password | LoginField::LogIn => submit(form, cx),
        LoginField::ShowPassword => {
            let masked = form.password.is_masked();
            form.password.set_masked(!masked);
        }
        LoginField::CreateProfile if !form.busy => register::open(cx),
        LoginField::ForgotPassword if !form.busy => forgot_password::open(cx),
        LoginField::CreateProfile | LoginField::ForgotPassword => {}
    }
}

/// Sends the sign-in, the identifier as typed. An empty field gets the app's toast instead; the server alone judges the rest.
fn submit(form: &mut LoginForm, cx: &mut Context<'_>) {
    if form.busy {
        return;
    }
    if form.identifier.value().trim().is_empty() || form.password.is_empty() {
        cx.toast(ToastKind::Error, EMPTY_FORM.to_owned());
        return;
    }
    form.busy = true;
    cx.request(
        Request::SignIn {
            identifier: form.identifier.value().to_owned(),
            password: Password(form.password.value().to_owned()),
        }
        .into(),
    );
}
