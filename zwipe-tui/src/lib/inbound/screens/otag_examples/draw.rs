//! The examples: the card on screen as text in a frame, or a ghost card while a page loads, or the end and empty lines; the details sheet over it when open.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::Stylize;
use ratatui::text::Line;
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use super::{EMPTY, END, ExamplesView, TITLE};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::{action_bar, card_text, skeleton};

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
    f.render_widget(Line::styled(TITLE, palette.accent).bold().centered(), title);
    let card_area = body.centered_horizontally(Constraint::Max(CARD_WIDTH));
    match view.current() {
        Some(card) => f.render_widget(
            Paragraph::new(card_text::lines(palette, &card.scryfall_data))
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .border_style(palette.accent)
                        .padding(Padding::horizontal(1)),
                ),
            card_area,
        ),
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
    let mut hints = vec![("Esc", "Back"), ("←/h", "Next"), ("↓/j", "Back one")];
    if view.current().is_some() {
        hints.push(("d", "Details"));
    }
    hints.push(("?", "Help"));
    f.render_widget(action_bar::line(palette, &hints), keys);
    if let Some(sheet) = &view.details {
        sheet.draw(f, palette, f.area());
    }
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
