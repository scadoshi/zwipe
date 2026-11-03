//! Create profile's keys: typing, each keystroke handed to the model, Tab and the arrows across the form, Enter on whatever has the focus, F2 for the password rules, and Esc to end typing. With no field active, Esc goes back to Login, `q` quits and `!` opens Help.

use crossterm::event::KeyCode;
use zwipe_ui::screens::register::Action;

use super::{RegisterField, RegisterForm, dispatch, edit};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;
use crate::inbound::widgets::hint::{self, HintTopic};

pub(crate) fn handle(form: &mut RegisterForm, code: KeyCode, cx: &mut Context<'_>) {
    let Some(focus) = form.focus else {
        match code {
            KeyCode::Esc => dispatch(form, Action::Back, cx),
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
            let Some((input, field)) = form.focused_mut() else {
                command(other, cx);
                return;
            };
            if input.handle_key(other) {
                let value = input.value().to_owned();
                dispatch(form, edit(field, value), cx);
            }
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
        RegisterField::Confirm | RegisterField::CreateProfile => {
            dispatch(form, Action::Submit, cx);
        }
    }
}
