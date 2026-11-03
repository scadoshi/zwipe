//! Home: the hero (logo, figures and chips), the color map panel with the featured flavor panel under it, each only as tall as what it holds, then the action bar.

use std::time::Duration;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Margin, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};
use ratatui_image::StatefulImage;
use ratatui_image::picker::ProtocolType;
use zwipe_ui::color_map::ColorMapModel;
use zwipe_ui::screens::home::{
    ACTIVITY, BUY, COLOR_MAP, FEATURED_FLAVOR, Flavor, IMAGE, LIVE, NO_PLAY_STYLE,
    PLAY_STYLE_FAILED, PLAYSTYLE, PlayStyle, Quote,
};

use super::{HomeView, Overlay};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::swipe::CardImage;
use crate::inbound::widgets;
use crate::inbound::widgets::color_map::Scale;
use crate::inbound::widgets::measure::BORDERS;
use crate::inbound::widgets::{
    action_bar, color_map, entrance, figures, logo, measure, skeleton, stagger,
};

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
    let strip_height = if area.height >= TILES_FROM_ROWS {
        figures::TILE_HEIGHT
    } else {
        figures::HEIGHT
    };
    let [content, status, keys] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(area);
    let map = map_heights(palette, view, content.width);
    let flavor = flavor_heights(shared, view, content.width);
    let (logo_text, top, hero_height, rows) =
        place_hero(area, content.height, strip_height, map, flavor);
    let [_, hero, map_area, flavor_area, _] = Layout::vertical([
        Constraint::Length(top),
        Constraint::Length(hero_height),
        Constraint::Length(rows.map),
        Constraint::Length(rows.flavor),
        Constraint::Fill(1),
    ])
    .areas(content);

    let since = view.opened.elapsed();
    if let Some(hero) = stagger::area(hero, since, Duration::ZERO).map(|r| r.intersection(f.area()))
    {
        draw_hero(f, shared, view, logo_text, strip_height, hero);
        stagger::fade(f, hero, since, Duration::ZERO, entrance::DURATION);
    }
    let rising =
        |panel: Rect| stagger::area(panel, since, stagger::DELAY).map(|r| r.intersection(content));
    if let (Some(map_area), Some(flavor_area)) = (rising(map_area), rising(flavor_area)) {
        draw_color_map(f, shared, view, map_area);
        draw_flavor(f, shared, view, flavor_area);
        let panels = map_area.union(flavor_area);
        stagger::fade(f, panels, since, stagger::DELAY, entrance::DURATION);
    }
    f.render_widget(Line::styled(shared.status.clone(), palette.muted), status);
    let bar = action_bar::line(
        palette,
        &[
            ("d", "Decks"),
            ("a", "Account"),
            ("c", "Config"),
            ("q", "quit"),
        ],
    );
    f.render_widget(
        action_bar::with_help(palette, bar, action_bar::Hint::Absent),
        keys,
    );
    match &view.overlay {
        Some(Overlay::Info(info)) => info.draw(f, palette),
        Some(Overlay::Buy(dialog)) => dialog.draw(f, palette),
        None => {}
    }
}

/// The color map panel's rows at full scale and at half scale, each with its rule and key.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct MapHeights {
    full: u16,
    half: u16,
}

/// The featured flavor panel's rows with its text alone, and with the card's image when the terminal draws one and the panel's width has room for it.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct FlavorHeights {
    text: u16,
    image: Option<u16>,
}

/// The rows the two panels take, the color map over the featured flavor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
struct PanelRows {
    map: u16,
    flavor: u16,
}

impl PanelRows {
    /// Each panel's rows in `room` rows: the first of these that fits, each panel only as tall as what it holds, is the full map over the image, the half-scale map over the image, the full map over the text, the half-scale map over the text. When none fits, the map keeps at least three fifths of the room, as much as the text leaves it, and the flavor panel the rest.
    fn fit(map: MapHeights, flavor: FlavorHeights, room: u16) -> Self {
        let mut tries = Vec::with_capacity(4);
        if let Some(image) = flavor.image {
            tries.extend([(map.full, image), (map.half, image)]);
        }
        tries.extend([(map.full, flavor.text), (map.half, flavor.text)]);
        if let Some((map, flavor)) = tries
            .into_iter()
            .find(|(map, flavor)| map.saturating_add(*flavor) <= room)
        {
            return Self { map, flavor };
        }
        let at_least = room.saturating_mul(3) / 5;
        let map_rows = map.full.min(room.saturating_sub(flavor.text).max(at_least));
        Self {
            map: map_rows,
            flavor: room.saturating_sub(map_rows).min(flavor.text),
        }
    }
}

