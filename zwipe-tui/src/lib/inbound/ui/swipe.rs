//! The swipe screen: the deck at the top, the card in the middle, the keys at the bottom.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};
use zwipe_core::domain::card::Card;

use super::hints;
use crate::domain;
use crate::domain::card::{self, Face};
use crate::domain::theme::Palette;
use crate::inbound::app::{App, SwipeView};

/// The card is at most this wide, in cells, so it reads like a card on a wide terminal.
const CARD_WIDTH: u16 = 64;

/// The card's border and padding, across both sides.
const CARD_FRAME: u16 = 4;

pub(super) fn draw(f: &mut Frame<'_>, palette: &Palette, app: &App, view: &SwipeView) {
    let [header, body, footer, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    let count = domain::deck::card_count(&view.deck).saturating_add(view.added);
    let commander = view
        .deck
        .commander_name
        .as_deref()
        .map_or_else(String::new, |c| format!("  {c}"));
    f.render_widget(
        Line::from(vec![
            Span::styled(view.deck.name.to_string(), palette.accent).bold(),
            Span::styled(commander, palette.muted),
            Span::styled(
                format!("  {}", domain::deck::cards_label(count)),
                palette.muted,
            ),
        ]),
        header,
    );

    let current = view.stack.as_ref().and_then(|s| s.current());
    match (current, placeholder(view)) {
        (Some(card), _) => draw_card(f, palette, card, body),
        (None, Some(message)) => {
            f.render_widget(
                Paragraph::new(message.style(palette.muted))
                    .centered()
                    .wrap(Wrap { trim: true }),
                body.centered(Constraint::Max(CARD_WIDTH), Constraint::Length(1)),
            );
        }
        (None, None) => {}
    }

    let remaining = view
        .stack
        .as_ref()
        .map_or(0, crate::domain::swipe::Stack::remaining);
    let footer_text = if view.fetching && current.is_some() {
        format!("{remaining} left, finding more...")
    } else if current.is_some() {
        format!("{remaining} left")
    } else {
        String::new()
    };
    f.render_widget(
        Line::from(footer_text).style(palette.muted).centered(),
        footer,
    );
    f.render_widget(
        Line::from(app.status.clone())
            .style(palette.accent)
            .centered(),
        status,
    );
    f.render_widget(
        hints(
            palette,
            &[
                ("h", "pass"),
                ("l", "add"),
                ("m", "maybe"),
                ("u", "undo"),
                ("Esc", "deck"),
            ],
        )
        .centered(),
        keys,
    );
}

/// What to show when there is no card: loading, an error, or the end of the cards.
fn placeholder(view: &SwipeView) -> Option<Line<'static>> {
    if let Some(error) = &view.error {
        return Some(Line::from(error.clone()));
    }
    let message = match &view.stack {
        None => "Loading the deck...",
        Some(_) if view.fetching => "Finding cards...",
        Some(stack) if stack.is_exhausted() => "No more cards match this deck.",
        Some(_) => return None,
    };
    Some(Line::from(message))
}

/// Draws the card centered in `area`, as tall as its text.
fn draw_card(f: &mut Frame<'_>, palette: &Palette, card: &Card, area: Rect) {
    let data = &card.scryfall_data;
    let mut lines = Vec::new();
    for (i, face) in card::faces(data).iter().enumerate() {
        if i > 0 {
            lines.push(Line::default());
            lines.push(Line::from("//").style(palette.muted).centered());
            lines.push(Line::default());
        }
        lines.extend(face_lines(palette, face));
    }
    lines.push(Line::default());
    lines.push(Line::from(data.set_name.clone()).style(palette.muted));

    let card = Paragraph::new(lines).wrap(Wrap { trim: true }).block(
        Block::bordered()
            .border_style(palette.accent)
            .padding(Padding::horizontal(1)),
    );
    let width = area.width.min(CARD_WIDTH);
    let height =
        u16::try_from(card.line_count(width.saturating_sub(CARD_FRAME))).unwrap_or(u16::MAX);
    f.render_widget(
        card,
        area.centered(
            Constraint::Length(width),
            Constraint::Length(height.min(area.height)),
        ),
    );
}

fn face_lines(palette: &Palette, face: &Face) -> Vec<Line<'static>> {
    let mut title = vec![Span::raw(face.name.clone()).bold()];
    if !face.mana_cost.is_empty() {
        title.push(Span::raw("  "));
        title.extend(face.mana_cost.iter().map(|symbol| {
            let style = symbol
                .color
                .and_then(|i| palette.mana.get(i).copied())
                .map_or(palette.muted, ratatui::prelude::Style::bold);
            Span::styled(format!("{{{}}}", symbol.text), style)
        }));
    }

    let mut lines = vec![Line::from(title)];
    if !face.type_line.is_empty() {
        lines.push(Line::from(face.type_line.clone()).italic());
    }
    if !face.rules.is_empty() {
        lines.push(Line::default());
        lines.extend(face.rules.iter().map(|rule| Line::from(rule.clone())));
    }
    if let Some(stats) = &face.stats {
        lines.push(Line::default());
        lines.push(Line::from(stats.clone()).bold().right_aligned());
    }
    lines
}
