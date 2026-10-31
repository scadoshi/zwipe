//! The deck cards view: the header, featured cards, quick add, the Group by, Boards and Show rows, the card list, and the keys.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::Paragraph;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::Board;
use zwipe_core::domain::deck::deck_metrics::card_price;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;

use super::{DeckCardsView, Dropdown, Item, Kind};
use crate::domain::theme::Palette;
use crate::domain::{deck_cards, filter};
use crate::inbound::app::Shared;
use crate::inbound::screens::card_sheets;
use crate::inbound::widgets::sheet::Sheet;
use crate::inbound::widgets::{action_bar, card_info, header, mana, otag, skeleton};

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &DeckCardsView) {
    let palette = &shared.palette;
    let area = f.area();
    let [title, header, controls, list, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(3),
        Constraint::Length(if view.loaded { 6 } else { 4 }),
        Constraint::Fill(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(area);

    header::draw(f, palette, title, "Deck cards", header::Corner::Hint);
    f.render_widget(Paragraph::new(header_lines(palette, view)), header);
    if view.loaded {
        f.render_widget(Paragraph::new(control_lines(palette, view)), controls);
        draw_list(f, shared, view, list);
    } else {
        let width = usize::from(area.width.saturating_sub(2));
        f.render_widget(Paragraph::new(skeleton::rows(palette, 3, width)), controls);
        f.render_widget(Paragraph::new(skeleton::rows(palette, 8, width)), list);
    }
    f.render_widget(key_line(palette, view, shared), keys);

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
    let Some(profile) = &view.profile else {
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
        chips.push(chip(&deck_tag_label(tag), palette.muted));
        chips.push(Span::raw(" "));
    }
    for tag in &profile.other_tags {
        chips.push(chip(tag.display_name(), palette.warning));
        chips.push(Span::raw(" "));
    }
    let mut featured = Vec::new();
    for (card, role) in deck_cards::featured(&view.zone, &view.entries, view.is_oathbreaker()) {
        let style = if role == "MVP" {
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
        Span::styled("Quick add ", palette.muted.bold()),
        Span::styled("[", palette.border),
    ];
    if quick.input.is_empty() {
        input.push(Span::styled("Search a card to add", palette.muted.italic()));
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
    let dropdown = match &quick.results {
        Dropdown::Closed => Line::default(),
        Dropdown::Searching => Line::from("Searching...").style(palette.muted),
        Dropdown::Results(results) if results.is_empty() => {
            Line::from("No results").style(palette.muted)
        }
        Dropdown::Results(results) => Line::from(
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
    let group_by: Vec<Span<'static>> =
        std::iter::once(Span::styled("Group by ", palette.muted.bold()))
            .chain(GroupByOption::all().into_iter().flat_map(|o| {
                [
                    Span::styled(format!(" {o} "), on(o == view.group_by)),
                    Span::raw(" "),
                ]
            }))
            .collect();
    let has = |board: Board| view.entries.iter().any(|e| e.deck_card.board == board);
    let board_chip = |label: &str, active: bool, enabled: bool| {
        let style = if !enabled && !active {
            palette.border
        } else {
            on(active)
        };
        Span::styled(format!(" {label} "), style)
    };
    let boards = Line::from(vec![
        Span::styled("Boards ", palette.muted.bold()),
        board_chip("Main", view.boards.main, true),
        Span::raw(" "),
        board_chip("Maybe", view.boards.maybe, has(Board::Maybeboard)),
        Span::raw(" "),
        board_chip("Side", view.boards.side, has(Board::Sideboard)),
    ]);
    let show = Line::from(vec![
        Span::styled("Show ", palette.muted.bold()),
        Span::styled(" Tokens ", on(view.show_tokens)),
        Span::raw(" "),
        Span::styled(" Command zone ", on(view.show_zone)),
        Span::raw(" "),
        Span::styled(" Art ", on(view.show_art)),
    ]);
    vec![
        Line::from(input),
        dropdown,
        Line::from(group_by),
        boards,
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
    for (i, item) in items.iter().enumerate() {
        let selected = i == view.cursor && !view.quick.focused;
        if i == view.cursor {
            cursor_line = lines.len();
        }
        match item {
            Item::Header {
                key, title, pips, ..
            } => {
                let open = !view.collapsed.contains(key);
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
                lines.push(row_line(palette, view, card, *kind, selected));
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
                        let mut reveal = vec![Line::from(otag::chip(palette, slug, true))];
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
                    lines.push(actions_line(palette, view, card, *kind));
                    lines.push(Line::from("    ────").style(palette.border));
                }
            }
        }
    }
    if view.boards.main && view.display.groups.is_empty() && view.display.lands.is_empty() {
        lines.push(Line::from("No cards").style(palette.muted));
    }
    let rows = usize::from(area.height);
    let scroll = cursor_line
        .saturating_sub(rows / 3)
        .min(lines.len().saturating_sub(rows));
    let shown: Vec<Line<'static>> = lines.into_iter().skip(scroll).take(rows).collect();
    f.render_widget(Paragraph::new(shown), area);
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

/// A compact row: arrow, quantity, star and name, stats, price, color pips.
fn row_line(
    palette: &Palette,
    view: &DeckCardsView,
    card: &Card,
    kind: Kind,
    selected: bool,
) -> Line<'static> {
    let data = &card.scryfall_data;
    let open = view.expanded == Some(data.id);
    let entry = view.entry(data.id);
    let mvp = entry.is_some_and(|e| e.deck_card.mvp_at.is_some()) && kind == Kind::Main;
    let quantity = match kind {
        Kind::Main | Kind::Board(_) => view.quantity(data.id),
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
        palette.base
    };
    spans.push(Span::styled(data.name.clone(), name_style));
    let stats = match (&data.power, &data.toughness, &data.loyalty) {
        (Some(p), Some(t), _) => Some(format!("{p}/{t}")),
        (_, _, Some(l)) => Some(l.clone()),
        _ => None,
    };
    if let Some(stats) = stats {
        spans.push(Span::styled(format!("  {stats}"), palette.muted));
    }
    let currency = view.currency();
    if let Some(price) = card_price(data, currency) {
        spans.push(Span::styled(
            format!("  {}", currency.format_amount(price)),
            palette.muted,
        ));
    }
    spans.push(Span::raw("  "));
    spans.extend(pip_spans(palette, &data.color_identity));
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
    let board = view
        .entry(id)
        .map(|e| e.deck_card.board)
        .unwrap_or_default();
    match kind {
        Kind::Main | Kind::Board(_) => {
            keys.push(("-/+", format!("×{}", view.quantity(id))));
            keys.push(("p", "printing".to_owned()));
            if kind == Kind::Main {
                let star = if view.entry(id).is_some_and(|e| e.deck_card.mvp_at.is_some()) {
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
    let mut line = action_bar::line(palette, &pairs);
    line.spans.insert(0, Span::raw("    "));
    line
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
    if !shared.cards.undo.is_empty(view.deck_id) {
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
