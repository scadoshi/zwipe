//! The printings: one row each with set name, set code, collector number and USD price, the one the card is on marked; ghost rows while they load.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;

use super::{Mode, PrintingsView, TITLE};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::{action_bar, item_list, skeleton};

/// Ghost rows while the printings load.
const GHOST_ROWS: usize = 6;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &PrintingsView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let [title, name, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(2),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .areas(area);
    f.render_widget(Line::styled(TITLE, palette.accent).bold().centered(), title);
    f.render_widget(
        Line::from(view.card.scryfall_data.name.clone())
            .bold()
            .centered(),
        name,
    );
    if view.loading {
        f.render_widget(
            Paragraph::new(skeleton::rows(palette, GHOST_ROWS, usize::from(body.width))),
            body,
        );
    } else {
        let current = view.card.scryfall_data.id;
        let items: Vec<Vec<Line<'static>>> = view
            .printings
            .iter()
            .map(|p| vec![row(palette, p, p.scryfall_data.id == current)])
            .collect();
        item_list::draw(f, palette, body, &items, view.selected);
    }
    let mut hints = vec![("Esc", "Back"), ("j/k", "Move"), ("i", "Image")];
    if view.mode == Mode::Pick && view.changed() {
        hints.push(("Enter", "Save"));
    }
    f.render_widget(action_bar::line(palette, &hints), keys);
}

/// One printing: set name, set code and collector number, price, and a mark on the one the card is on.
fn row(palette: &Palette, printing: &Card, current: bool) -> Line<'static> {
    let data = &printing.scryfall_data;
    let mut spans = vec![
        Span::raw(data.set_name.clone()).bold(),
        Span::styled(
            format!("  {} #{}", data.set.to_uppercase(), data.collector_number),
            palette.muted,
        ),
    ];
    if let Some(price) = deck_metrics::card_price(data, PriceCurrency::Usd) {
        spans.push(Span::styled(
            format!("  {}", PriceCurrency::Usd.format_amount(price)),
            palette.muted,
        ));
    }
    if current {
        spans.push(Span::styled("  current", palette.accent));
    }
    Line::from(spans)
}
