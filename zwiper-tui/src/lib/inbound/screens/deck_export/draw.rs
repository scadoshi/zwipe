//! Export: the board toggles over the decklist.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::text::Line;
use ratatui::widgets::{Paragraph, Wrap};
use zwipe_ui::screens::deck_export::{BOARDS, Board, DECKLIST, LOAD_FAILED, Status};

use super::ExportView;
use crate::inbound::app::Shared;
use crate::inbound::widgets::pill::{self, PillState};
use crate::inbound::widgets::{action_bar, skeleton};

/// Ghost rows while the deck loads.
const SKELETON_ROWS: usize = 10;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ExportView) {
    let palette = &shared.palette;
    let bar = action_bar::line(
        palette,
        &[
            ("h/l Space", "boards"),
            ("j/k", "scroll"),
            ("y", "Copy"),
            ("Esc", "Back"),
            ("q", "quit"),
        ],
    );
    let bar = action_bar::with_help(palette, bar, action_bar::Hint::Question);
    let bar_height = action_bar::height(&bar, f.area().width.saturating_sub(2));
    let [chips, body, status, keys] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(bar_height),
    ])
    .horizontal_margin(1)
    .areas(f.area());

    let block = crate::inbound::widgets::panel::bordered(palette)
        .title(format!(" {DECKLIST} "))
        .border_style(palette.border);
    if view.model.status() == Status::Failed {
        f.render_widget(
            Paragraph::new(Line::from(LOAD_FAILED).style(palette.muted)).block(block),
            body,
        );
    } else if let Some(text) = view.model.text() {
        let mut row = vec![pill::label(palette, BOARDS)];
        row.extend(Board::ALL.iter().enumerate().map(|(i, board)| {
            pill::pill(
                board.label(),
                palette.muted,
                PillState {
                    picked: view.model.is_on(*board),
                    focused: i == view.focus,
                },
            )
        }));
        f.render_widget(pill::row(row), chips);
        f.render_widget(
            Paragraph::new(text)
                .block(block)
                .wrap(Wrap { trim: false })
                .scroll((view.scroll, 0)),
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
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
}
