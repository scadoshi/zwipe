//! The Add screen: the title, the From and Synergy chips, the card with its info block under it, the keys, and any sheet over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};
use ratatui_image::StatefulImage;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::{CardImage, Source, SwipeView};
use crate::domain::stack::Direction;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets::filter;
use crate::inbound::widgets::{action_bar, card_info, skeleton};

/// The card is at most this wide, in cells.
const CARD_WIDTH: u16 = 64;

/// An image's width for each row of height, in tenths of a cell: a card is about 0.72 as wide as tall, and a cell about half as wide as tall.
const IMAGE_WIDTH_PER_ROW_TENTHS: u16 = 14;

/// Cells between the image and the text.
const GAP: u16 = 2;

/// The info block's height.
const INFO_HEIGHT: u16 = 5;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &SwipeView) {
    let palette = &shared.palette;
    let area = f.area();
    let [header, sources, body, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(area);

    f.render_widget(
        Line::from(vec![
            Span::styled("Add deck cards", palette.accent).bold(),
            Span::styled(format!("  {}", view.deck_name), palette.muted),
        ]),
        header,
    );
    f.render_widget(source_line(palette, view), sources);

    match view.current() {
        Some(card) => draw_card(f, shared, view, card, body),
        None => draw_empty(f, palette, view, body),
    }

    f.render_widget(key_line(palette, view), keys);

    if let Some(details) = &view.details {
        details.draw(f, palette, area);
    }
    if let Some(printing) = &view.printing {
        printing.draw(f, palette, area);
    }
    if let Some(sheet) = &view.sheet {
        filter::draw::draw(f, shared, sheet, area);
    }
}

/// "From" with the Search and Maybeboard chips, and Synergy when it applies.
fn source_line(palette: &Palette, view: &SwipeView) -> Line<'static> {
    let chip = |label: &str, on: bool| -> Span<'static> {
        let style = if on {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        Span::styled(format!(" {label} "), style)
    };
    let mut spans = vec![
        Span::styled("From ", palette.muted.bold()),
        chip("Search", view.source == Source::Search),
        Span::raw(" "),
        chip("Maybeboard", view.source == Source::Maybeboard),
    ];
    if view.synergy_shown() {
        spans.push(Span::raw("   "));
        spans.push(chip("Synergy", view.filter.synergy()));
    }
    Line::from(spans)
}

/// The keys, with a dot on Filter when a user filter is on.
fn key_line(palette: &Palette, view: &SwipeView) -> Line<'static> {
    let filter = if view.filter_dot() {
        "filter •"
    } else {
        "filter"
    };
    let mut keys = vec![
        ("h", "skip"),
        ("l", "add"),
        ("m", "maybe"),
        ("u", "undo"),
        ("f", filter),
        ("r", "refresh"),
    ];
    if view.current().is_some() {
        keys.push(("e", "details"));
    }
    keys.push(("Tab", "from"));
    if view.synergy_shown() {
        keys.push(("y", "synergy"));
    }
    keys.push(("?", "hint"));
    keys.push(("Esc", "back"));
    action_bar::line(palette, &keys)
}

/// No card on top: loading, the end of results, or nothing.
fn draw_empty(f: &mut Frame<'_>, palette: &Palette, view: &SwipeView, area: Rect) {
    let width = area.width.min(CARD_WIDTH);
    let loading = match view.source {
        Source::Search => {
            (view.stack.is_empty() && view.loading_cards)
                || (!view.stack.is_empty() && view.loading_more)
        }
        Source::Maybeboard => false,
    };
    let lines = if loading {
        skeleton::rows(palette, 5, usize::from(width))
    } else {
        let mut lines = vec![Line::from("No cards").style(palette.muted).centered()];
        if view.source == Source::Search && !view.stack.is_empty() {
            lines.push(
                Line::from("u steps back into the pile")
                    .style(palette.muted)
                    .centered(),
            );
        }
        lines
    };
    let height = u16::try_from(lines.len())
        .unwrap_or(u16::MAX)
        .min(area.height);
    f.render_widget(
        Paragraph::new(lines),
        area.centered(Constraint::Length(width), Constraint::Length(height)),
    );
}

