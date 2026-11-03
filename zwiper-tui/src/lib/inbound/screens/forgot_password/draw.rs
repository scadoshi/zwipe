//! Reset password: the hero, the email field with `Send reset link` under it and its spinner between the two while sending, or the `Check your email` panel once sent; and the key bar.

use std::time::Instant;

use ratatui::Frame;
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_ui::screens::forgot_password::{EMAIL, SEND, SENT_BODY, SENT_TITLE};

use super::{ForgotField, ForgotForm};
use crate::inbound::app::Shared;
use crate::inbound::screens::login;
use crate::inbound::screens::login::draw::{AuthScreen, Placement};
use crate::inbound::widgets::button::Button;
use crate::inbound::widgets::{action_bar, spinner};

/// Reset password's key hints: the form's keys while the focus is in it, with Enter named for what it does; the way back into the form, if there is one, once Esc has ended typing or the request is in.
pub(crate) fn keys(form: &ForgotForm) -> Vec<(&'static str, &'static str)> {
    if form.focus.is_some() {
        return vec![
            ("Tab", "move"),
            ("Enter", "send link"),
            ("F1", "help"),
            ("Esc", "done"),
        ];
    }
    let mut keys = Vec::new();
    if !form.stops().is_empty() {
        keys.push(("Tab", "fields"));
    }
    keys.extend([("F1", "help"), ("Esc", "back"), ("q", "quit")]);
    keys
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &ForgotForm) {
    let palette = &shared.palette;
    let sent = form.model.is_sent();
    let body_height = if sent { 4 } else { form.email.height() };
    let row = if sent {
        Vec::new()
    } else {
        vec![Button::new(SEND)]
    };
    let focused = (form.focus == Some(ForgotField::SendResetLink)).then_some(0);
    let keys = action_bar::line(palette, &keys(form)).centered();
    let areas = login::draw::frame(
        f,
        shared,
        &form.hero,
        AuthScreen {
            body_height,
            placement: Placement::Centered,
            keys,
            buttons: &row,
            focused,
            min_gap: 1,
        },
    );
    if sent {
        let block = crate::inbound::widgets::panel::bordered(palette)
            .border_style(palette.border)
            .title(Line::from(SENT_TITLE).bold().centered());
        f.render_widget(
            Paragraph::new(Line::from(SENT_BODY).centered())
                .wrap(Wrap { trim: true })
                .block(block),
            areas.body,
        );
    } else {
        login::draw::field(
            f,
            palette,
            areas.body,
            &form.email,
            EMAIL,
            form.focus == Some(ForgotField::Email),
        );
        if form.model.is_loading() {
            f.render_widget(
                Line::from(spinner::frame(form.spin_since, Instant::now()))
                    .style(palette.accent)
                    .centered(),
                areas.gap,
            );
        }
    }
}
