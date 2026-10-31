//! Changelog: the header, the chip rows, the release cards, and the action bar; a skeleton while loading and a note when the fetch failed.

use std::time::Instant;

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Paragraph, Wrap};

use super::{ChangelogView, Chip, FAILED};
use crate::domain::changelog::{self, ChangelogCache, Tag};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::action_bar::{self, Button};
use crate::inbound::widgets::header::{self, Corner};
use crate::inbound::widgets::measure::height_of;
use crate::inbound::widgets::{entrance, skeleton};

/// The cards are at most this wide, in cells.
const WIDTH: u16 = 72;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &ChangelogView) {
    let palette = &shared.palette;
    let [head, _, body, buttons, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(f.area());
    header::draw(f, palette, head, "Changelog", Corner::Empty);
    let body = body.centered_horizontally(Constraint::Length(WIDTH.min(body.width)));
    match &shared.changelog {
        ChangelogCache::Loading => draw_skeleton(f, palette, body),
        ChangelogCache::Failed => {
            f.render_widget(Line::from(FAILED).style(palette.muted), body);
        }
        ChangelogCache::Loaded(log) => {
            let listed = changelog::listed(log);
            let rows = view.rows(&listed);
            let chip_lines: Vec<Line<'static>> = rows
                .iter()
                .enumerate()
                .map(|(r, chips)| chip_line(palette, chips, (r == view.row).then_some(view.chip)))
                .collect();
            let chips_height = height_of(
                Paragraph::new(chip_lines.clone())
                    .wrap(Wrap { trim: false })
                    .line_count(body.width),
            );
            let [chips_area, _, cards_area] = Layout::vertical([
                Constraint::Length(chips_height),
                Constraint::Length(1),
                Constraint::Fill(1),
            ])
            .areas(body);
            f.render_widget(
                Paragraph::new(chip_lines).wrap(Wrap { trim: false }),
                chips_area,
            );
            let cards = card_lines(palette, &changelog::visible(&listed, &view.filter));
            let cards = Paragraph::new(cards).wrap(Wrap { trim: false });
            let total = height_of(cards.line_count(cards_area.width));
            let scroll = view.scroll.min(total.saturating_sub(cards_area.height));
            f.render_widget(cards.scroll((scroll, 0)), cards_area);
            entrance::draw(
                f,
                cards_area,
                Instant::now().saturating_duration_since(view.filtered_at),
            );
        }
    }
    f.render_widget(
        action_bar::buttons(palette, &[Button::new("Back")], None).centered(),
        buttons,
    );
    f.render_widget(
        action_bar::line(
            palette,
            &[
                ("h/l", "chip"),
                ("Tab", "next row"),
                ("Enter", "pick"),
                ("j/k", "scroll"),
                ("!", "help"),
                ("Esc", "back"),
            ],
        )
        .centered(),
        keys,
    );
}

/// A chip row: lit chips reversed, the focused one underlined.
fn chip_line(palette: &Palette, chips: &[Chip], focused: Option<usize>) -> Line<'static> {
    let spans: Vec<Span<'static>> = chips
        .iter()
        .enumerate()
        .flat_map(|(i, chip)| {
            let style = if chip.lit {
                palette.accent.reversed().bold()
            } else {
                palette.muted
            };
            let style = if focused == Some(i) {
                style.underlined()
            } else {
                style
            };
            [
                Span::styled(format!(" {} ", chip.label), style),
                Span::raw(" "),
            ]
        })
        .collect();
    Line::from(spans)
}

/// The cards: version, date and tag, then a bullet per entry, a blank line between cards.
fn card_lines(palette: &Palette, shown: &[changelog::Listed<'_>]) -> Vec<Line<'static>> {
    let mut lines = Vec::new();
    for item in shown {
        let mut head = vec![
            Span::styled(item.release.version.clone(), palette.accent).bold(),
            Span::raw("  "),
            Span::styled(item.release.date.clone(), palette.muted),
        ];
        if let Some(tag) = item.tag {
            let style = match tag {
                Tag::Upcoming => palette.warning,
                Tag::Latest => palette.add,
            };
            head.push(Span::raw("  "));
            head.push(Span::styled(format!(" {} ", tag.label()), style.reversed()));
        }
        lines.push(Line::from(head));
        lines.extend(item.release.entries.iter().map(|entry| {
            Line::from(vec![
                Span::styled("• ", palette.muted),
                Span::raw(entry.clone()),
            ])
        }));
        lines.push(Line::default());
    }
    lines
}

/// The loading skeleton: a row of chip ghosts, then three card ghosts of five, three and four bullets.
fn draw_skeleton(f: &mut Frame<'_>, palette: &Palette, area: Rect) {
    let chip = Span::styled("▒▒▒▒▒", palette.skeleton);
    let mut lines = vec![Line::from(
        std::iter::repeat_n([chip, Span::raw(" ")], 8)
            .flatten()
            .collect::<Vec<_>>(),
    )];
    lines.push(Line::default());
    let width = usize::from(area.width);
    for bullets in [5, 3, 4] {
        lines.push(Line::from(vec![
            Span::styled("▒▒▒▒▒▒", palette.skeleton),
            Span::raw("  "),
            Span::styled("▒▒▒▒▒▒▒▒▒▒", palette.skeleton),
        ]));
        lines.extend(skeleton::rows(palette, bullets, width));
        lines.push(Line::default());
    }
    f.render_widget(Paragraph::new(lines), area);
}
