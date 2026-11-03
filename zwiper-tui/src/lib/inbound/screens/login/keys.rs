//! Login's keys: typing in the fields, Tab and the arrows across the form, Enter on whatever has the focus, and Esc to end typing. With no field taking letters, `c` opens Create profile, `f` Reset password, `q` quits and `!` opens Help.

use crossterm::event::KeyCode;
use zwipe_ui::screens::login::Action;

use super::{LoginField, LoginForm, dispatch, edit};
use crate::inbound::app::Context;
use crate::inbound::widgets::help;

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
            Some((input, field)) => {
                if input.handle_key(other) {
                    let value = input.value().to_owned();
                    dispatch(form, edit(field, value), cx);
                }
            }
            None => command(form, other, cx),
        },
    }
}

/// A letter while no field takes it: `c` and `f` ask the model for Create profile and Reset password, which it holds while a sign-in is out, `q` quits and `!` opens Help.
fn command(form: &mut LoginForm, code: KeyCode, cx: &mut Context<'_>) {
    match code {
        KeyCode::Char('c') => dispatch(form, Action::CreateProfile, cx),
        KeyCode::Char('f') => dispatch(form, Action::ForgotPassword, cx),
        KeyCode::Char('q') => cx.quit(),
        KeyCode::Char('!') => help::open(cx),
        _ => {}
    }
}

/// Enter: the identifier moves on to the password, the password and `Log in` submit, and Show or Hide toggles the mask.
fn press(form: &mut LoginForm, focus: LoginField, cx: &mut Context<'_>) {
    match focus {
        LoginField::Identifier => form.focus = Some(LoginField::Password),
        LoginField::Password | LoginField::LogIn => dispatch(form, Action::Submit, cx),
        LoginField::ShowPassword => {
            let masked = form.password.is_masked();
            form.password.set_masked(!masked);
        }
    }
}
