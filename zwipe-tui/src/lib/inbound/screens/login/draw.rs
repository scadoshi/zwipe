//! The sign-in screen: the ZWIPE logo over a username and password form.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Text};
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_core::domain::logo;

use super::{LoginField, LoginForm};
use crate::inbound::app::Shared;
use crate::inbound::widgets::measure::{height_of, width_of};
use crate::inbound::widgets::{action_bar, panel};

/// The form is this wide inside its panel, in cells.
const FORM_WIDTH: u16 = 48;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, form: &LoginForm) {
    let palette = &shared.palette;
    let logo_lines: Vec<&str> = logo::ZWIPE.lines().collect();
    let logo_width = logo_lines.iter().map(|l| width_of(l)).max().unwrap_or(0);
    let logo_height = height_of(logo_lines.len());

    let error = form.error.as_deref().map(|e| {
        Paragraph::new(Line::from(e.to_owned()).style(palette.error)).wrap(Wrap { trim: true })
    });
    let error_height = error
        .as_ref()
        .map_or(0, |e| height_of(e.line_count(FORM_WIDTH)));
    let form_height = 3_u16
        .saturating_add(3)
        .saturating_add(error_height)
        .saturating_add(2);

    let [logo_area, form_area] = Layout::vertical([
        Constraint::Length(logo_height.saturating_add(1)),
        Constraint::Length(form_height.saturating_add(2)),
    ])
    .areas(f.area().centered(
        Constraint::Max(logo_width.max(FORM_WIDTH.saturating_add(4))),
        Constraint::Length(logo_height.saturating_add(form_height).saturating_add(3)),
    ));

    let logo = Text::from(logo_lines.into_iter().map(Line::from).collect::<Vec<_>>())
        .style(palette.accent);
    f.render_widget(
        Paragraph::new(logo),
        logo_area.centered_horizontally(Constraint::Length(logo_width)),
    );

    let inner = panel::draw(f, palette, form_area, FORM_WIDTH, form_height);
    let [identifier_area, password_area, error_area, _, hints_area] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(3),
        Constraint::Length(error_height),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);

    form.identifier.draw(
        f,
        palette,
        identifier_area,
        "Username or email",
        form.focus == LoginField::Identifier,
    );
    form.password.draw(
        f,
        palette,
        password_area,
        "Password",
        form.focus == LoginField::Password,
    );
    let cursor = match form.focus {
        LoginField::Identifier => form.identifier.cursor_position(identifier_area),
        LoginField::Password => form.password.cursor_position(password_area),
    };
    f.set_cursor_position(cursor);

    if let Some(error) = error {
        f.render_widget(error, error_area);
    }
    let keys = if form.busy {
        Line::from("Signing in...").style(palette.muted).bold()
    } else {
        action_bar::line(
            palette,
            &[("Tab", "switch"), ("Enter", "sign in"), ("Esc", "quit")],
        )
    };
    f.render_widget(keys, hints_area);
}