/// The color map panel's rows at `width` cells wide.
fn map_heights(palette: &Palette, view: &HomeView, width: u16) -> MapHeights {
    let inner = width.saturating_sub(BORDERS);
    let framed = |map_rows: u16, under: u16| map_rows.saturating_add(under).saturating_add(BORDERS);
    let scaled = |model: &ColorMapModel, under: u16| {
        let full = if model.columns.saturating_mul(2) <= i32::from(inner) {
            Scale::Full
        } else {
            Scale::Half
        };
        MapHeights {
            full: framed(full.rows(model), under),
            half: framed(Scale::Half.rows(model), under),
        }
    };
    match view.model.play_style() {
        PlayStyle::Map(counts) => {
            let model = ColorMapModel::new(&counts, view.model.seed());
            let key = color_map::key(palette, &counts, Some(("p", PLAYSTYLE)));
            // The rule, then the key.
            scaled(&model, 1_u16.saturating_add(key_height(key, inner)))
        }
        // The blank row and the ghost key.
        PlayStyle::Loading => scaled(&zwipe_ui::color_map::ghost_model(), 2),
        PlayStyle::Empty | PlayStyle::Failed => {
            let notice = Paragraph::new(notice_lines(palette, &view.model.play_style()))
                .wrap(Wrap { trim: true });
            let rows = framed(measure::height_of(notice.line_count(inner)), 0);
            MapHeights {
                full: rows,
                half: rows,
            }
        }
    }
}

/// The rows `key` takes wrapped at `width`.
fn key_height(key: Line<'static>, width: u16) -> u16 {
    measure::height_of(
        Paragraph::new(key)
            .wrap(Wrap { trim: true })
            .line_count(width),
    )
}

/// What the color map panel says when it has no map: no decks yet or a failed load, then the Playstyle key.
fn notice_lines(palette: &Palette, panel: &PlayStyle) -> Vec<Line<'static>> {
    let notice = match panel {
        PlayStyle::Failed => PLAY_STYLE_FAILED,
        PlayStyle::Empty | PlayStyle::Loading | PlayStyle::Map(_) => NO_PLAY_STYLE,
    };
    vec![
        Line::styled(notice, palette.muted),
        Line::default(),
        Line::from(button(palette, "p", PLAYSTYLE)),
    ]
}

/// The featured flavor panel's rows at `width` cells wide.
fn flavor_heights(shared: &Shared, view: &HomeView, width: u16) -> FlavorHeights {
    let inner = width.saturating_sub(BORDERS);
    match view.model.flavor() {
        Flavor::Quote(quote) => {
            let (quote, buy) = flavor_text(&shared.palette, &quote);
            let text_rows = |width: u16| flavor_text_rows(&quote, &buy, width);
            let image = flavor_image_ready(shared)
                .then(|| flavor_image_rows(inner, text_rows))
                .flatten();
            FlavorHeights {
                text: text_rows(inner).saturating_add(BORDERS),
                image: image.map(|rows| rows.saturating_add(BORDERS)),
            }
        }
        Flavor::Blank => FlavorHeights {
            text: 0,
            image: None,
        },
        Flavor::Loading => FlavorHeights {
            text: FLAVOR_GHOST_ROWS.saturating_add(BORDERS),
            image: None,
        },
    }
}

/// The loading ghost's rows: a title bar, three skeleton rows and the ghost card line.
const FLAVOR_GHOST_ROWS: u16 = 5;

/// The inside rows the featured card takes with its image at `inner` cells wide: beside the text at the image's least height when the width has room for both, else above the text, or `None` when the image does not fit at all. `text_rows` gives the text's height at a width.
fn flavor_image_rows(inner: u16, text_rows: impl Fn(u16) -> u16) -> Option<u16> {
    let image_width = flavor_image_width(FLAVOR_IMAGE_MIN_ROWS);
    let beside = inner
        .checked_sub(image_width.saturating_add(FLAVOR_GAP))
        .filter(|text_width| *text_width >= FLAVOR_TEXT_MIN_WIDTH)
        .map(|text_width| FLAVOR_IMAGE_MIN_ROWS.max(text_rows(text_width)))
        .filter(|rows| {
            inner
                >= flavor_image_width(*rows)
                    .saturating_add(FLAVOR_GAP)
                    .saturating_add(FLAVOR_TEXT_MIN_WIDTH)
        });
    if beside.is_some() {
        return beside;
    }
    let widest = inner.saturating_mul(10) / FLAVOR_IMAGE_WIDTH_PER_ROW_TENTHS;
    (widest >= FLAVOR_IMAGE_MIN_ROWS).then(|| {
        FLAVOR_IMAGE_MIN_ROWS
            .saturating_add(1)
            .saturating_add(text_rows(inner))
    })
}

/// Whether the featured card's image is loaded and the terminal draws real images.
fn flavor_image_ready(shared: &Shared) -> bool {
    super::draws_images(shared)
        && shared
            .caches
            .flavor_image
            .as_ref()
            .is_some_and(|slot| matches!(slot.image, CardImage::Ready(_)))
}

/// The hero's rows above the panels: `top` free row(s), the logo, a gap, the figures and the chips row.
fn hero_rows(logo_text: Option<&str>, strip_height: u16) -> u16 {
    let logo_height = logo_text.map_or(0, |l| measure::height_of(l.lines().count()));
    logo_height
        .saturating_add(u16::from(logo_height > 0))
        .saturating_add(strip_height)
        .saturating_add(1)
}

