//! Import: the From, Mode and Board rows, the explainer, the input, and the last import's result.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::deck::ImportMode;

use super::{Focus, ImportView, Source, Target};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::pill::{self, PillState};
use crate::inbound::widgets::text_input::BOX_HEIGHT;
use crate::inbound::widgets::{action_bar, header};

/// The result list is at most this tall, border included.
const RESULT_HEIGHT: u16 = 10;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ImportView) {
    let palette = &shared.palette;
    let result_height = if view.result.is_some() {
        RESULT_HEIGHT
    } else {
        0
    };
    let input_height = match view.source {
        Source::Text => Constraint::Fill(1),
        Source::Archidekt => Constraint::Length(BOX_HEIGHT),
    };
    let [header, chips, explainer, input, results, _, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(2),
        input_height,
        Constraint::Length(result_height),
        Constraint::Fill(0),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());
    header::draw(f, palette, header, "Import", header::Corner::Hint);
    f.render_widget(Paragraph::new(chip_lines(palette, view)), chips);
    draw_explainer(f, palette, view, explainer);
    draw_input(f, palette, view, input);
    draw_result(f, palette, view, results);

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    let submit_style = if view.loading || view.blank() {
        palette.muted
    } else {
        palette.base.bold()
    };
    let mut bar = action_bar::line(
        palette,
        &[("Tab", "next"), ("h/l", "pick"), ("Esc", "Back")],
    );
    bar.spans.push(Span::styled("F2 ", submit_style));
    bar.spans
        .push(Span::styled(view.submit_label().to_owned(), palette.muted));
    f.render_widget(bar, keys);
}

/// The From, Mode and Board rows of chips.
fn chip_lines(palette: &Palette, view: &ImportView) -> Vec<Line<'static>> {
    let row = |label: &str, focus: Focus, options: Vec<(&str, bool)>| {
        let mut spans = vec![pill::label(palette, label)];
        spans.extend(options.into_iter().map(|(text, picked)| {
            pill::pill(
                text,
                palette.muted,
                PillState {
                    picked,
                    focused: picked && view.focus == focus,
                },
            )
        }));
        pill::row(spans)
    };
    vec![
        row(
            "From ",
            Focus::From,
            vec![
                ("Text", view.source == Source::Text),
                ("Archidekt", view.source == Source::Archidekt),
            ],
        ),
        row(
            "Mode ",
            Focus::Mode,
            vec![
                ("Add", view.mode == ImportMode::Add),
                ("Replace", view.mode == ImportMode::Replace),
            ],
        ),
        row(
            "Board",
            Focus::Board,
            Target::ALL
                .iter()
                .map(|t| (t.label(), *t == view.target))
                .collect(),
        ),
    ]
}

/// The line saying what the import does to which board.
fn draw_explainer(f: &mut Frame<'_>, palette: &Palette, view: &ImportView, area: Rect) {
    let (before, after) = view.explainer();
    let board_style = if view.mode.is_replace() {
        palette.warning.bold()
    } else {
        palette.accent.bold()
    };
    f.render_widget(
        Paragraph::new(Line::from(vec![
            Span::styled(before, palette.muted),
            Span::styled(view.target.word(), board_style),
            Span::styled(after, palette.muted),
        ]))
        .wrap(Wrap { trim: true }),
        area,
    );
}

/// The decklist or URL input, with the cursor when it has focus.
fn draw_input(f: &mut Frame<'_>, palette: &Palette, view: &ImportView, area: Rect) {
    let focused = view.focus == Focus::Input && !view.loading;
    match view.source {
        Source::Text => {
            view.text.draw(f, palette, area, "Paste decklist", focused);
            if focused {
                f.set_cursor_position(view.text.cursor_position(area));
            }
        }
        Source::Archidekt => {
            view.url
                .draw(f, palette, area, "Paste an Archidekt deck URL", focused);
            if focused {
                f.set_cursor_position(view.url.cursor_position(area));
            }
        }
    }
}

/// The last import's imported and unresolved cards, when there is one.
fn draw_result(f: &mut Frame<'_>, palette: &Palette, view: &ImportView, area: Rect) {
    let Some(result) = &view.result else {
        return;
    };
    let mut lines = Vec::new();
    if !result.imported.is_empty() {
        lines.push(Line::from(Span::styled("Imported", palette.muted)));
        lines.extend(result.imported.iter().map(|c| {
            Line::from(vec![
                Span::raw(format!("  {}", c.name)),
                Span::styled(format!(" x{}", c.quantity), palette.muted),
            ])
        }));
    }
    if !result.unresolved.is_empty() {
        lines.push(Line::from(Span::styled("Unresolved", palette.muted)));
        lines.extend(result.unresolved.iter().map(|c| {
            Line::from(vec![
                Span::styled(format!("  {}", c.name), palette.error),
                Span::styled(format!(" {}", c.reason), palette.muted),
            ])
        }));
    }
    f.render_widget(
        Paragraph::new(lines).block(Block::bordered().border_style(palette.border)),
        area,
    );
}
