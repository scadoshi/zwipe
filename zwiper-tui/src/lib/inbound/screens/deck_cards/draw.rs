//! The deck cards view: the deck's name and chips, featured cards, quick add, the Group by, Boards and Show rows, the card list, and the keys.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Board, deck_metrics, deck_tag};
use zwipe_ui::filter;
use zwipe_ui::screens::deck_cards::{self, QuickAddResults};

use super::{DeckCardsView, Item, Kind};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::{action_bar, card_info, mana, measure, otag, skeleton};

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &DeckCardsView) {
    let palette = &shared.palette;
    let area = f.area();
    let bar = action_bar::with_help(
        palette,
        key_line(palette, view, shared),
        action_bar::Hint::Question,
    );
    let bar_height = action_bar::height(&bar, area.width.saturating_sub(2));
    let [header, controls, list, keys] = Layout::vertical([
        Constraint::Length(3),
        Constraint::Length(if view.model.is_loaded() { 6 } else { 4 }),
        Constraint::Fill(1),
        Constraint::Length(bar_height),
    ])
    .horizontal_margin(1)
    .areas(area);

    f.render_widget(Paragraph::new(header_lines(palette, view)), header);
    if view.model.is_loaded() {
        f.render_widget(Paragraph::new(control_lines(palette, view)), controls);
        draw_list(f, shared, view, list);
    } else {
        let width = usize::from(area.width.saturating_sub(2));
        f.render_widget(Paragraph::new(skeleton::rows(palette, 3, width)), controls);
        f.render_widget(Paragraph::new(skeleton::rows(palette, 8, width)), list);
    }
    f.render_widget(action_bar::wrap(bar, keys.width), keys);

    if let Some((card, face)) = &view.preview {
        Sheet::new(
            &card.scryfall_data.name,
            card_info::proxy_lines(palette, card, *face),
        )
        .with_footer("t", "flip")
        .with_footer("any key", "close")
        .draw(f, palette, area);
    }
    if let Some(printing) = &view.printing {
        printing.draw(f, palette, area);
    }
    if let Some(sheet) = &view.sheet {
        card_sheets::filter::draw::draw(f, shared, sheet, area);
    }
}

fn chip(text: &str, style: Style) -> Span<'static> {
    Span::styled(format!(" {text} "), style.reversed())
}

/// The deck's name and chips, then the featured strip; ghosts until loaded.
fn header_lines(palette: &Palette, view: &DeckCardsView) -> Vec<Line<'static>> {
    let Some(profile) = view.model.profile() else {
        return skeleton::rows(palette, 2, 30);
    };
    let mut chips = Vec::new();
    if let Some(format) = profile.format {
        chips.push(chip(format.display_name(), palette.accent));
        chips.push(Span::raw(" "));
    }
    if let Some(power) = profile.power_level {
        chips.push(chip(power.display_name(), palette.warning));
        chips.push(Span::raw(" "));
    }
    for tag in &profile.tags {
        chips.push(chip(&deck_tag::deck_tag_label(tag), palette.muted));
        chips.push(Span::raw(" "));
    }
    for tag in &profile.other_tags {
        chips.push(chip(tag.display_name(), palette.warning));
        chips.push(Span::raw(" "));
    }
    let mut featured = Vec::new();
    for (card, role) in view.model.featured() {
        let style = if role == deck_cards::MVP {
            palette.warning
        } else {
            palette.accent
        };
        featured.push(Span::styled(format!("{role} "), style.bold()));
        featured.push(Span::raw(format!("{}   ", card.scryfall_data.name)));
    }
    vec![
        Line::from(profile.name.to_string()).bold(),
        Line::from(chips),
        Line::from(featured),
    ]
}