/// The border color of a card coming back after an undo: the side it left by.
fn entering_style(palette: &Palette, view: &SwipeView) -> Style {
    let entering = match view.source {
        Source::Search => view.stack.entering(),
        Source::Maybeboard => view.mb_stack.entering(),
    };
    match entering {
        Some(Direction::Right) => palette.add,
        Some(Direction::Left) => palette.error,
        Some(Direction::Up) => palette.warning,
        Some(Direction::Down) | None => palette.accent,
    }
}

/// The card: its image beside its text when there is room, else the one `i` picks, with the info block under.
fn draw_card(f: &mut Frame<'_>, shared: &Shared, view: &SwipeView, card: &Card, area: Rect) {
    let palette = &shared.palette;
    let [card_area, _, info_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(INFO_HEIGHT),
    ])
    .areas(area);
    let info_width = card_area.width.min(CARD_WIDTH);
    f.render_widget(
        Paragraph::new(card_info::info_lines(palette, card)),
        info_area.centered_horizontally(Constraint::Length(info_width)),
    );

    let border = entering_style(palette, view);
    if shared.images.is_none() {
        draw_text(f, palette, card, view.face, border, card_area);
        return;
    }
    let image = card
        .scryfall_data
        .face_image_url(view.face % card_info::face_count(card), ImageSize::Normal)
        .and_then(|url| view.images.get(url));
    let image_width = card_area.height.saturating_mul(IMAGE_WIDTH_PER_ROW_TENTHS) / 10;
    if card_area.width >= image_width.saturating_add(CARD_WIDTH).saturating_add(GAP) {
        let [image_area, _, text_area] = Layout::horizontal([
            Constraint::Length(image_width),
            Constraint::Length(GAP),
            Constraint::Fill(1),
        ])
        .areas(card_area);
        draw_image(f, palette, image, image_area);
        draw_text(f, palette, card, view.face, border, text_area);
    } else if view.show_image {
        draw_image(
            f,
            palette,
            image,
            card_area.centered_horizontally(Constraint::Length(image_width)),
        );
    } else {
        draw_text(f, palette, card, view.face, border, card_area);
    }
}

/// A card's image fitted into `area`, or what stands in for it while it loads or when there is none.
fn draw_image(f: &mut Frame<'_>, palette: &Palette, image: Option<&CardImage>, area: Rect) {
    let message = match image {
        Some(CardImage::Ready(protocol)) => {
            // A protocol already borrowed by a draw in progress is skipped for this frame.
            if let Ok(mut protocol) = protocol.try_borrow_mut() {
                f.render_stateful_widget(StatefulImage::default(), area, &mut *protocol);
            }
            return;
        }
        Some(CardImage::Loading) => "Loading image...",
        Some(CardImage::Failed) | None => "No image",
    };
    f.render_widget(
        Paragraph::new(Line::from(message).style(palette.muted)).centered(),
        area.centered_vertically(Constraint::Length(1)),
    );
}

/// The card as text, one face at a time, centered in `area`.
fn draw_text(
    f: &mut Frame<'_>,
    palette: &Palette,
    card: &Card,
    face: usize,
    border: Style,
    area: Rect,
) {
    let text = Paragraph::new(card_info::proxy_lines(palette, card, face))
        .wrap(Wrap { trim: true })
        .block(
            Block::bordered()
                .border_style(border)
                .padding(Padding::horizontal(1)),
        );
    let width = area.width.min(CARD_WIDTH);
    let height = u16::try_from(text.line_count(width.saturating_sub(4))).unwrap_or(u16::MAX);
    f.render_widget(
        text,
        area.centered(
            Constraint::Length(width),
            Constraint::Length(height.min(area.height)),
        ),
    );
}
