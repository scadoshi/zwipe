//! Remove: the title, the board chips, the card with its info block, the keys, and any sheet over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Padding, Paragraph, Wrap};

use super::{BoardFilter, RemoveView};
use crate::domain::filter;
use crate::domain::stack::Direction;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets;
use crate::inbound::widgets::{action_bar, card_info};

/// The card is at most this wide, in cells.
const CARD_WIDTH: u16 = 64;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &RemoveView) {
    let palette = &shared.palette;
    let area = f.area();
    let [title, boards, body, info, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(5),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(area);
    f.render_widget(
        Line::from("Remove deck cards").style(palette.accent).bold(),
        title,
    );
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
    match view.current() {
        Some(card) => {
            let border = match view.stack.entering() {
                Some(Direction::Right) => palette.add,
                Some(Direction::Left) => palette.error,
                Some(Direction::Up) => palette.warning,
                None => palette.accent,
            };
            let text = Paragraph::new(card_info::proxy_lines(palette, card, view.face))
                .wrap(Wrap { trim: true })
                .block(
                    Block::bordered()
                        .border_style(border)
                        .padding(Padding::horizontal(1)),
                );
            let height =
                u16::try_from(text.line_count(width.saturating_sub(4))).unwrap_or(u16::MAX);
            f.render_widget(
                text,
                body.centered(
                    Constraint::Length(width),
                    Constraint::Length(height.min(body.height)),
                ),
            );
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

    let filter_label = if filter::shows_dot(&view.filter) {
        "filter •"
    } else {
        "filter"
    };
    let mut pairs = vec![
        ("h", "keep"),
        ("l", "remove"),
        ("m", "move"),
        ("u", "undo"),
        ("1-4", "boards"),
        ("f", filter_label),
        ("r", "refresh"),
    ];
    if view.current().is_some() {
        pairs.push(("e", "details"));
    }
    pairs.extend([("?", "hint"), ("Esc", "back")]);
    f.render_widget(action_bar::line(palette, &pairs), keys);

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