/// Quick add with its dropdown, then the Group by, Boards and Show rows.
fn control_lines(palette: &Palette, view: &DeckCardsView) -> Vec<Line<'static>> {
    let quick = &view.quick;
    let mut input = vec![
        Span::styled(format!("{} ", deck_cards::QUICK_ADD), palette.muted.bold()),
        Span::styled("[", palette.border),
    ];
    if quick.input.is_empty() {
        input.push(Span::styled(
            deck_cards::QUICK_ADD_PLACEHOLDER,
            palette.muted.italic(),
        ));
    } else {
        input.push(Span::raw(quick.input.value().to_owned()));
    }
    if quick.focused {
        input.push(Span::styled("▏", palette.accent));
    }
    input.push(Span::styled("]", palette.border));
    if !quick.input.is_empty() {
        input.push(Span::styled(" ×", palette.muted));
    }
    let dropdown = match view.model.quick_add() {
        None => Line::default(),
        Some(QuickAddResults::Searching) => Line::from(deck_cards::SEARCHING).style(palette.muted),
        Some(QuickAddResults::Empty) => Line::from(deck_cards::NO_RESULTS).style(palette.muted),
        Some(QuickAddResults::Found(results)) => Line::from(
            results
                .iter()
                .enumerate()
                .flat_map(|(i, c)| {
                    let style = if i == quick.highlight {
                        palette.accent.reversed()
                    } else {
                        palette.muted
                    };
                    [
                        Span::styled(format!(" {} ", c.scryfall_data.name), style),
                        Span::raw(" "),
                    ]
                })
                .collect::<Vec<_>>(),
        ),
    };
    let on = |on: bool| {
        if on {
            palette.accent.bold().reversed()
        } else {
            palette.muted
        }
    };
    let group_by: Vec<Span<'static>> = std::iter::once(Span::styled(
        format!("{} ", deck_cards::GROUP_BY),
        palette.muted.bold(),
    ))
    .chain(GroupByOption::all().into_iter().flat_map(|o| {
        [
            Span::styled(format!(" {o} "), on(o == view.model.group_by())),
            Span::raw(" "),
        ]
    }))
    .collect();
    let board_chip = |board: Board| {
        let active = view.model.board_on(board);
        let style = if !view.model.board_enabled(board) && !active {
            palette.border
        } else {
            on(active)
        };
        Span::styled(format!(" {} ", deck_cards::board_label(board)), style)
    };
    let mut boards = vec![Span::styled(
        format!("{} ", deck_cards::BOARDS),
        palette.muted.bold(),
    )];
    for (i, board) in deck_cards::BOARD_CHIPS.into_iter().enumerate() {
        if i > 0 {
            boards.push(Span::raw(" "));
        }
        boards.push(board_chip(board));
    }
    let show = Line::from(vec![
        Span::styled(format!("{} ", deck_cards::SHOW), palette.muted.bold()),
        Span::styled(
            format!(" {} ", deck_cards::TOKENS),
            on(view.model.show_tokens()),
        ),
        Span::raw(" "),
        Span::styled(
            format!(" {} ", deck_cards::COMMAND_ZONE),
            on(view.model.show_command_zone()),
        ),
    ]);
    vec![
        Line::from(input),
        dropdown,
        Line::from(group_by),
        Line::from(boards),
        show,
        Line::default(),
    ]
}

