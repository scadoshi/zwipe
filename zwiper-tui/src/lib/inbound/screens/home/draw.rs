//! Home: the hero (logo, figures and chips), the color map panel and the featured flavor panel, then the action bar.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui_image::StatefulImage;
use ratatui_image::picker::ProtocolType;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_ui::color_map::{ColorMapCounts, ColorMapModel};
use zwipe_ui::playstyle;

use super::{HomeView, Overlay, keys};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::swipe::CardImage;
use crate::inbound::widgets::color_map::Scale;
use crate::inbound::widgets::{
    action_bar, color_map, entrance, figures, header, logo, measure, skeleton, stagger,
};

/// The two lower panels sit side by side from this width, in cells.
const SIDE_BY_SIDE: u16 = 90;

/// Rows kept for the panels under the hero before the logo may take room.
const PANEL_ROOM: u16 = 12;

/// The figures sit in tiles from this terminal height; a shorter one keeps the plain strip so the panels keep their rows.
const TILES_FROM_ROWS: u16 = 22;

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
    let logo_height = logo_text.map_or(0, |l| measure::height_of(l.lines().count()));
    let strip_height = if area.height >= TILES_FROM_ROWS {
        figures::TILE_HEIGHT
    } else {
        figures::HEIGHT
    };
    let hero_height = logo_height
        .saturating_add(u16::from(logo_height > 0))
        .saturating_add(strip_height)
        .saturating_add(1);

    let [title, hero, panels, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(hero_height),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);

    header::draw(f, palette, title, "Home", header::Corner::Empty);
    let since = view.opened.elapsed();
    if let Some(hero) = stagger::area(hero, since, Duration::ZERO).map(|r| r.intersection(f.area()))
    {
        draw_hero(f, shared, view, logo_text, strip_height, hero);
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
    strip_height: u16,
    area: Rect,
) {
    let palette = &shared.palette;
    let logo_height = logo_text.map_or(0, |l| measure::height_of(l.lines().count()));
    let [logo_area, _, strip, chips] = Layout::vertical([
        Constraint::Length(logo_height),
        Constraint::Length(u16::from(logo_height > 0)),
        Constraint::Length(strip_height),
        Constraint::Length(1),
    ])
    .areas(area);
    if let Some(text) = logo_text {
        logo::draw(
            f,
            logo_area,
            text,
            palette.logo,
            view.logo_started.map(|_| view.logo_elapsed()),
        );
    }
    figures::draw_tiles(
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
    draw_flavor(f, shared, view, flavor_area);
}

fn draw_color_map(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, area: Rect) {
    let palette = &shared.palette;
    let summary = super::play_style_for(shared);
    let ghost = summary.is_none() && !view.play_style_failed;
    let mut block = crate::inbound::widgets::panel::bordered().border_style(palette.border);
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

    let counts = summary
        .filter(|s| s.deck_count > 0)
        .and_then(|s| s.color_map.as_ref())
        .map(ColorMapCounts::from)
        .filter(playstyle::color_map_has_data);
    let playstyle = Line::from(button(palette, "p", "Playstyle"));
    let lines: Vec<Line<'static>> = match (summary, counts) {
        (_, Some(counts)) => {
            let model = ColorMapModel::new(&counts, super::seed(shared));
            let key = color_map::key(palette, &counts, Some(("p", "Playstyle")));
            let key_height = measure::height_of(
                Paragraph::new(key.clone())
                    .wrap(Wrap { trim: true })
                    .line_count(width),
            );
            // The rule gives way when the map, even at half scale, needs its row.
            let room = inner.height.saturating_sub(key_height);
            let beside_rule = room.saturating_sub(1);
            let ruled = Scale::fitting(&model, width, beside_rule).rows(&model) <= beside_rule;
            let rows = if ruled { beside_rule } else { room };
            let mut lines = color_map::lines(palette, &model, view.map_elapsed(), width, rows);
            if ruled {
                lines.push(color_map::rule(palette, usize::from(width)));
            }
            lines.push(key);
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
            let mut lines = color_map::ghost_lines(
                palette,
                &zwipe_ui::color_map::ghost_model(),
                elapsed,
                width,
                // The blank row and the ghost key under it.
                inner.height.saturating_sub(2),
            );
            lines.push(Line::default());
            lines.push(Line::styled(
                "░░░ ░░░ ░░░ ░░░ ░░░ ░░░░░░░░ ░░░░░░░░",
                palette.muted,
            ));
            lines
        }
    };
    f.render_widget(
        Paragraph::new(lines).centered().wrap(Wrap { trim: true }),
        inner,
    );
}

