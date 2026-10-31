//! Reset password: the header, the hero, the email field with its spinner while sending, or the `Check your email` panel once sent; and the action bar.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use super::{ForgotField, ForgotForm, SENT_BODY, SENT_TITLE};
use crate::inbound::app::Shared;
use crate::inbound::screens::login;
use crate::inbound::screens::login::draw::{AuthScreen, Placement};
use crate::inbound::widgets::button::Button;
use crate::inbound::widgets::{action_bar, spinner};

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &ForgotForm) {
    let palette = &shared.palette;
    let body_height = if form.sent {
        4
    } else {
        form.email.height().saturating_add(1)
    };
    let mut row = vec![Button::new("Back to login")];
    if !form.sent {
        row.push(Button::new("Send reset link").primary());
    }
    let focused = match form.focus {
        ForgotField::BackToLogin => Some(0),
        ForgotField::SendResetLink => Some(1),
        ForgotField::Email => None,
    };
    let keys = action_bar::line(
        palette,
        &[
            ("Tab", "move"),
            ("Enter", "select"),
            ("F1", "help"),
            ("Esc", "back"),
        ],
    )
    .centered();
    let areas = login::draw::frame(
        f,
        shared,
        &form.hero,
        AuthScreen {
            title: "Reset password",
            body_height,
            placement: Placement::Centered,
            keys,
            buttons: &row,
            focused,
        },
    );
    if form.sent {
        let block = crate::inbound::widgets::panel::bordered()
            .border_style(palette.border)
            .title(Line::from(SENT_TITLE).bold().centered());
        f.render_widget(
            Paragraph::new(Line::from(SENT_BODY).centered())
                .wrap(Wrap { trim: true })
                .block(block),
            areas.body,
        );
    } else {
        let [email, spin] = Layout::vertical([
            Constraint::Length(form.email.height()),
            Constraint::Length(1),
        ])
        .areas(areas.body);
        login::draw::field(
            f,
            palette,
            email,
            &form.email,
            "Email",
            form.focus == ForgotField::Email,
        );
        if let Some(since) = form.sending_since {
            f.render_widget(
                Line::from(spinner::frame(since, Instant::now()))
                    .style(palette.accent)
                    .centered(),
                spin,
            );
        }
    }
}
