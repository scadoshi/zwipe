//! Create profile's keys: typing with live validation, Tab and the arrows across the form, Enter on whatever has the focus, F2 for the password rules, and Esc to end typing. With no field active, Esc goes back to Login, `q` quits and `!` opens Help.

use crossterm::event::KeyCode;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use super::{RegisterField, RegisterForm, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::{self, Password};
use crate::inbound::widgets::help;
use crate::inbound::widgets::hint::{self, HintTopic};

pub(crate) fn handle(form: &mut RegisterForm, code: KeyCode, cx: &mut Context<'_>) {
    let Some(focus) = form.focus else {
        match code {
            KeyCode::Esc => cx.back(),
            KeyCode::Tab | KeyCode::Down => form.focus = Some(RegisterField::Username),
            KeyCode::BackTab | KeyCode::Up => form.focus = Some(RegisterField::Confirm),
            other => command(other, cx),
        }
        return;
    };
    match code {
        KeyCode::Esc => form.focus = None,
        KeyCode::Tab | KeyCode::Down => form.focus = Some(focus.next()),
        KeyCode::BackTab | KeyCode::Up => form.focus = Some(focus.previous()),
        KeyCode::F(2) if focus == RegisterField::Password => {
            hint::topic(cx, HintTopic::Password);
        }
        KeyCode::Enter => press(form, focus, cx),
        other => {
            let Some((field, index)) = form.focused_mut() else {
                command(other, cx);
                return;
            };
            if field.handle_key(other)
                && !field.is_empty()
                && let Some(touched) = form.touched.get_mut(index)
            {
                *touched = true;
            }
            form.validate();
        }
    }
}

/// A letter while no field takes it: `q` quits and `!` opens Help.
fn command(code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('!') => help::open(cx),
        _ => {}
    }
}

/// Enter: a field moves on, the confirmation and `Create profile` submit, and Show and Hide toggle their field.
fn press(form: &mut RegisterForm, focus: RegisterField, cx: &mut Context<'_>) {
    match focus {
        RegisterField::Username | RegisterField::Email | RegisterField::Password => {
            form.focus = Some(focus.next());
        }
        RegisterField::ShowPassword => {
            let masked = form.password.is_masked();
            form.password.set_masked(!masked);
        }
        RegisterField::ShowConfirm => {
            let masked = form.confirm.is_masked();
            form.confirm.set_masked(!masked);
        }
        RegisterField::Confirm | RegisterField::CreateProfile => submit(form, cx),
    }
}

/// Checks every field; when all pass, tells the server a registration was submitted and sends it.
fn submit(form: &mut RegisterForm, cx: &mut Context<'_>) {
    if form.busy {
        return;
    }
    form.attempted = true;
    form.validate();
    if !form.valid() {
        return;
    }
    form.busy = true;
    cx.request(screens::Request::Record(
        AnonymousEventKind::RegisterSubmitted,
    ));
    cx.request(
        Request::Register {
            username: form.username.value().to_owned(),
            email: form.email.value().trim().to_owned(),
            password: Password(form.password.value().to_owned()),
        }
        .into(),
    );
}
