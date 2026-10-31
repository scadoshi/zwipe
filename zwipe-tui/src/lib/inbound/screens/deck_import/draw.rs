//! Import: the From, Mode and Board rows, the explainer, the input, and the last import's result.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::deck::ImportMode;

use super::{Focus, ImportView, Source, Target};
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar;
use crate::inbound::widgets::pill::{self, PillState, pill};
use crate::inbound::widgets::text_input::BOX_HEIGHT;

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
    f.render_widget(
        Line::from(vec![
            Span::styled("Import", palette.accent).bold(),
            Span::styled("  ? hint", palette.muted),
        ]),
        header,
    );

    let row = |label: &str, focus: Focus, options: Vec<(&str, bool)>| {
        let mut spans = vec![pill::label(palette, label)];
        spans.extend(options.into_iter().map(|(text, picked)| {
            pill(
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
    let lines = vec![
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
    ];
    f.render_widget(Paragraph::new(lines), chips);

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
        explainer,
    );

    let focused = view.focus == Focus::Input && !view.loading;
    match view.source {
        Source::Text => {
            view.text.draw(f, palette, input, "Paste decklist", focused);
            if focused {
                f.set_cursor_position(view.text.cursor_position(input));
            }
        }
        Source::Archidekt => {
            view.url
                .draw(f, palette, input, "Paste an Archidekt deck URL", focused);
            if focused {
                f.set_cursor_position(view.url.cursor_position(input));
            }
        }
    }

    if let Some(result) = &view.result {
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
            results,
        );
    }

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
