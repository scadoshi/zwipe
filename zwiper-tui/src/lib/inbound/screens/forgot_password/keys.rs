//! Reset password's keys: typing, Tab across the field and its button, Enter to send, and Esc to end typing. With no field active, Esc goes back to Login, `q` quits and `!` opens Help. Every keystroke goes to the model as it lands; Enter and Esc back become the model's actions.

use crossterm::event::KeyCode;
use zwipe_ui::screens::forgot_password::Action;

use super::{ForgotField, ForgotForm, dispatch};
use crate::inbound::app::Context;
use crate::inbound::toward::Toward;
use crate::inbound::widgets::help;

pub(crate) fn handle(form: &mut ForgotForm, code: KeyCode, cx: &mut Context<'_>) {
    match (form.focus, code) {
        (None, KeyCode::Esc) => dispatch(form, Action::Back, cx),
        (Some(_), KeyCode::Esc) => form.focus = None,
        (_, KeyCode::Tab | KeyCode::Down) => form.step(Toward::Next),
        (_, KeyCode::BackTab | KeyCode::Up) => form.step(Toward::Previous),
        (Some(_), KeyCode::Enter) => dispatch(form, Action::Submit, cx),
        (Some(ForgotField::Email), other) => {
            if form.email.handle_key(other) {
                let value = form.email.value().to_owned();
                dispatch(form, Action::EditEmail(value), cx);
            }
        }
        (_, KeyCode::Char('q')) => cx.quit(),
        (_, KeyCode::Char('!')) => help::open(cx),
        _ => {}
    }
}
