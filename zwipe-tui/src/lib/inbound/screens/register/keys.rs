//! Create profile's keys: typing with live validation, Tab and the arrows across the fields and buttons, Enter on whatever has the focus, F2 for the password rules.

use crossterm::event::KeyCode;
use zwipe_core::http::contracts::metrics::AnonymousEventKind;

use super::{RegisterField, RegisterForm, Request};
use crate::inbound::app::Context;
use crate::inbound::screens::{self, Password};
use crate::inbound::widgets::help;
use crate::inbound::widgets::hint::{self, HintTopic};

pub(crate) fn handle(form: &mut RegisterForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Tab | KeyCode::Down => form.focus = form.focus.next(),
        KeyCode::BackTab | KeyCode::Up => form.focus = form.focus.previous(),
        KeyCode::F(2) if form.focus == RegisterField::Password => {
            hint::topic(cx, HintTopic::Password);
        }
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
            if let Some((field, index)) = form.focused_mut() {
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
}

/// Enter: a field moves on, the confirmation submits, Show and Hide toggle their field, and a button does what it says unless the registration is out.
fn press(form: &mut RegisterForm, cx: &mut Context<'_>) {
    match form.focus {
        RegisterField::Username | RegisterField::Email | RegisterField::Password => {
            form.focus = form.focus.next();
        }
        RegisterField::ShowPassword => {
            let masked = form.password.is_masked();
            form.password.set_masked(!masked);
        }
        RegisterField::ShowConfirm => {
            let masked = form.confirm.is_masked();
            form.confirm.set_masked(!masked);
        }
        RegisterField::BackToLogin if !form.busy => cx.back(),
        RegisterField::Confirm | RegisterField::CreateProfile => submit(form, cx),
        RegisterField::BackToLogin => {}
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
