//! The Add screen: the title, the From and Synergy chips, the card with its info block under it, the keys, and any sheet over them.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};
use ratatui_image::StatefulImage;
use ratatui_image::picker::ProtocolType;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use super::{CardImage, Motion, Source, SwipeView};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets::filter;
use crate::inbound::widgets::card_motion::{self, Phase};
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

    let now = Instant::now();
    let motion = view.motion.as_ref().filter(|m| m.running(now));
    match view.current() {
        Some(card) => draw_card(f, shared, view, card, motion, body),
        None => draw_empty(f, palette, view, body),
    }
    if let Some(motion) = motion {
        draw_motion(f, shared, view, motion, body, now);
    }

    f.render_widget(key_line(palette, view), keys);

    if let Some(details) = &view.details {
        details.draw(f, shared, area);
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
    let mut keys = action_bar::swipes(
        palette,
        ("l", "add"),
        ("h", "skip"),
        Some(("m", "maybeboard")),
        ("u", "undo"),
    );
    let mut rest = vec![("f", filter), ("r", "refresh")];
    if view.current().is_some() {
        rest.push(("e", "details"));
    }
    rest.push(("Tab", "from"));
    if view.synergy_shown() {
        rest.push(("y", "synergy"));
    }
    rest.push(("?", "hint"));
    rest.push(("Esc", "back"));
    keys.extend(rest.into_iter().map(|(k, a)| (k, a, palette.muted)));
    action_bar::styled_line(&keys)
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
    entering.map_or(palette.accent, |d| card_motion::side_style(palette, d))
}

/// Splits the body into the card's area and the info block's.
fn card_areas(area: Rect) -> (Rect, Rect) {
    let [card_area, _, info_area] = Layout::vertical([
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(INFO_HEIGHT),
    ])
    .areas(area);
    (card_area, info_area)
}

/// The width an image takes in `card_area`, and whether the text fits beside it.
fn image_layout(card_area: Rect) -> (u16, bool) {
    let image_width = card_area.height.saturating_mul(IMAGE_WIDTH_PER_ROW_TENTHS) / 10;
    let beside = card_area.width >= image_width.saturating_add(CARD_WIDTH).saturating_add(GAP);
    (image_width, beside)
}

/// The image's area and the text's when the text fits beside the image.
fn side_by_side(card_area: Rect, image_width: u16) -> (Rect, Rect) {
    let [image_area, _, text_area] = Layout::horizontal([
        Constraint::Length(image_width),
        Constraint::Length(GAP),
        Constraint::Fill(1),
    ])
    .areas(card_area);
    (image_area, text_area)
}

/// Whether images wait for a motion to settle: Sixel and iTerm2 draw as one escape that later cells skip, so a card passing over them would leave holes. Half blocks and kitty's placeholders are ordinary cells and stay.
fn images_held(shared: &Shared) -> bool {
    shared.images.as_ref().is_some_and(|p| {
        matches!(
            p.protocol_type(),
            ProtocolType::Sixel | ProtocolType::Iterm2
        )
    })
}

/// The top card with its info block under it. While a card comes back after an undo, the card it covers rests here instead.
fn draw_card(
    f: &mut Frame<'_>,
    shared: &Shared,
    view: &SwipeView,
    card: &Card,
    motion: Option<&Motion>,
    area: Rect,
) {
    let palette = &shared.palette;
    let (card_area, info_area) = card_areas(area);
    let info_width = card_area.width.min(CARD_WIDTH);
    f.render_widget(
        Paragraph::new(card_info::info_lines(palette, card)),
        info_area.centered_horizontally(Constraint::Length(info_width)),
    );
    let images = if motion.is_some() && images_held(shared) {
        Images::Held
    } else {
        Images::Shown
    };
    let resting = match motion.map(Motion::phase) {
        Some(Phase::Enter) => view.under().map(|under| Resting {
            card: under,
            face: 0,
            border: palette.accent,
        }),
        _ => Some(Resting {
            card,
            face: view.face,
            border: entering_style(palette, view),
        }),
    };
    if let Some(resting) = resting {
        draw_face(f, shared, view, resting, images, card_area);
    }
}

/// A card to draw at rest: which face shows and its border.
#[derive(Debug, Clone, Copy)]
struct Resting<'card> {
    card: &'card Card,
    face: usize,
    border: Style,
}

/// Whether card images draw this frame.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Images {
    Shown,
    /// Left out until a motion settles; see [`images_held`].
    Held,
}

/// One card's face at rest: its image beside its text when there is room, else the one `i` picks. With images held, the image is left out and a card shown as its image shows as text.
fn draw_face(
    f: &mut Frame<'_>,
    shared: &Shared,
    view: &SwipeView,
    resting: Resting<'_>,
    images: Images,
    card_area: Rect,
) {
    let palette = &shared.palette;
    let Resting { card, face, border } = resting;
    let held = images == Images::Held;
    if shared.images.is_none() {
        draw_text(f, palette, card, face, border, card_area);
        return;
    }
    let image = card
        .scryfall_data
        .face_image_url(
            face.checked_rem(card_info::face_count(card)).unwrap_or(0),
            ImageSize::Normal,
        )
        .and_then(|url| view.images.get(url));
    let (image_width, beside) = image_layout(card_area);
    if beside {
        let (image_area, text_area) = side_by_side(card_area, image_width);
        if !held {
            draw_image(f, palette, image, image_area);
        }
        draw_text(f, palette, card, face, border, text_area);
    } else if view.show_image && !held {
        draw_image(
            f,
            palette,
            image,
            card_area.centered_horizontally(Constraint::Length(image_width)),
        );
    } else {
        draw_text(f, palette, card, face, border, card_area);
    }
}

/// The card in motion over the body: the one that left, from its snapshot, or the top card coming back. It moves as its text card, clipped at the screen's edge.
fn draw_motion(
    f: &mut Frame<'_>,
    shared: &Shared,
    view: &SwipeView,
    motion: &Motion,
    area: Rect,
    now: Instant,
) {
    let palette = &shared.palette;
    let (card, face, border) = match motion.phase() {
        Phase::Exit(snapshot) => (
            &snapshot.card,
            snapshot.face,
            card_motion::side_style(palette, motion.direction()),
        ),
        Phase::Enter => {
            let Some(card) = view.current() else {
                return;
            };
            (card, view.face, entering_style(palette, view))
        }
    };
    let (card_area, _) = card_areas(area);
    let (image_width, beside) = image_layout(card_area);
    let text_area = if shared.images.is_some() && beside {
        side_by_side(card_area, image_width).1
    } else {
        card_area
    };
    let (text, rest) = card_info::proxy_card(palette, card, face, border, text_area, CARD_WIDTH);
    let bounds = f.area();
    if let Some(placement) = motion.placement(now, rest, bounds) {
        card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| {
            text.render(rest, buf);
        });
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
    let (text, rect) = card_info::proxy_card(palette, card, face, border, area, CARD_WIDTH);
    f.render_widget(text, rect);
}
