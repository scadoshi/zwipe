//! Login's keys: typing in the fields, Tab and the arrows across the form, Enter on whatever has the focus, and Esc to end typing. With no field taking letters, `c` opens Create profile, `f` Reset password, `q` quits and `!` opens Help.

use crossterm::event::KeyCode;

use super::{EMPTY_FORM, LoginField, LoginForm, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::{Password, forgot_password, register};
use crate::inbound::widgets::help;
use crate::inbound::widgets::toast::ToastKind;

pub(crate) fn handle(form: &mut LoginForm, code: KeyCode, cx: &mut Context<'_>) {
    let Some(focus) = form.focus else {
        match code {
            KeyCode::Tab | KeyCode::Down => form.focus = Some(LoginField::Identifier),
            KeyCode::BackTab | KeyCode::Up => form.focus = Some(LoginField::Password),
            other => command(form, other, cx),
        }
        return;
    };
    match code {
        KeyCode::Esc => form.focus = None,
        KeyCode::Tab | KeyCode::Down => form.focus = Some(focus.next()),
        KeyCode::BackTab | KeyCode::Up => form.focus = Some(focus.previous()),
        KeyCode::Enter => press(form, focus, cx),
        other => match form.focused_mut() {
            Some(field) => {
                field.handle_key(other);
            }
            None => command(form, other, cx),
        },
    }
}

/// A letter while no field takes it: `c` and `f` open Create profile and Reset password unless a sign-in is out, `q` quits and `!` opens Help.
fn command(form: &LoginForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('c') if !form.busy => register::open(cx),
        KeyCode::Char('f') if !form.busy => forgot_password::open(cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('!') => help::open(cx),
        _ => {}
    }
}

/// Enter: the identifier moves on to the password, the password and `Log in` submit, and Show or Hide toggles the mask.
fn press(form: &mut LoginForm, focus: LoginField, cx: &mut Context<'_>) {
    match focus {
        LoginField::Identifier => form.focus = Some(LoginField::Password),
        LoginField::Password | LoginField::LogIn => submit(form, cx),
        LoginField::ShowPassword => {
            let masked = form.password.is_masked();
            form.password.set_masked(!masked);
        }
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