fn draw_flavor(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, area: Rect) {
    let palette = &shared.palette;
    let card = shared.caches.flavor.as_ref().map(|c| &c.card);
    match card {
        Some(card) if keys::has_flavor(card) => draw_flavor_card(f, shared, view, card, area),
        Some(_) => {}
        None => {
            let block = crate::inbound::widgets::panel::bordered().border_style(palette.border);
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

/// The featured card's panel: its image beside or above the quote, name, price and keys when the terminal draws real images and the panel has room, else those alone.
fn draw_flavor_card(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, card: &Card, area: Rect) {
    let palette = &shared.palette;
    let block = crate::inbound::widgets::panel::bordered()
        .title(Line::from(" Featured flavor ").bold().centered())
        .border_style(palette.border);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let (quote, buy) = flavor_text(palette, card);
    let ready = shared
        .caches
        .flavor_image
        .as_ref()
        .and_then(|slot| match &slot.image {
            CardImage::Ready(protocol) => Some(protocol),
            CardImage::Loading | CardImage::Failed => None,
        })
        .filter(|_| super::draws_images(shared));
    let layout = match ready {
        Some(_) => flavor_layout(inner, |width| flavor_text_rows(&quote, &buy, width)),
        None => FlavorLayout::Text(inner),
    };
    let text_area = match layout {
        FlavorLayout::Text(text) => text,
        FlavorLayout::Beside { image, text } | FlavorLayout::Above { image, text } => {
            if let Some(protocol) = ready
                && !flavor_image_held(shared, view)
                // A protocol already borrowed by a draw in progress is skipped for this frame.
                && let Ok(mut protocol) = protocol.try_borrow_mut()
            {
                f.render_stateful_widget(StatefulImage::default(), image, &mut *protocol);
            }
            text
        }
    };
    let centered = !matches!(layout, FlavorLayout::Text(_));
    draw_flavor_text(f, quote, buy, text_area, centered);
}

/// The featured card's image takes at least this many rows, enough to tell the card by.
const FLAVOR_IMAGE_MIN_ROWS: u16 = 18;

/// An image's width for each row of height, in tenths of a cell, as Add sizes its images: a card is about 0.72 as wide as tall, and a cell about half as wide as tall.
const FLAVOR_IMAGE_WIDTH_PER_ROW_TENTHS: u16 = 14;

/// Cells between the image and the text beside it.
const FLAVOR_GAP: u16 = 2;

/// The text beside an image keeps at least this many cells.
const FLAVOR_TEXT_MIN_WIDTH: u16 = 28;

/// Where the featured card's parts go inside its panel.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum FlavorLayout {
    /// The quote, name, price and keys alone.
    Text(Rect),
    /// The image on the left, the text beside it, on a wide panel.
    Beside { image: Rect, text: Rect },
    /// The image on top, the text under it, on a tall narrow panel.
    Above { image: Rect, text: Rect },
}

/// The image's width in cells for `rows` rows.
fn flavor_image_width(rows: u16) -> u16 {
    rows.saturating_mul(FLAVOR_IMAGE_WIDTH_PER_ROW_TENTHS) / 10
}

/// How the featured card lays out in `inner` with an image to show: beside the text when the panel is wide enough for both at full height, above it when it is tall enough to leave the text its rows, else the text alone. `text_rows` gives the text's height at a width.
fn flavor_layout(inner: Rect, text_rows: impl Fn(u16) -> u16) -> FlavorLayout {
    if inner.height >= FLAVOR_IMAGE_MIN_ROWS {
        let image_width = flavor_image_width(inner.height);
        let needed = image_width
            .saturating_add(FLAVOR_GAP)
            .saturating_add(FLAVOR_TEXT_MIN_WIDTH);
        if inner.width >= needed {
            let [image, _, text] = Layout::horizontal([
                Constraint::Length(image_width),
                Constraint::Length(FLAVOR_GAP),
                Constraint::Fill(1),
            ])
            .areas(inner);
            return FlavorLayout::Beside { image, text };
        }
    }
    let text_height = text_rows(inner.width);
    let room = inner.height.saturating_sub(text_height).saturating_sub(1);
    let widest = inner.width.saturating_mul(10) / FLAVOR_IMAGE_WIDTH_PER_ROW_TENTHS;
    let rows = room.min(widest);
    if rows < FLAVOR_IMAGE_MIN_ROWS {
        return FlavorLayout::Text(inner);
    }
    let [image, _, text] = Layout::vertical([
        Constraint::Length(rows),
        Constraint::Length(1),
        Constraint::Fill(1),
    ])
    .areas(inner);
    FlavorLayout::Above {
        image: image.centered_horizontally(Constraint::Length(flavor_image_width(rows))),
        text,
    }
}

/// Whether the featured card's image waits this frame: Sixel and iTerm2 draw as one escape that cells drawn over it would break, so the image waits while a popup, a hint or the panels' entrance is over it. Kitty's placeholders are ordinary cells and stay.
fn flavor_image_held(shared: &Shared, view: &HomeView) -> bool {
    let escape = shared.images.as_ref().is_some_and(|p| {
        matches!(
            p.protocol_type(),
            ProtocolType::Sixel | ProtocolType::Iterm2
        )
    });
    let covered = view.overlay.is_some()
        || shared.hint.is_some()
        || stagger::entering(view.opened.elapsed(), stagger::DELAY, entrance::DURATION);
    escape && covered
}

/// The quote and the name, price, Image and Buy row, as paragraphs that wrap.
fn flavor_text(palette: &Palette, card: &Card) -> (Paragraph<'static>, Paragraph<'static>) {
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
    let lines: Vec<Line<'static>> = quote
        .lines()
        .map(|l| Line::from(l.to_owned()).italic())
        .collect();
    let quote = Paragraph::new(lines).centered().wrap(Wrap { trim: false });
    let buy = Paragraph::new(Line::from(buy_row))
        .centered()
        .wrap(Wrap { trim: false });
    (quote, buy)
}

/// The rows the quote, a gap and the buy row take at `width`.
fn flavor_text_rows(quote: &Paragraph<'_>, buy: &Paragraph<'_>, width: u16) -> u16 {
    measure::height_of(quote.line_count(width))
        .saturating_add(1)
        .saturating_add(measure::height_of(buy.line_count(width)))
}

/// Draws the quote over the buy row in `area`, from its top or, beside an image, centered down it. The name and its Image and Buy keys stay in the panel; a quote too long for the rest is cut.
fn draw_flavor_text(
    f: &mut Frame<'_>,
    quote: Paragraph<'static>,
    buy: Paragraph<'static>,
    area: Rect,
    centered: bool,
) {
    let area = if centered {
        let rows = flavor_text_rows(&quote, &buy, area.width);
        area.centered_vertically(Constraint::Length(rows.min(area.height)))
    } else {
        area
    };
    let buy_height = measure::height_of(buy.line_count(area.width)).min(area.height);
    let room = area.height.saturating_sub(buy_height);
    let quote_height = measure::height_of(quote.line_count(area.width)).min(room);
    let gap = u16::from(room > quote_height);
    let [quote_area, _, buy_area] = Layout::vertical([
        Constraint::Length(quote_height),
        Constraint::Length(gap),
        Constraint::Length(buy_height),
    ])
    .areas(area);
    f.render_widget(quote, quote_area);
    f.render_widget(buy, buy_area);
}