/// The list, scrolled to keep the selection in view.
fn draw_list(f: &mut Frame<'_>, shared: &Shared, view: &DeckCardsView, area: Rect) {
    let palette = &shared.palette;
    let items = view.items();
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut cursor_line = 0;
    // The open row's action lines, as where they start in `lines` and the lines themselves.
    let mut open_actions: Option<(usize, Vec<Line<'static>>)> = None;
    for (i, item) in items.iter().enumerate() {
        let selected = i == view.cursor && !view.quick.focused;
        if i == view.cursor {
            cursor_line = lines.len();
        }
        match item {
            Item::Header {
                key, title, pips, ..
            } => {
                let open = !view.model.is_collapsed(key);
                let mut spans = vec![
                    pointer(palette, selected),
                    Span::styled(if open { "▾ " } else { "▸ " }, palette.muted),
                ];
                if let Some(pips) = pips {
                    spans.extend(pip_spans(palette, pips));
                    spans.push(Span::raw(" "));
                }
                spans.push(Span::styled(title.clone(), palette.accent.bold()));
                lines.push(Line::from(spans));
            }
            Item::Card { card, kind } => {
                lines.push(row_line(
                    palette,
                    view,
                    card,
                    *kind,
                    selected,
                    usize::from(area.width),
                ));
                if view.expanded == Some(card.scryfall_data.id) && i == view.cursor {
                    lines.push(Line::from("    ────").style(palette.border));
                    let details: Vec<Line<'static>> =
                        card_info::details_lines(palette, card, view.face)
                            .into_iter()
                            .map(|l| {
                                let mut spans = vec![Span::raw("    ")];
                                spans.extend(l.spans);
                                Line::from(spans)
                            })
                            .collect();
                    lines.extend(card_info::wrap(details, usize::from(area.width), 4));
                    let tags = card_info::oracle_tags(card);
                    if let Some(slug) = view.open_tag.and_then(|i| tags.get(i)) {
                        let mut reveal =
                            vec![Line::from(otag::chip(palette, slug, /* open */ true))];
                        reveal.extend(otag::def_lines(palette, &shared.caches.oracle_tags, slug));
                        let indented: Vec<Line<'static>> = reveal
                            .into_iter()
                            .map(|l| {
                                let mut spans = vec![Span::raw("    ")];
                                spans.extend(l.spans);
                                Line::from(spans).style(l.style)
                            })
                            .collect();
                        lines.extend(card_info::wrap(indented, usize::from(area.width), 4));
                    }
                    let actions = action_bar::wrap(
                        actions_line(palette, view, card, *kind),
                        area.width.saturating_sub(4),
                    );
                    let actions: Vec<Line<'static>> = actions
                        .lines
                        .into_iter()
                        .map(|row| {
                            let mut spans = vec![Span::raw("    ")];
                            spans.extend(row.spans);
                            Line::from(spans)
                        })
                        .collect();
                    open_actions = Some((lines.len(), actions.clone()));
                    lines.extend(actions);
                    lines.push(Line::from("    ────").style(palette.border));
                }
            }
        }
    }
    if view.model.no_cards(&view.display) {
        lines.push(Line::from(deck_cards::NO_CARDS).style(palette.muted));
    }
    let shown = in_view(lines, cursor_line, open_actions, usize::from(area.height));
    f.render_widget(Paragraph::new(shown), area);
}

/// The `rows` of `lines` in view: from a third of the list above the cursor, scrolled further to show the open row's actions (`open_actions`, where they start and the lines) but never past its name, and with those actions kept in the bottom rows when the open row is taller than the list.
fn in_view(
    lines: Vec<Line<'static>>,
    cursor_line: usize,
    open_actions: Option<(usize, Vec<Line<'static>>)>,
    rows: usize,
) -> Vec<Line<'static>> {
    let mut scroll = cursor_line.saturating_sub(rows / 3);
    if let Some((start, actions)) = &open_actions {
        let end = start.saturating_add(actions.len());
        scroll = scroll.max(end.saturating_sub(rows)).min(cursor_line);
    }
    let scroll = scroll.min(lines.len().saturating_sub(rows));
    let pinned = open_actions
        .filter(|(start, actions)| {
            start.saturating_add(actions.len()) > scroll.saturating_add(rows)
        })
        .map(|(_, actions)| actions)
        .unwrap_or_default();
    lines
        .into_iter()
        .skip(scroll)
        .take(rows.saturating_sub(pinned.len()))
        .chain(pinned)
        .collect()
}

fn pointer(palette: &Palette, selected: bool) -> Span<'static> {
    if selected {
        Span::styled("› ", palette.accent.bold())
    } else {
        Span::raw("  ")
    }
}

fn pip_spans(palette: &Palette, colors: &[Color]) -> Vec<Span<'static>> {
    colors
        .iter()
        .filter_map(|c| Color::all().iter().position(|a| a == c))
        .map(|i| mana::pip(palette, i))
        .collect()
}

