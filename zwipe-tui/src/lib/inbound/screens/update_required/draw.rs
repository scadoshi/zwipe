//! The notice shown when the server no longer accepts this version.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};

use crate::inbound::app::Shared;
use crate::inbound::widgets::{action_bar, panel};

/// The command that installs the newest version.
const INSTALL: &str = "cargo install --git https://github.com/scadoshi/zwipe zwipe-tui";

/// The notice is this wide inside its panel, in cells.
const WIDTH: u16 = 70;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared) {
    let palette = &shared.palette;
    let body = Paragraph::new(vec![
        Line::from("Update required").style(palette.error).bold(),
        Line::default(),
        Line::from("This version of zwipe-tui is no longer supported. Update to keep building:"),
        Line::default(),
        Line::from(INSTALL).style(palette.accent),
    ])
    .wrap(Wrap { trim: true });
    let body_height = u16::try_from(body.line_count(WIDTH)).unwrap_or(u16::MAX);
    let inner = panel::draw(f, palette, f.area(), WIDTH, body_height.saturating_add(2));
    let [body_area, _, keys] = Layout::vertical([
        Constraint::Length(body_height),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    f.render_widget(body, body_area);
    f.render_widget(action_bar::line(palette, &[("q", "quit")]), keys);
}
