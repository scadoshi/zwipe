//! The home screen: the logo, the counters, the color map and the featured flavor card.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span, Text};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::logo;

use super::charts::row;
use super::{height_of, hints, width_of};
use crate::domain::decode;
use crate::domain::home::{self, Figures};
use crate::domain::theme::Palette;
use crate::inbound::app::{App, HomeView};

/// The two panels sit side by side from this width, in cells.
const SIDE_BY_SIDE: u16 = 90;

/// The panels need at least this many rows.
const PANEL_HEIGHT: u16 = 11;

/// Pairs listed under the color map.
const PAIRS_SHOWN: usize = 3;

pub(super) fn draw(f: &mut Frame<'_>, palette: &Palette, app: &App, view: &HomeView) {
    let area = f.area().inner(ratatui::layout::Margin::new(1, 1));
    let logo_text = pick_logo(area);
    let logo_height = logo_text.map_or(0, |l| height_of(l.lines().count()));

    let [title, logo_area, _, figures, scope, _, panels, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(logo_height),
        Constraint::Length(u16::from(logo_height > 0)),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    f.render_widget(
        Line::from(vec![
            Span::styled("Home", palette.accent).bold(),
            Span::styled(
                format!("  {}", app.username.as_deref().unwrap_or_default()),
                palette.muted,
            ),
        ]),
        title,
    );
    if let Some(text) = logo_text {
        let width = text.lines().map(width_of).max().unwrap_or(0);
        let frame = decode::at(text, view.logo_elapsed());
        let art = Text::from(
            frame
                .lines()
                .map(|l| Line::from(l.to_owned()))
                .collect::<Vec<_>>(),
        )
        .style(palette.accent);
        f.render_widget(
            Paragraph::new(art),
            logo_area.centered_horizontally(Constraint::Length(width)),
        );
    }

    f.render_widget(figures_line(palette, view.figures()).centered(), figures);
    f.render_widget(
        Line::from(vec![
            Span::styled(view.scope.label(), palette.accent).bold(),
            Span::styled("  s to switch", palette.muted),
        ])
        .centered(),
        scope,
    );

    draw_panels(f, palette, view, panels);
    f.render_widget(Line::from(app.status.clone()).style(palette.muted), status);
    f.render_widget(
        hints(
            palette,
            &[
                ("d", "decks"),
                ("a", "account"),
                ("c", "config"),
                ("s", "everybody/only me"),
                ("r", "refresh"),
                ("O", "sign out"),
                ("q", "quit"),
            ],
        ),
        keys,
    );
}

/// The full logo when it fits with room for the panels, the compact Z when only that fits, else none.
fn pick_logo(area: Rect) -> Option<&'static str> {
    let fits = |art: &str| {
        let height = height_of(art.lines().count());
        let width = art.lines().map(width_of).max().unwrap_or(0);
        area.width >= width && area.height >= height.saturating_add(PANEL_HEIGHT).saturating_add(8)
    };
    [logo::ZWIPE, logo::Z].into_iter().find(|art| fits(art))
}

fn figures_line(palette: &Palette, figures: Option<Figures>) -> Line<'static> {
    let Some(figures) = figures else {
        return Line::from("Counting...").style(palette.muted);
    };
    let figure = |n: i64, label: &str| {
        [
            Span::styled(home::grouped(n), palette.accent).bold(),
            Span::styled(format!(" {label}   "), palette.muted),
        ]
    };
    Line::from(
        [
            figure(figures.decks_created, "Decks created"),
            figure(figures.searches, "Searches run"),
            figure(figures.cards_swiped, "Cards swiped"),
        ]
        .concat(),
    )
}

fn draw_panels(f: &mut Frame<'_>, palette: &Palette, view: &HomeView, area: Rect) {
    let flavor = view.flavor.as_ref().filter(|card| {
        card.scryfall_data
            .flavor_text
            .as_deref()
            .is_some_and(|t| !t.trim().is_empty())
    });
    let Some(card) = flavor else {
        draw_color_map(f, palette, view, area);
        return;
    };
    let [map_area, flavor_area] = if area.width >= SIDE_BY_SIDE {
        Layout::horizontal([Constraint::Percentage(50), Constraint::Percentage(50)]).areas(area)
    } else {
        Layout::vertical([Constraint::Fill(1), Constraint::Fill(1)]).areas(area)
    };
    draw_color_map(f, palette, view, map_area);
    draw_flavor(f, palette, card, flavor_area);
}

fn draw_color_map(f: &mut Frame<'_>, palette: &Palette, view: &HomeView, area: Rect) {
    let block = Block::bordered()
        .title(" Color map ")
        .border_style(palette.border);
    let inner_width = usize::from(area.width.saturating_sub(2));
    let map = view
        .play_style
        .as_ref()
        .filter(|s| s.deck_count > 0)
        .and_then(|s| s.color_map.as_ref())
        .filter(|m| home::has_data(m));

    let lines = match (&view.play_style, map) {
        (_, Some(map)) => {
            let rows = home::color_rows(map);
            let max = rows.iter().map(|r| r.cards).max().unwrap_or(0);
            let bar_width = inner_width.saturating_sub(10);
            let mut lines: Vec<Line<'static>> = rows
                .iter()
                .map(|r| {
                    let style = r
                        .color
                        .and_then(|i| palette.mana.get(i).copied())
                        .unwrap_or(palette.muted);
                    let pct = r.cards.saturating_mul(100).checked_div(max).unwrap_or(0);
                    row(
                        Span::styled(r.label, style.bold()),
                        1,
                        pct,
                        bar_width,
                        style,
                        vec![Span::raw(r.cards.to_string())],
                    )
                })
                .collect();
            let pairs = home::top_pairs(map, PAIRS_SHOWN);
            if !pairs.is_empty() {
                lines.push(Line::default());
                let listed: Vec<String> = pairs
                    .iter()
                    .map(|(pair, decks)| format!("{pair} {decks}"))
                    .collect();
                lines.push(
                    Line::from(format!("Most shared: {}", listed.join("  "))).style(palette.muted),
                );
            }
            lines
        }
        (Some(_), None) => {
            vec![Line::from("Build a deck to see your playstyle").style(palette.muted)]
        }
        (None, None) if view.play_style_failed => {
            vec![Line::from("Could not load your playstyle").style(palette.error)]
        }
        (None, None) => vec![Line::from("Loading...").style(palette.muted)],
    };
    f.render_widget(Paragraph::new(lines).block(block), area);
}

fn draw_flavor(f: &mut Frame<'_>, palette: &Palette, card: &Card, area: Rect) {
    let data = &card.scryfall_data;
    let quote = data.flavor_text.clone().unwrap_or_default();
    let mut name = vec![Span::raw(data.name.clone()).bold()];
    if let Some(price) = deck_metrics::card_price(data, PriceCurrency::Usd) {
        name.push(Span::styled(
            format!("  {}", PriceCurrency::Usd.format_amount(price)),
            palette.muted,
        ));
    }
    let mut lines: Vec<Line<'static>> = quote
        .lines()
        .map(|l| Line::from(l.to_owned()).italic())
        .collect();
    lines.push(Line::default());
    lines.push(Line::from(name));
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(" Featured flavor ")
                .border_style(palette.border),
        ),
        area,
    );
}
