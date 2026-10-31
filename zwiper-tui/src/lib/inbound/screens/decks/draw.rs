//! The deck list: the Group by and Show rows, the tag row, then the decks in their groups.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::Stylize;
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Wrap};
use zwipe_core::domain::deck::DeckProfile;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;

use super::{DeckList, Focus, Row, ShowChip};
use crate::domain;
use crate::domain::deck::list::{DeckGroup, GroupBy};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::pill::{self, PillState, pill};
use crate::inbound::widgets::{action_bar, header, mana, measure, skeleton};

/// Ghost rows while the decks load.
const SKELETON_ROWS: usize = 8;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, list: &DeckList) {
    let palette = &shared.palette;
    let chips = if list.chips_shown() {
        let width = usize::from(f.area().width.saturating_sub(2));
        u16::try_from(chip_rows(&shared.palette, list, width).len()).unwrap_or(u16::MAX)
    } else if list.loading {
        2
    } else {
        0
    };
    let bar = action_bar::line(
        palette,
        &[
            ("Tab", "focus"),
            ("Enter", "open"),
            ("c", "Create"),
            ("m", "More"),
            ("Esc", "Back"),
            ("q", "quit"),
        ],
    );
    let bar_height = action_bar::height(&bar, f.area().width.saturating_sub(2));
    let [header, chips_area, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(chips),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(bar_height),
    ])
    .margin(1)
    .areas(f.area());

    header::draw(f, palette, header, "Decks", header::Corner::Hint);
    if list.chips_shown() {
        f.render_widget(
            Paragraph::new(chip_rows(palette, list, usize::from(chips_area.width))),
            chips_area,
        );
    } else if list.loading {
        let width = usize::from(chips_area.width) / 2;
        f.render_widget(
            Paragraph::new(skeleton::rows(palette, 2, width)),
            chips_area,
        );
    }
    draw_list(f, palette, list, body);

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    f.render_widget(action_bar::wrap(bar, keys.width), keys);
    if let Some(menu) = &list.more {
        menu.draw(f, palette, f.area());
    }
}

/// The Group by row, the Show row, and the tag row while it is open.
fn chip_rows(palette: &Palette, list: &DeckList, width: usize) -> Vec<Line<'static>> {
    let state = |picked: bool, focus: Focus, index: usize, at: usize| PillState {
        picked,
        focused: list.focus == focus && index == at,
    };
    let mut group_by = vec![pill::label(palette, "Group by")];
    group_by.extend(GroupBy::ALL.iter().enumerate().map(|(i, by)| {
        pill(
            by.label(),
            palette.muted,
            state(*by == list.group_by, Focus::GroupBy, i, list.group_focus),
        )
    }));

    let mut show = vec![pill::label(palette, "Show")];
    show.extend(
        list.show_chips()
            .into_iter()
            .enumerate()
            .map(|(i, chip)| match chip {
                ShowChip::All => pill(
                    "All",
                    palette.muted,
                    state(list.show.is_all(), Focus::Show, i, list.show_focus),
                ),
                ShowChip::Color(color) => {
                    let place = zwipe_core::domain::card::scryfall_data::colors::Color::all()
                        .iter()
                        .position(|c| *c == color)
                        .unwrap_or(0);
                    pill(
                        &color.to_short_name(),
                        mana::style(palette, place),
                        state(
                            list.show.colors.contains(&color),
                            Focus::Show,
                            i,
                            list.show_focus,
                        ),
                    )
                }
                ShowChip::Tags => {
                    let arrow = if list.tags_open { "▾" } else { "▸" };
                    pill(
                        &format!("Tags {arrow}"),
                        palette.muted,
                        state(false, Focus::Show, i, list.show_focus),
                    )
                }
                ShowChip::Picked { label, .. } => pill(
                    &label,
                    palette.muted,
                    state(true, Focus::Show, i, list.show_focus),
                ),
            }),
    );

    let mut lines = vec![pill::row(group_by), pill::row(show)];
    if list.tags_open {
        let tags: Vec<Span<'static>> = list
            .tags()
            .into_iter()
            .enumerate()
            .map(|(i, (key, label))| {
                pill(
                    &label,
                    palette.muted,
                    state(
                        list.show.tags.contains(&key),
                        Focus::Tags,
                        i,
                        list.tag_focus,
                    ),
                )
            })
            .collect();
        if !tags.is_empty() {
            lines.extend(pill::wrap("     ", tags, width));
        }
    }
    lines
}

