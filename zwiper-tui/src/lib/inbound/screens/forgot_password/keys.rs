//! Reset password's keys: typing with live validation, Tab across the field and its button, Enter to send, and Esc to end typing. With no field active, Esc goes back to Login, `q` quits and `!` opens Help. Sending again while one is out is allowed, as in the app.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_core::domain::user::Email;

use super::{ForgotField, ForgotForm, Request};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::help;

pub(crate) fn handle(form: &mut ForgotForm, code: KeyCode, cx: &mut Context<'_>) {
    match (form.focus, code) {
        (None, KeyCode::Esc) => cx.back(),
        (Some(_), KeyCode::Esc) => form.focus = None,
        (_, KeyCode::Tab | KeyCode::Down) => form.step(Toward::Next),
        (_, KeyCode::BackTab | KeyCode::Up) => form.step(Toward::Previous),
        (Some(_), KeyCode::Enter) => send(form, cx),
        (Some(ForgotField::Email), other) => {
            if form.email.handle_key(other) && !form.email.is_empty() {
                form.touched = true;
            }
            form.validate();
        }
        (_, KeyCode::Char('q')) => cx.quit(),
        (_, KeyCode::Char('!')) => help::open(cx),
        _ => {}
    }
}

/// Sends the reset request for the email as typed, once it is a valid address.
fn send(form: &mut ForgotForm, cx: &mut Context<'_>) {
    if form.sent {
        return;
    }
    form.attempted = true;
    form.validate();
    if Email::new(form.email.value()).is_err() {
        return;
    }
    form.sending_since = Some(Instant::now());
    cx.request(Request::Send(form.email.value().to_owned()).into());
}
