//! Reset password's keys: typing with live validation, Tab across the field and buttons, Enter to send or go back. Sending again while one is out is allowed, as in the app.

use std::time::Instant;

use crossterm::event::KeyCode;
use zwipe_core::domain::user::Email;

use super::{ForgotField, ForgotForm, Request};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

pub(crate) fn handle(form: &mut ForgotForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Esc => cx.back(),
        KeyCode::Tab | KeyCode::Down => form.step(true),
        KeyCode::BackTab | KeyCode::Up => form.step(false),
        KeyCode::Right if form.focus != ForgotField::Email => form.step(true),
        KeyCode::Left if form.focus != ForgotField::Email => form.step(false),
        KeyCode::Enter => match form.focus {
            ForgotField::BackToLogin => cx.back(),
            ForgotField::Email | ForgotField::SendResetLink => send(form, cx),
        },
        KeyCode::Char('!') if form.focus != ForgotField::Email => help::open(cx),
        other if form.focus == ForgotField::Email => {
            if form.email.handle_key(other) && !form.email.is_empty() {
                form.touched = true;
            }
            form.validate();
        }
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
