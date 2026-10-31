//! Remove: the title, the board chips, the card with its info block, the keys, and any sheet over them.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Style;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Widget};

use super::{BoardFilter, RemoveView};
use crate::domain::filter;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets;
use crate::inbound::widgets::card_motion::{self, Phase};
use crate::inbound::widgets::{action_bar, card_info, header};

/// The card is at most this wide, in cells.
const CARD_WIDTH: u16 = 64;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &RemoveView) {
    let palette = &shared.palette;
    let area = f.area();
    let bar = key_line(palette, view);
    let bar_height = action_bar::height(&bar, area.width.saturating_sub(2));
    let [title, boards, body, info, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(5),
        Constraint::Length(bar_height),
    ])
    .margin(1)
    .areas(area);
    header::draw(f, palette, title, "Remove deck cards", header::Corner::Hint);
    let mut chips = vec![Span::styled("Boards ", palette.muted.bold())];
    for board in BoardFilter::ALL {
        let style = if board == view.board {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        };
        chips.push(Span::styled(format!(" {} ", board.label()), style));
        chips.push(Span::raw(" "));
    }
    f.render_widget(Line::from(chips), boards);

    let width = body.width.min(CARD_WIDTH);
    let now = Instant::now();
    let motion = view.motion.as_ref().filter(|m| m.running(now));
    match view.current() {
        Some(card) => {
            // While a card comes back after an undo, the card it covers rests under it.
            let resting = match motion.map(card_motion::CardMotion::phase) {
                Some(Phase::Enter) => view.stack.next_wrapping().map(|c| (c, 0, palette.accent)),
                _ => Some((card, view.face, entering_style(palette, view))),
            };
            if let Some((card, face, border)) = resting {
                let (text, rect) =
                    card_info::proxy_card(palette, card, face, border, body, CARD_WIDTH);
                f.render_widget(text, rect);
            }
            f.render_widget(
                Paragraph::new(card_info::info_lines(palette, card)),
                info.centered_horizontally(Constraint::Length(width)),
            );
        }
        None => f.render_widget(
            Line::from("No cards").style(palette.muted).centered(),
            body.centered(Constraint::Length(width), Constraint::Length(1)),
        ),
    }
    if let Some(motion) = motion {
        draw_motion(f, palette, view, motion, body, now);
    }

    f.render_widget(action_bar::wrap(bar, keys.width), keys);

    if let Some(details) = &view.details {
        details.draw(f, shared, area);
    }
    if let Some(printing) = &view.printing {
        printing.draw(f, palette, area);
    }
    if let Some(sheet) = &view.sheet {
        card_sheets::filter::draw::draw(f, shared, sheet, area);
    }
}

/// The keys: the swipes in their colors, then the boards, the filter with a dot when one is on, and the rest.
fn key_line(palette: &Palette, view: &RemoveView) -> Line<'static> {
    let filter_label = if filter::shows_dot(&view.filter) {
        "filter •"
    } else {
        "filter"
    };
    let mut pairs = action_bar::swipes(
        palette,
        ("l", "remove"),
        ("h", "skip"),
        Some(("m", "maybeboard")),
        ("u", "undo"),
    );
    let mut rest = vec![("1-4", "boards"), ("f", filter_label), ("r", "refresh")];
    if view.current().is_some() {
        rest.push(("e", "details"));
    }
    rest.extend([("?", "hint"), ("Esc", "back")]);
    pairs.extend(rest.into_iter().map(|(k, a)| (k, a, palette.muted)));
    action_bar::styled_line(&pairs)
}

/// The border of the top card: the side it came back from after an undo, else the accent.
fn entering_style(palette: &Palette, view: &RemoveView) -> Style {
    view.stack
        .entering()
        .map_or(palette.accent, |d| card_motion::side_style(palette, d))
}

/// The card in motion over the body: the one that left, from its snapshot, or the top card coming back, clipped at the screen's edge.
fn draw_motion(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &RemoveView,
    motion: &card_motion::CardMotion<card_motion::Snapshot>,
    body: Rect,
    now: Instant,
) {
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
    let (text, rest) = card_info::proxy_card(palette, card, face, border, body, CARD_WIDTH);
    if let Some(placement) = motion.placement(now, rest, f.area()) {
        card_motion::draw(f.buffer_mut(), &placement, palette.base, |buf| {
            text.render(rest, buf);
        });
    }
}
