//! The examples: the card on screen as text in a frame, or a ghost card while a page loads, or the end and empty lines; the details sheet over it when open.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Style;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Widget, Wrap};
use zwipe_core::domain::card::Card;

use super::{EMPTY, END, ExamplesView, TITLE};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::card_motion::{self, CardMotion, Phase};
use crate::inbound::widgets::{action_bar, card_text, header, skeleton};

/// The card's frame is at most this wide, as the image preview's text fallback.
const CARD_WIDTH: u16 = 48;

/// Ghost rows inside the loading card.
const GHOST_ROWS: usize = 6;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ExamplesView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let [title, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    header::draw(f, palette, title, TITLE, header::Corner::Hint);
    let card_area = body.centered_horizontally(Constraint::Max(CARD_WIDTH));
    let now = Instant::now();
    let motion = view.motion.as_ref().filter(|m| m.running(now));
    // While a card comes back after a step back, the card it covers rests under it.
    let resting = match motion.map(CardMotion::phase) {
        Some(Phase::Enter) => view.under(),
        _ => view.current(),
    };
    match resting {
        Some(card) => f.render_widget(card_widget(palette, card), card_area),
        None if view.current().is_some() => {}
        None if view.loading => draw_ghost_card(f, palette, card_area),
        None => {
            let message = if view.cards.is_empty() { EMPTY } else { END };
            f.render_widget(
                Paragraph::new(Line::styled(message, palette.muted))
                    .centered()
                    .wrap(Wrap { trim: true }),
                body.centered_vertically(Constraint::Length(1)),
            );
        }
    }
    if let Some(motion) = motion {
        let moving = match motion.phase() {
            Phase::Exit(card) => Some(card),
            Phase::Enter => view.current(),
        };
        if let Some(card) = moving
            && let Some(placement) = motion.placement(now, card_area, body)
        {
            let widget = card_widget(palette, card);
            card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| {
                widget.render(card_area, buf);
            });
        }
    }
    f.render_widget(action_bar::styled_line(&key_hints(palette, view)), keys);
    if let Some(sheet) = &view.details {
        sheet.draw(f, palette, f.area());
    }
}

/// The card as text in its frame.
fn card_widget(palette: &Palette, card: &Card) -> Paragraph<'static> {
    Paragraph::new(card_text::lines(palette, &card.scryfall_data))
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .border_style(palette.accent)
                .padding(Padding::horizontal(1)),
        )
}

/// Esc, then the two swipes that commit here in the app's swipe colors (left skips to the next card, down undoes a step), then Details and the hint.
fn key_hints(palette: &Palette, view: &ExamplesView) -> Vec<(&'static str, &'static str, Style)> {
    let mut keys = vec![
        ("Esc", "Back", palette.muted),
        ("←", "skip", palette.error),
        ("↓", "undo", palette.tertiary),
    ];
    if view.current().is_some() {
        keys.push(("d", "Details", palette.muted));
    }
    keys.push(("?", "Help", palette.muted));
    keys
}

/// A card frame of ghost rows, with no text in it.
fn draw_ghost_card(f: &mut Frame<'_>, palette: &Palette, area: Rect) {
    let block = Block::bordered()
        .border_style(palette.muted)
        .padding(Padding::horizontal(1));
    let width = usize::from(block.inner(area).width);
    f.render_widget(
        Paragraph::new(skeleton::rows(palette, GHOST_ROWS, width)).block(block),
        area,
    );
}
