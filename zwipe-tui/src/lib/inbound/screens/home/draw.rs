//! Home: the hero (logo, figures and chips), the color map panel and the featured flavor panel, then the action bar.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;

use super::keys::has_flavor;
use super::{HomeView, Overlay};
use crate::domain::color_map::{ColorMapCounts, ColorMapModel, ghost_model};
use crate::domain::home;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::measure::height_of;
use crate::inbound::widgets::{action_bar, color_map, entrance, figures, logo, skeleton, stagger};

/// The two lower panels sit side by side from this width, in cells.
const SIDE_BY_SIDE: u16 = 90;

/// Rows kept for the panels under the hero before the logo may take room.
const PANEL_ROOM: u16 = 12;

/// A chip: its label padded, in `style` reversed.
pub(crate) fn chip(label: &str, style: Style) -> Span<'static> {
    Span::styled(format!(" {label} "), style.reversed())
}

/// A key letter and its label, as a button reads in the TUI.
pub(crate) fn button(palette: &Palette, key: &str, label: &str) -> Vec<Span<'static>> {
    vec![
        Span::raw(key.to_owned()).bold(),
        Span::styled(format!(" {label}"), palette.accent),
    ]
}

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &HomeView) {
    let palette = &shared.palette;
    let area = f.area().inner(Margin::new(1, 0));
    let logo_text = pick_logo(area);
    let logo_height = logo_text.map_or(0, |l| height_of(l.lines().count()));
    let hero_height = logo_height
        .saturating_add(u16::from(logo_height > 0))
        .saturating_add(figures::HEIGHT)
        .saturating_add(1)
        .saturating_add(2);

    let [title, hero, panels, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(hero_height),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    f.render_widget(
        Line::styled("Home", palette.accent).bold().centered(),
        title,
    );
    let since = view.opened.elapsed();
    if let Some(hero) = stagger::area(hero, since, Duration::ZERO).map(|r| r.intersection(f.area()))
    {
        draw_hero(f, shared, view, logo_text, hero);
        stagger::fade(f, hero, since, Duration::ZERO, entrance::DURATION);
    }
    if let Some(panels) =
        stagger::area(panels, since, stagger::DELAY).map(|r| r.intersection(f.area()))
    {
        draw_panels(f, shared, view, panels);
        stagger::fade(f, panels, since, stagger::DELAY, entrance::DURATION);
    }
    f.render_widget(Line::styled(shared.status.clone(), palette.muted), status);
    f.render_widget(
        action_bar::line(
            palette,
            &[
                ("d", "Decks"),
                ("a", "Account"),
                ("c", "Config"),
                ("q", "quit"),
            ],
        ),
        keys,
    );
    match &view.overlay {
        Some(Overlay::Info(info)) => info.draw(f, palette),
        Some(Overlay::Buy(dialog)) => dialog.draw(f, palette),
        None => {}
    }
}

/// The full logo when it fits with room for the panels, the compact Z when only that fits, else none.
fn pick_logo(area: Rect) -> Option<&'static str> {
    logo::fitting(
        area.width.saturating_sub(2),
        area.height.saturating_sub(PANEL_ROOM).saturating_sub(9),
    )
}

fn draw_hero(
    f: &mut Frame<'_>,
    shared: &Shared,
    view: &HomeView,
    logo_text: Option<&str>,
    area: Rect,
) {
    let palette = &shared.palette;
    let block = Block::bordered().border_style(palette.border);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let logo_height = logo_text.map_or(0, |l| height_of(l.lines().count()));
    let [logo_area, _, strip, chips] = Layout::vertical([
        Constraint::Length(logo_height),
        Constraint::Length(u16::from(logo_height > 0)),
        Constraint::Length(figures::HEIGHT),
        Constraint::Length(1),
    ])
    .areas(inner);
    if let Some(text) = logo_text {
        logo::draw(
            f,
            logo_area,
            text,
            palette.logo,
            view.logo_started.map(|_| view.logo_elapsed()),
        );
    }
    figures::draw(
        f,
        palette,
        strip,
        super::figures_for(shared),
        super::since_open(view),
        view.since_arrival(),
    );
    let settling = view
        .scope_at
        .is_some_and(|t| stagger::entering(t.elapsed(), Duration::ZERO, stagger::SCOPE_IN));
    let scope_in = |style: Style| if settling { style.dim() } else { style };
    let mut spans = Vec::new();
    if super::live(shared) {
        spans.push(chip("Live", scope_in(palette.add)));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw("s ").bold());
    spans.push(chip(shared.caches.scope.label(), scope_in(palette.accent)));
    spans.push(Span::raw("  "));
    spans.extend(button(palette, "v", "Activity"));
    f.render_widget(Line::from(spans).centered(), chips);
}