/// A compact row no wider than `width`: arrow, quantity, star and name, stats, price, color pips.
fn row_line(
    palette: &Palette,
    view: &DeckCardsView,
    card: &Card,
    kind: Kind,
    selected: bool,
    width: usize,
) -> Line<'static> {
    let data = &card.scryfall_data;
    let open = view.expanded == Some(data.id);
    let mvp = view.model.row(data.id).is_some_and(|r| r.mvp) && kind == Kind::Main;
    let quantity = match kind {
        Kind::Main | Kind::Board(_) => view.model.quantity(data.id),
        Kind::Zone(_) | Kind::Token => 1,
    };
    let mut spans = vec![
        pointer(palette, selected),
        Span::styled(if open { "▾ " } else { "▸ " }, palette.muted),
        Span::styled(format!("{quantity} "), palette.muted),
    ];
    if mvp {
        spans.push(Span::styled("★", palette.warning));
    }
    let name_style = if selected {
        palette.accent.bold()
    } else {
        palette.text()
    };
    let name_at = spans.len();
    spans.push(Span::styled(data.name.clone(), name_style));
    let stats = match (&data.power, &data.toughness, &data.loyalty) {
        (Some(p), Some(t), _) => Some(format!("{p}/{t}")),
        (_, _, Some(l)) => Some(l.clone()),
        _ => None,
    };
    if let Some(stats) = stats {
        spans.push(Span::styled(format!("  {stats}"), palette.muted));
    }
    let currency = view.model.currency();
    if let Some(price) = deck_metrics::card_price(data, currency) {
        spans.push(Span::styled(
            format!("  {}", currency.format_amount(price)),
            palette.muted,
        ));
    }
    spans.push(Span::raw("  "));
    spans.extend(pip_spans(palette, &data.color_identity));
    // A name too long for the row ends in an ellipsis, keeping its stats, price and pips.
    let overflow = Line::from(spans.clone()).width().saturating_sub(width);
    if overflow > 0
        && let Some(name) = spans.get_mut(name_at)
    {
        let keep = name.width().saturating_sub(overflow);
        name.content = measure::ellipsize(&name.content, keep).into();
    }
    Line::from(spans)
}

/// The open row's actions for its kind.
fn actions_line(palette: &Palette, view: &DeckCardsView, card: &Card, kind: Kind) -> Line<'static> {
    let id = card.scryfall_data.id;
    let mut keys: Vec<(&str, String)> = Vec::new();
    if card_info::face_count(card) > 1 {
        keys.push(("t", "flip".to_owned()));
    }
    keys.push(("i", "image".to_owned()));
    if !card_info::oracle_tags(card).is_empty() {
        keys.push(("Tab", "tags".to_owned()));
    }
    let row = view.model.row(id);
    let board = row.map(|r| r.board).unwrap_or_default();
    match kind {
        Kind::Main | Kind::Board(_) => {
            keys.push(("-/+", format!("×{}", view.model.quantity(id))));
            keys.push(("p", "printing".to_owned()));
            if kind == Kind::Main {
                let star = if row.is_some_and(|r| r.mvp) {
                    "unstar"
                } else {
                    "star"
                };
                keys.push(("s", star.to_owned()));
            }
            for (key, target, label) in [
                ("d", Board::Deck, "to mainboard"),
                ("m", Board::Maybeboard, "to maybeboard"),
                ("b", Board::Sideboard, "to sideboard"),
            ] {
                if board != target {
                    keys.push((key, label.to_owned()));
                }
            }
        }
        Kind::Zone(_) => keys.push(("p", "printing".to_owned())),
        Kind::Token => {}
    }
    let pairs: Vec<(&str, &str)> = keys.iter().map(|(k, a)| (*k, a.as_str())).collect();
    action_bar::line(palette, &pairs)
}

fn key_line(palette: &Palette, view: &DeckCardsView, shared: &Shared) -> Line<'static> {
    let filter_label = if filter::shows_dot(&view.filter) {
        "filter •"
    } else {
        "filter"
    };
    let mut keys = vec![
        ("Esc", "back"),
        ("a", "add"),
        ("r", "remove"),
        ("f", filter_label),
    ];
    if view
        .model
        .deck_id()
        .is_some_and(|id| !shared.cards.undo.is_empty(id))
    {
        keys.push(("u", "undo"));
    }
    if !view.filter.is_empty() {
        keys.push(("x", "reset filter"));
    }
    keys.extend([
        ("/", "quick add"),
        ("g", "group"),
        ("1-3", "boards"),
        ("?", "hint"),
    ]);
    action_bar::line(palette, &keys)
}
