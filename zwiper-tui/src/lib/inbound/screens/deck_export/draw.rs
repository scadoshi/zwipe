//! Export: the board toggles over the decklist.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Block, Paragraph};

use super::ExportView;
use crate::inbound::app::Shared;
use crate::inbound::widgets::pill::{self, PillState, pill};
use crate::inbound::widgets::{action_bar, header, skeleton};

/// Ghost rows while the deck loads.
const SKELETON_ROWS: usize = 10;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ExportView) {
    let palette = &shared.palette;
    let [header, chips, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());
    header::draw(f, palette, header, "Export", header::Corner::Hint);

    let block = Block::bordered()
        .title(" Decklist ")
        .border_style(palette.border);
    if view.failed {
        f.render_widget(
            Paragraph::new(Line::from("Could not load deck").style(palette.muted)).block(block),
            body,
        );
    } else if let Some(text) = view.text() {
        let boards = view.boards;
        let mut row = vec![pill::label(palette, "Export")];
        row.extend(
            [
                ("Deck", boards.deck),
                ("Maybe", boards.maybe),
                ("Side", boards.side),
            ]
            .iter()
            .enumerate()
            .map(|(i, (label, on))| {
                pill(
                    label,
                    palette.muted,
                    PillState {
                        picked: *on,
                        focused: i == view.focus,
                    },
                )
            }),
        );
        f.render_widget(pill::row(row), chips);
        f.render_widget(
            Paragraph::new(text).block(block).scroll((view.scroll, 0)),
            body,
        );
    } else {
        f.render_widget(
            Paragraph::new(skeleton::rows(palette, 1, usize::from(chips.width) / 3)),
            chips,
        );
        f.render_widget(
            Paragraph::new(skeleton::rows(
                palette,
                SKELETON_ROWS,
                usize::from(body.width.saturating_sub(2)),
            ))
            .block(block),
            body,
        );
    }

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    f.render_widget(
        action_bar::line(
            palette,
            &[
                ("h/l Space", "boards"),
                ("j/k", "scroll"),
                ("y", "Copy"),
                ("Esc", "Back"),
                ("q", "quit"),
            ],
        ),
        keys,
    );
}