fn draw_panels(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, area: Rect) {
    let [map_area, flavor_area] = if area.width >= SIDE_BY_SIDE {
        Layout::horizontal([Constraint::Percentage(55), Constraint::Percentage(45)]).areas(area)
    } else {
        Layout::vertical([Constraint::Fill(3), Constraint::Fill(2)]).areas(area)
    };
    draw_color_map(f, shared, view, map_area);
    draw_flavor(f, shared, flavor_area);
}

fn draw_color_map(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, area: Rect) {
    let palette = &shared.palette;
    let summary = super::play_style_for(shared);
    let ghost = summary.is_none() && !view.play_style_failed;
    let mut block = Block::bordered().border_style(palette.border);
    if ghost {
        block = block.title(Line::styled(" ░░░░░░░░ ", palette.muted).centered());
    } else {
        block = block
            .title(Line::from(" Color map ").bold().centered())
            .title(Line::from(vec![Span::raw(" ? ").bold()]).right_aligned());
    }
    let inner = block.inner(area);
    f.render_widget(block, area);
    let width = inner.width;
    let rows = inner.height.saturating_sub(2);

    let counts = summary
        .filter(|s| s.deck_count > 0)
        .and_then(|s| s.color_map.as_ref())
        .filter(|m| home::has_data(m))
        .map(ColorMapCounts::from);
    let playstyle = Line::from(button(palette, "p", "Playstyle"));
    let lines: Vec<Line<'static>> = match (summary, counts) {
        (_, Some(counts)) => {
            let model = ColorMapModel::new(&counts, super::seed(shared));
            let mut lines = color_map::lines(palette, &model, view.map_elapsed(), width, rows);
            lines.push(Line::default());
            lines.push(color_map::key(palette, &counts, Some(("p", "Playstyle"))));
            lines
        }
        (Some(_), None) => vec![
            Line::styled("Build a deck to see your playstyle", palette.muted),
            Line::default(),
            playstyle,
        ],
        (None, None) if view.play_style_failed => vec![
            Line::styled("Could not load your playstyle", palette.muted),
            Line::default(),
            playstyle,
        ],
        (None, None) => {
            let elapsed = view.opened.elapsed();
            let mut lines = color_map::ghost_lines(palette, &ghost_model(), elapsed, width, rows);
            lines.push(Line::default());
            lines.push(Line::styled(
                "░░░ ░░░ ░░░ ░░░ ░░░ ░░░░░░░░ ░░░░░░░░",
                palette.muted,
            ));
            lines
        }
    };
    f.render_widget(Paragraph::new(lines).centered(), inner);
}

fn draw_flavor(f: &mut Frame<'_>, shared: &Shared, area: Rect) {
    let palette = &shared.palette;
    let card = shared.caches.flavor.as_ref().map(|c| &c.card);
    match card {
        Some(card) if has_flavor(card) => draw_flavor_card(f, palette, card, area),
        Some(_) => {}
        None => {
            let block = Block::bordered().border_style(palette.border);
            let inner = block.inner(area);
            f.render_widget(block, area);
            let width = usize::from(inner.width);
            let mut lines = vec![
                Line::styled("░".repeat(width.saturating_mul(45) / 100), palette.muted).centered(),
            ];
            lines.extend(skeleton::rows(palette, 3, width));
            lines.push(Line::styled("░░░░░░░░ ░░░░ ░░░░ ░░░░", palette.muted).centered());
            f.render_widget(Paragraph::new(lines), inner);
        }
    }
}

fn draw_flavor_card(f: &mut Frame<'_>, palette: &Palette, card: &Card, area: Rect) {
    let data = &card.scryfall_data;
    let quote = format!("{} ", data.flavor_text.as_deref().unwrap_or_default());
    let price = deck_metrics::card_price(data, PriceCurrency::Usd);
    let mut buy_row = vec![Span::raw(data.name.clone()).bold()];
    if let Some(price) = price {
        buy_row.push(Span::styled(
            format!("  {}", PriceCurrency::Usd.format_amount(price)),
            palette.muted,
        ));
    }
    buy_row.push(Span::raw("  "));
    buy_row.extend(button(palette, "i", "Image"));
    if price.is_some() {
        buy_row.push(Span::raw("  "));
        buy_row.extend(button(palette, "b", "Buy"));
    }
    let mut lines: Vec<Line<'static>> = quote
        .lines()
        .map(|l| Line::from(l.to_owned()).italic())
        .collect();
    lines.push(Line::default());
    lines.push(Line::from(buy_row));
    f.render_widget(
        Paragraph::new(lines)
            .centered()
            .wrap(Wrap { trim: false })
            .block(
                Block::bordered()
                    .title(Line::from(" Featured flavor ").bold().centered())
                    .border_style(palette.border),
            ),
        area,
    );
}
