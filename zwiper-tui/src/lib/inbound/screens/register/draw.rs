//! Create profile: the header, the hero, the four fields with their error lines, and the action bar.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};

use super::{RegisterField, RegisterForm};
use crate::inbound::app::Shared;
use crate::inbound::screens::login;
use crate::inbound::screens::login::draw::Placement;
use crate::inbound::widgets::action_bar::{self, Button};

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
    let mut keys = vec![("Tab", "move"), ("Enter", "select")];
    if focus == RegisterField::Password {
        keys.push(("F2", "password rules"));
    }
    keys.extend([("F1", "help"), ("Esc", "back")]);
    let areas = login::draw::frame(
        f,
        shared,
        &form.hero,
        "Create profile",
        body_height,
        Placement::Top,
        action_bar::line(palette, &keys).centered(),
    );
    let [username, email, password, confirm] =
        Layout::vertical(heights.map(Constraint::Length)).areas(areas.body);
    login::draw::field(
        f,
        palette,
        username,
        &form.username,
        "Username",
        focus == RegisterField::Username,
    );
    login::draw::field(
        f,
        palette,
        email,
        &form.email,
        "Email",
        focus == RegisterField::Email,
    );
    login::draw::field(
        f,
        palette,
        password,
        &form.password,
        "Password ?",
        focus == RegisterField::Password,
    );
    login::draw::show_button(
        f,
        palette,
        password,
        &form.password,
        focus == RegisterField::ShowPassword,
    );
    login::draw::field(
        f,
        palette,
        confirm,
        &form.confirm,
        "Confirm password",
        focus == RegisterField::Confirm,
    );
    login::draw::show_button(
        f,
        palette,
        confirm,
        &form.confirm,
        focus == RegisterField::ShowConfirm,
    );
    let create = if form.busy {
        "Creating..."
    } else {
        "Create profile"
    };
    let row = [
        Button::new("Back to login").disabled(form.busy),
        Button::new(create).disabled(form.busy),
    ];
    f.render_widget(
        action_bar::buttons(palette, &row, focus.button()).centered(),
        areas.buttons,
    );
}