fn draw_list(f: &mut Frame<'_>, palette: &Palette, list: &DeckList, area: Rect) {
    let block = Block::bordered().border_style(palette.border);
    let message = if list.loading {
        let width = usize::from(area.width.saturating_sub(4));
        f.render_widget(
            Paragraph::new(skeleton::rows(palette, SKELETON_ROWS, width)).block(block),
            area,
        );
        return;
    } else if list.error.is_some() {
        Some("Could not load decks")
    } else if list.decks.is_empty() {
        Some("No decks")
    } else if list.groups().is_empty() {
        Some("No decks match")
    } else {
        None
    };
    if let Some(message) = message {
        f.render_widget(
            Paragraph::new(Line::from(message).style(palette.muted)).block(block),
            area,
        );
        return;
    }

    let focused = list.focus == Focus::List;
    let width = usize::from(area.width.saturating_sub(2));
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut selected_end = 0;
    for (i, row) in list.rows().iter().enumerate() {
        let here = focused && i == list.selected;
        let pointer = Span::raw(if here { "> " } else { "  " });
        match row {
            Row::Header { group, collapsed } => {
                lines.push(header_line(palette, group, *collapsed, pointer, here));
            }
            Row::Deck(profile) => {
                let (mut first, chips) = deck_lines(palette, profile, here);
                // A name too long for the row ends in an ellipsis, keeping its pips and count on the row.
                let overflow = first.width().saturating_sub(width.saturating_sub(4));
                if overflow > 0
                    && let Some(name) = first.spans.first_mut()
                {
                    let keep = name.width().saturating_sub(overflow);
                    name.content = measure::ellipsize(&name.content, keep).into();
                }
                let mut spans = vec![pointer, Span::raw("  ")];
                spans.extend(first.spans);
                lines.push(Line::from(spans));
                lines.extend(pill::wrap("      ", chips, width));
            }
        }
        if i == list.selected {
            selected_end = lines.len();
        }
    }
    let rows = usize::from(area.height.saturating_sub(2));
    let scroll = u16::try_from(selected_end.saturating_sub(rows)).unwrap_or(u16::MAX);
    f.render_widget(
        Paragraph::new(lines)
            .block(block)
            .wrap(Wrap { trim: false })
            .scroll((scroll, 0)),
        area,
    );
}

/// A group header: the arrow, then the pips or the header text, and the count.
fn header_line(
    palette: &Palette,
    group: &DeckGroup,
    collapsed: bool,
    pointer: Span<'static>,
    here: bool,
) -> Line<'static> {
    let arrow = if collapsed { "▸ " } else { "▾ " };
    let mut spans = vec![pointer, Span::styled(arrow, palette.muted)];
    let count = group.decks.len();
    match &group.pips {
        Some(pips) if !pips.is_empty() => {
            spans.extend(pips.iter().map(|color| {
                let place = zwipe_core::domain::card::scryfall_data::colors::Color::all()
                    .iter()
                    .position(|c| c == color)
                    .unwrap_or(0);
                mana::pip(palette, place)
            }));
            spans.push(Span::styled(format!(" ({count})"), palette.muted));
        }
        _ => spans.push(Span::styled(
            format!("{} ({count})", group.header),
            palette.accent,
        )),
    }
    let line = Line::from(spans);
    if here { line.bold() } else { line }
}

/// A deck's name line, as the app's row lays it out (the name, pips and count), and the chips under it: the format, power level, command zone and tags.
fn deck_lines(
    palette: &Palette,
    deck: &DeckProfile,
    here: bool,
) -> (Line<'static>, Vec<Span<'static>>) {
    let name_style = if here {
        palette.accent.bold()
    } else {
        palette.base.bold()
    };
    let mut first = vec![Span::styled(deck.name.to_string(), name_style)];
    let pips = domain::deck::pips(deck);
    if !pips.is_empty() {
        first.push(Span::raw("  "));
        first.extend(pips.into_iter().map(|pip| mana::pip(palette, pip)));
    }
    let count = domain::deck::card_count(deck);
    let count_style = if domain::deck::count_fits(deck, count) {
        palette.muted
    } else {
        palette.warning
    };
    first.push(Span::raw("  "));
    first.push(pill(
        &domain::deck::cards_label(count),
        count_style,
        PillState::default(),
    ));

    let mut second: Vec<Span<'static>> = Vec::new();
    if let Some(format) = deck.format {
        second.push(pill(
            format.display_name(),
            palette.accent,
            PillState::default(),
        ));
    }
    if let Some(power) = deck.power_level {
        second.push(pill(
            power.display_name(),
            palette.warning,
            PillState::default(),
        ));
    }
    for name in [
        &deck.commander_name,
        &deck.partner_commander_name,
        &deck.background_name,
        &deck.signature_spell_name,
    ]
    .into_iter()
    .flatten()
    {
        second.push(pill(name, palette.accent, PillState::default()));
    }
    second.extend(
        deck.tags
            .iter()
            .map(|t| pill(&deck_tag_label(t), palette.muted, PillState::default())),
    );
    second.extend(
        deck.other_tags
            .iter()
            .map(|t| pill(t.display_name(), palette.warning, PillState::default())),
    );
    (Line::from(first), second)
}