/// Picks the logo and lays the hero over the panels. A drawn logo always has a free row above it, so it never sits on the screen's top edge. The flavor text always shows whole; within that, the first of these that fits wins: the full logo over a full-scale map, the compact Z over a full-scale map, the full logo over a half-scale map, the Z over a half-scale map. When none fits, no logo. Returns the logo, the free rows on top, the hero's height and the panels' rows.
fn place_hero(
    area: Rect,
    content_height: u16,
    strip_height: u16,
    map: MapHeights,
    flavor: FlavorHeights,
) -> (Option<&'static str>, u16, u16, PanelRows) {
    let width = area.width.saturating_sub(2);
    let arts = [zwipe_core::domain::logo::ZWIPE, zwipe_core::domain::logo::Z];
    for map_rows in [map.full, map.half] {
        for art in arts {
            if logo::size(art).0 > width {
                continue;
            }
            let hero = hero_rows(Some(art), strip_height);
            let room = content_height.saturating_sub(hero).saturating_sub(1);
            let rows = PanelRows::fit(map, flavor, room);
            if rows.map >= map_rows && rows.flavor >= flavor.text {
                return (Some(art), 1, hero, rows);
            }
        }
    }
    let hero = hero_rows(None, strip_height);
    let rows = PanelRows::fit(map, flavor, content_height.saturating_sub(hero));
    (None, 0, hero, rows)
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
        view.figures(),
        super::since_open(view),
        view.since_arrival(),
    );
    let settling = view
        .scope_at
        .is_some_and(|t| stagger::entering(t.elapsed(), Duration::ZERO, stagger::SCOPE_IN));
    let scope_in = |style: Style| if settling { style.dim() } else { style };
    let mut spans = Vec::new();
    if view.model.live() {
        spans.push(chip(LIVE, scope_in(palette.add)));
        spans.push(Span::raw(" "));
    }
    spans.push(Span::raw("s ").bold());
    spans.push(chip(view.model.scope().label(), scope_in(palette.accent)));
    spans.push(Span::raw("  "));
    spans.extend(button(palette, "v", ACTIVITY));
    f.render_widget(Line::from(spans).centered(), chips);
}

fn draw_color_map(f: &mut Frame<'_>, shared: &Shared, view: &HomeView, area: Rect) {
    let palette = &shared.palette;
    let panel = view.model.play_style();
    let ghost = panel == PlayStyle::Loading;
    let mut block = crate::inbound::widgets::panel::bordered(palette).border_style(palette.border);
    if ghost {
        block = block.title(widgets::panel::title(palette, "░░░░░░░░").centered());
    } else {
        block = block
            .title(widgets::panel::title(palette, COLOR_MAP).centered())
            .title(Line::from(widgets::button::about_corner(palette)).right_aligned());
    }
    let inner = block.inner(area);
    f.render_widget(block, area);
    let width = inner.width;

    let lines: Vec<Line<'static>> = match panel {
        PlayStyle::Map(counts) => {
            let model = ColorMapModel::new(&counts, view.model.seed());
            let key = color_map::key(palette, &counts, Some(("p", PLAYSTYLE)));
            let key_height = key_height(key.clone(), width);
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
        PlayStyle::Empty | PlayStyle::Failed => notice_lines(palette, &panel),
        PlayStyle::Loading => {
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
    match view.model.flavor() {
        Flavor::Quote(quote) => draw_flavor_card(f, shared, view, &quote, area),
        Flavor::Blank => {}
        Flavor::Loading => {
            let block =
                crate::inbound::widgets::panel::bordered(palette).border_style(palette.border);
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
fn draw_flavor_card(
    f: &mut Frame<'_>,
    shared: &Shared,
    view: &HomeView,
    quote: &Quote,
    area: Rect,
) {
    let palette = &shared.palette;
    let block = crate::inbound::widgets::panel::bordered(palette)
        .title(widgets::panel::title(palette, FEATURED_FLAVOR).centered())
        .border_style(palette.border);
    let inner = block.inner(area);
    f.render_widget(block, area);
    let (quote, buy) = flavor_text(palette, quote);
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
fn flavor_text(palette: &Palette, shown: &Quote) -> (Paragraph<'static>, Paragraph<'static>) {
    let mut buy_row = vec![Span::raw(shown.name.clone()).bold()];
    if let Some(price) = &shown.price {
        buy_row.push(Span::styled(format!("  {price}"), palette.muted));
    }
    buy_row.push(Span::raw("  "));
    buy_row.extend(button(palette, "i", IMAGE));
    if shown.price.is_some() {
        buy_row.push(Span::raw("  "));
        buy_row.extend(button(palette, "b", BUY));
    }
    // Trimmed so every row centers on its words alone, as the card line does.
    let lines: Vec<Line<'static>> = shown
        .text
        .lines()
        .map(|l| Line::from(l.trim().to_owned()).italic())
        .collect();
    let quote = Paragraph::new(lines).centered().wrap(Wrap { trim: true });
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
