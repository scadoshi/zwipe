//! Create profile: the hero, the four fields with their error lines, `Create profile` under them, and the key bar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use zwipe_ui::screens::register::Field;

use super::{RegisterField, RegisterForm};
use crate::inbound::app::Shared;
use crate::inbound::screens::login;
use crate::inbound::screens::login::draw::{AuthScreen, Placement};
use crate::inbound::widgets::action_bar;
use crate::inbound::widgets::button::Button;
use crate::inbound::widgets::text_input::Label;

/// Create profile's key hints: the form's keys while the focus is in it, with Enter named for what it does from the focused stop; the way back into the form once Esc has ended typing.
pub(crate) fn keys(form: &RegisterForm) -> Vec<(&'static str, &'static str)> {
    let Some(focus) = form.focus else {
        return vec![
            ("Tab", "fields"),
            ("F1", "help"),
            ("Esc", "back"),
            ("q", "quit"),
        ];
    };
    let enter = match focus {
        RegisterField::Username | RegisterField::Email | RegisterField::Password => "next",
        RegisterField::Confirm | RegisterField::CreateProfile => "create profile",
        RegisterField::ShowPassword if form.password.is_masked() => "show",
        RegisterField::ShowConfirm if form.confirm.is_masked() => "show",
        RegisterField::ShowPassword | RegisterField::ShowConfirm => "hide",
    };
    let mut keys = vec![("Tab", "move"), ("Enter", enter)];
    if focus == RegisterField::Password {
        keys.push(("F2", "password rules"));
    }
    keys.extend([("F1", "help"), ("Esc", "done")]);
    keys
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &RegisterForm) {
    let palette = &shared.palette;
    let heights = [
        form.username.height(),
        form.email.height(),
        form.password.height(),
        form.confirm.height(),
    ];
    let body_height = heights.iter().fold(0_u16, |sum, h| sum.saturating_add(*h));
    let focus = form.focus;
    let row = [Button::new(form.model.submit_label()).disabled(form.model.is_loading())];
    let areas = login::draw::frame(
        f,
        shared,
        &form.hero,
        AuthScreen {
            body_height,
            placement: Placement::Top,
            keys: action_bar::line(palette, &keys(form)).centered(),
            buttons: &row,
            focused: focus.and_then(RegisterField::button),
            min_gap: 0,
        },
    );
    let [username, email, password, confirm] =
        Layout::vertical(heights.map(Constraint::Length)).areas(areas.body);
    login::draw::field(
        f,
        palette,
        username,
        &form.username,
        Field::Username.label(),
        focus == Some(RegisterField::Username),
    );
    login::draw::field(
        f,
        palette,
        email,
        &form.email,
        Field::Email.label(),
        focus == Some(RegisterField::Email),
    );
    login::draw::field(
        f,
        palette,
        password,
        &form.password,
        Label::About(Field::Password.label()),
        focus == Some(RegisterField::Password),
    );
    login::draw::show_button(
        f,
        palette,
        password,
        &form.password,
        focus == Some(RegisterField::ShowPassword),
    );
    login::draw::field(
        f,
        palette,
        confirm,
        &form.confirm,
        Field::ConfirmPassword.label(),
        focus == Some(RegisterField::Confirm),
    );
    login::draw::show_button(
        f,
        palette,
        confirm,
        &form.confirm,
        focus == Some(RegisterField::ShowConfirm),
    );
}
