//! One deck: the Profile card beside the sections as an accordion, one open at a time, with the More sheet and dialogs over them.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Borders, Paragraph, Wrap};
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::domain::deck::{Format, draw_odds};

use super::{CloneDialog, DeckView, Dialog, Section, Target};
use crate::domain::deck::warning;
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::charts::{Column, columns, row};
use crate::inbound::widgets::measure::height_of;
use crate::inbound::widgets::pill::{self, PillState, pill};
use crate::inbound::widgets::{action_bar, mana, panel, skeleton};

/// A vertical chart is at most this tall, frame included, so a small deck's bars stay short.
const CHART_HEIGHT: u16 = 14;

/// The profile's labels take this many cells before their values.
const LABEL_WIDTH: usize = 16;

/// The profile column is this wide, in cells.
const PROFILE_WIDTH: u16 = 38;

/// Cards drawn before the first turn's draw: the opening hand.
const OPENING_HAND: u32 = 7;

/// The budget's currencies, in the app's order.
const CURRENCIES: [PriceCurrency; 3] = [PriceCurrency::Usd, PriceCurrency::Eur, PriceCurrency::Tix];

/// The color chart's labels, by WUBRG place, for coloring each column.
const COLOR_LABELS: [&str; 5] = ["white", "blue", "black", "red", "green"];

/// Ghost rows standing where the stats will load.
const SKELETON_ROWS: u16 = 6;

/// The Clone dialog is at most this wide inside its panel, in cells.
const DIALOG_WIDTH: u16 = 56;

pub(crate) fn draw(f: &mut Frame<'_>, shared: &Shared, view: &DeckView) {
    let palette = &shared.palette;
    let [header, body, status, keys] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Fill(1),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .margin(1)
    .areas(f.area());

    f.render_widget(
        Line::from(vec![
            Span::styled("Deck", palette.accent).bold(),
            Span::styled("  ? hint", palette.muted),
        ]),
        header,
    );

    let [profile_area, sections_area] =
        Layout::horizontal([Constraint::Length(PROFILE_WIDTH), Constraint::Fill(1)]).areas(body);
    draw_profile(f, palette, view, profile_area);
    draw_accordion(f, palette, view, sections_area);

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    let mut keys_shown = vec![("j/k", "move"), ("Enter", "open")];
    match view.open {
        Some(Section::Budget) if view.has_cards() => keys_shown.push(("h/l", "currency")),
        Some(Section::DrawOdds) => keys_shown.push(("-/+", "turn")),
        _ => {}
    }
    keys_shown.extend([("i", "info"), ("e", "Edit"), ("c", "Cards"), ("m", "More")]);
    if view.deck.is_some() {
        keys_shown.push(("s", "swipe"));
    }
    keys_shown.extend([("Esc", "Back"), ("q", "quit")]);
    f.render_widget(action_bar::line(palette, &keys_shown), keys);

    if let Some(menu) = &view.more {
        menu.draw(f, palette, f.area());
    }
    match &view.dialog {
        Some(
            Dialog::Buy(alert)
            | Dialog::Share(alert)
            | Dialog::ClearSkips(alert)
            | Dialog::Delete(alert),
        ) => alert.draw(f, palette),
        Some(Dialog::Clone(clone)) => draw_clone(f, palette, clone),
        Some(Dialog::Info(popup)) => popup.draw(f, palette),
        None => {}
    }
}

fn draw_profile(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let profile = &view.profile;
    let value_width = usize::from(area.width.saturating_sub(2)).saturating_sub(LABEL_WIDTH);
    let mut lines = Vec::new();
    let mut field = |label: &str, value: String| {
        if value.chars().count() <= value_width {
            lines.push(Line::from(vec![
                Span::styled(format!("{label:<LABEL_WIDTH$}"), palette.muted),
                Span::raw(value),
            ]));
        } else {
            lines.push(Line::from(Span::styled(label.to_owned(), palette.muted)));
            lines.push(Line::from(format!("  {value}")));
        }
    };
    let cards = view.metrics.as_ref().map_or_else(
        || profile.card_count.max(0),
        |m| i64::try_from(m.total_cards).unwrap_or(i64::MAX),
    );
    field("Name", profile.name.to_string());
    field("Cards", cards.to_string());
    field(
        "Format",
        profile
            .format
            .map_or_else(|| "None".to_owned(), |f| f.display_name().to_owned()),
    );
    if let Some(format) = profile.format.filter(Format::has_commander) {
        let label = if format.has_signature_spell() {
            "Oathbreaker"
        } else {
            "Commander"
        };
        field(
            label,
            profile
                .commander_name
                .clone()
                .unwrap_or_else(|| "None".to_owned()),
        );
    }
    for (label, name) in [
        ("Partner", &profile.partner_commander_name),
        ("Background", &profile.background_name),
        ("Signature spell", &profile.signature_spell_name),
    ] {
        if let Some(name) = name {
            field(label, name.clone());
        }
    }
    if let Some(power) = profile.power_level {
        field("Power level", power.display_name().to_owned());
    }
    f.render_widget(
        Paragraph::new(lines).wrap(Wrap { trim: true }).block(
            Block::bordered()
                .title(" Profile ")
                .border_style(palette.border),
        ),
        area,
    );
}

/// The section headers stacked, the open one's body under its header, and a skeleton where stats are still loading.
fn draw_accordion(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let area = area.inner(ratatui::layout::Margin::new(1, 0));
    if view.failed {
        f.render_widget(Line::from("Could not load deck").style(palette.muted), area);
        return;
    }
    let sections = view.sections();
    let coming = view.stats_coming();
    if sections.is_empty() && !coming {
        f.render_widget(
            Line::from("Stats fill in as the deck grows.").style(palette.muted),
            area,
        );
        return;
    }

    let mut constraints = Vec::new();
    for section in &sections {
        constraints.push(Constraint::Length(1));
        if view.open == Some(*section) {
            constraints.push(Constraint::Fill(1));
        }
    }
    if coming {
        constraints.push(Constraint::Length(SKELETON_ROWS));
    }
    if view.open.is_none() {
        constraints.push(Constraint::Fill(1));
    }
    let areas = Layout::vertical(constraints).split(area);
    let mut slots = areas.iter().copied();
    let focused = view.focused();
    for section in &sections {
        let Some(header_area) = slots.next() else {
            return;
        };
        let header_focused = focused == Some(Target::Header(*section));
        f.render_widget(
            header_line(palette, view, *section, header_focused),
            header_area,
        );
        if view.open == Some(*section) {
            let Some(body) = slots.next() else {
                return;
            };
            let block =
                Block::new()
                    .borders(Borders::TOP)
                    .border_style(if *section == Section::Warnings {
                        palette.warning
                    } else {
                        palette.border
                    });
            let inner = block.inner(body);
            f.render_widget(block, body);
            draw_body(f, palette, view, *section, inner);
        }
    }
    if coming && let Some(ghost) = slots.next() {
        f.render_widget(
            Paragraph::new(skeleton::rows(
                palette,
                usize::from(SKELETON_ROWS),
                usize::from(ghost.width),
            )),
            ghost,
        );
    }
}

/// A section's header: the arrow and title, its badge, and while open its accessory.
fn header_line(
    palette: &Palette,
    view: &DeckView,
    section: Section,
    focused: bool,
) -> Line<'static> {
    let open = view.open == Some(section);
    let title_style = if section == Section::Warnings {
        palette.warning
    } else {
        palette.accent
    };
    let mut spans = vec![
        Span::raw(if focused { "> " } else { "  " }),
        Span::styled(if open { "▾ " } else { "▸ " }, palette.muted),
        Span::styled(section.title().to_owned(), title_style.bold()),
    ];
    let profile = &view.profile;
    let badge = match section {
        Section::Tags => Some(
            profile
                .tags
                .len()
                .saturating_add(profile.oracle_tags.len())
                .saturating_add(profile.other_tags.len()),
        ),
        Section::Warnings => view.deck.as_ref().map(|d| d.warnings.len()),
        _ => None,
    };
    if let Some(badge) = badge {
        spans.push(Span::styled(format!("  {badge}"), palette.muted));
    }
    if open && section == Section::Budget && view.has_cards() {
        spans.push(Span::raw("   "));
        spans.extend(CURRENCIES.iter().flat_map(|c| {
            [
                pill(
                    c.label(),
                    palette.muted,
                    PillState {
                        picked: *c == view.currency,
                        focused: false,
                    },
                ),
                Span::raw(" "),
            ]
        }));
    }
    let line = Line::from(spans);
    if focused { line.bold() } else { line }
}

fn draw_body(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, section: Section, area: Rect) {
    match section {
        Section::Budget => draw_budget(f, palette, view, area),
        Section::Tags => draw_tags(f, palette, view, area),
        Section::Distributions => {
            if let Some(m) = view.charted() {
                draw_distributions(f, palette, view, m, area);
            }
        }
        Section::Mana => draw_mana(f, palette, view, area),
        Section::DrawOdds => {
            if let Some(m) = view.charted() {
                draw_draw_odds(f, palette, m, view.turn, area);
            }
        }
        Section::Warnings => draw_warnings(f, palette, view, area),
    }
}

fn stat(palette: &Palette, label: &str, value: String) -> Line<'static> {
    Line::from(vec![
        Span::styled(format!("{label:<22}"), palette.muted),
        Span::raw(value).bold(),
    ])
}

fn draw_budget(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let currency = view.currency;
    let metrics = view.metrics.as_ref();
    let (total, average) = metrics.map_or((None, None), |m| match currency {
        PriceCurrency::Usd => (m.total_price_usd, m.avg_price_usd),
        PriceCurrency::Eur => (m.total_price_eur, m.avg_price_eur),
        PriceCurrency::Tix => (m.total_price_tix, m.avg_price_tix),
    });
    let mut total_text = currency.format_amount(total.unwrap_or(0.0));
    if let Some(target) = view.profile.price_target {
        let target_currency = view.profile.price_target_currency.unwrap_or_default();
        total_text = format!("{total_text} / {}", target_currency.format_amount(target));
    }
    let mut lines = vec![stat(palette, "Total price", total_text)];
    if metrics.is_some() {
        lines.push(stat(
            palette,
            "Average card price",
            average.map_or_else(|| "N/A".to_owned(), |a| currency.format_amount(a)),
        ));
    }
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_tags(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let profile = &view.profile;
    let mut lines = Vec::new();
    let mut group = |label: &str, tags: Vec<String>, style: Style| {
        if !tags.is_empty() {
            lines.push(Line::from(Span::styled(label.to_owned(), palette.muted)));
            lines.push(pill::row(
                tags.iter()
                    .map(|t| pill(t, style, PillState::default()))
                    .collect(),
            ));
            lines.push(Line::default());
        }
    };
    group(
        "Deck tags",
        profile.tags.iter().map(|t| deck_tag_label(t)).collect(),
        palette.accent,
    );
    group("Oracle tags", profile.oracle_tags.clone(), palette.muted);
    group(
        "Other tags",
        profile
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
        palette.warning,
    );
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn draw_distributions(
    f: &mut Frame<'_>,
    palette: &Palette,
    view: &DeckView,
    metrics: &DeckMetrics,
    area: Rect,
) {
    let one_decimal = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
    let has_pt = metrics.avg_power.is_some() || metrics.avg_toughness.is_some();
    let [pt, types_area, roles_area] = Layout::vertical([
        Constraint::Length(u16::from(has_pt)),
        Constraint::Max(CHART_HEIGHT),
        Constraint::Fill(1),
    ])
    .areas(area);
    if has_pt {
        f.render_widget(
            stat(
                palette,
                "Average P/T",
                format!(
                    "{} / {}",
                    one_decimal(metrics.avg_power),
                    one_decimal(metrics.avg_toughness)
                ),
            ),
            pt,
        );
    }
    let types: Vec<Column> = metrics
        .type_bars()
        .into_iter()
        .map(|(label, count, _)| Column {
            label: label.to_uppercase(),
            count,
            style: palette.accent,
        })
        .collect();
    columns(f, palette, "Types", &types, types_area);

    let Some(roles) = metrics.card_role_bars() else {
        return;
    };
    let focused = view.focused();
    let label_width = roles.iter().map(|(l, _, _)| l.len()).max().unwrap_or(0);
    let bar_width = usize::from(roles_area.width).saturating_sub(label_width.saturating_add(10));
    let mut lines = vec![Line::from(Span::styled("Roles", palette.muted))];
    let mut focus_line = 0;
    for (label, count, pct) in roles {
        let tags = metrics.card_role_tags.get(label);
        let is_focused = focused == Some(Target::Role(label.to_owned()));
        if is_focused {
            focus_line = lines.len();
        }
        let label_style = match (tags.is_some(), is_focused) {
            (true, true) => palette.accent.bold().reversed(),
            (true, false) => palette.accent.underlined(),
            (false, _) => palette.muted,
        };
        let mut line = row(
            Span::styled(label.to_uppercase(), label_style),
            label_width,
            pct,
            bar_width,
            palette.accent,
            vec![Span::raw(count.to_string())],
        );
        line.spans
            .insert(0, Span::raw(if is_focused { "> " } else { "  " }));
        lines.push(line);
        if view.role_open.as_deref() != Some(label) {
            continue;
        }
        for tag in tags.into_iter().flatten() {
            let tag_focused = focused == Some(Target::Tag(tag.clone()));
            if tag_focused {
                focus_line = lines.len();
            }
            lines.push(Line::from(vec![
                Span::raw(if tag_focused { ">     " } else { "      " }),
                pill(
                    tag,
                    palette.muted,
                    PillState {
                        picked: view.tag_open.as_deref() == Some(tag.as_str()),
                        focused: tag_focused,
                    },
                ),
            ]));
            if view.tag_open.as_deref() == Some(tag.as_str()) {
                let description = view
                    .tag_description(tag)
                    .unwrap_or("No description yet")
                    .to_owned();
                lines.push(Line::from(vec![
                    Span::raw("        "),
                    Span::styled(description, palette.base),
                ]));
                let examples_focused = focused == Some(Target::Examples(tag.clone()));
                if examples_focused {
                    focus_line = lines.len();
                }
                lines.push(Line::from(vec![
                    Span::raw(if examples_focused {
                        ">       "
                    } else {
                        "        "
                    }),
                    pill(
                        "Examples",
                        palette.accent,
                        PillState {
                            picked: false,
                            focused: examples_focused,
                        },
                    ),
                ]));
            }
        }
    }
    let scroll = focus_line.saturating_sub(usize::from(roles_area.height).saturating_sub(1));
    f.render_widget(
        Paragraph::new(lines).scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        roles_area,
    );
}

fn draw_mana(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let metrics = view.charted();
    let lands = metrics.map_or(0, |m| m.land_count);
    let lands_text = view
        .profile
        .land_target
        .map_or_else(|| lands.to_string(), |target| format!("{lands} / {target}"));
    let mut summary = vec![stat(palette, "Lands", lands_text)];
    if let Some(m) = metrics {
        summary.push(stat(
            palette,
            "Average mana value",
            format!("{:.1}", m.avg_cmc),
        ));
    }
    let Some(m) = metrics else {
        f.render_widget(Paragraph::new(summary), area);
        return;
    };

    let fulfillment = m.mana_balance_rows();
    let [summary_area, charts_area, fulfillment_area] = Layout::vertical([
        Constraint::Length(2),
        Constraint::Max(CHART_HEIGHT),
        Constraint::Length(
            u16::try_from(fulfillment.len())
                .unwrap_or(0)
                .saturating_add(1),
        ),
    ])
    .areas(area);
    f.render_widget(Paragraph::new(summary), summary_area);

    let [curve_area, colors_area] =
        Layout::horizontal([Constraint::Fill(1), Constraint::Fill(1)]).areas(charts_area);
    let curve: Vec<Column> = m
        .mana_curve_bars()
        .into_iter()
        .zip(["0", "1", "2", "3", "4", "5", "6+"])
        .map(|((count, _), label)| Column {
            label: label.to_owned(),
            count,
            style: palette.accent,
        })
        .collect();
    columns(f, palette, "Mana curve", &curve, curve_area);
    let colors: Vec<Column> = m
        .color_bars()
        .into_iter()
        .map(|(label, count, _)| Column {
            label: label.to_owned(),
            count,
            style: color_style(palette, label),
        })
        .collect();
    columns(f, palette, "Colors", &colors, colors_area);

    if !fulfillment.is_empty() {
        let bar_width = usize::from(fulfillment_area.width).saturating_sub(16);
        let mut lines = vec![Line::from(Span::styled(
            "Mana cost fulfillment",
            palette.muted,
        ))];
        lines.extend(fulfillment.into_iter().map(|r| {
            let style = mana::letter_style(palette, r.label);
            let mark = if r.is_surplus { "✔ " } else { "  " };
            row(
                Span::styled(r.label, style.bold()),
                1,
                r.fill_pct,
                bar_width,
                style,
                vec![
                    Span::styled(mark, palette.add),
                    Span::raw(format!("{}/{}", r.produced, r.consumed)),
                ],
            )
        }));
        f.render_widget(Paragraph::new(lines), fulfillment_area);
    }
}

fn draw_draw_odds(
    f: &mut Frame<'_>,
    palette: &Palette,
    metrics: &DeckMetrics,
    turn: u32,
    area: Rect,
) {
    let (deck_size, buckets) = metrics.draw_odds_buckets();
    let heading = if turn == 0 {
        "Odds of ≥1 in opening hand".to_owned()
    } else {
        format!("Odds of ≥1 by turn {turn}")
    };
    let draws = OPENING_HAND.saturating_add(turn);
    let label_width = buckets.iter().map(|(l, _)| l.len()).max().unwrap_or(0);
    let bar_width = usize::from(area.width).saturating_sub(label_width.saturating_add(8));
    let mut lines = vec![
        Line::from(vec![
            Span::styled("- ", palette.accent.bold()),
            Span::raw(heading).bold(),
            Span::styled(" +", palette.accent.bold()),
        ]),
        Line::default(),
    ];
    lines.extend(buckets.into_iter().map(|(label, successes)| {
        let odds = draw_odds::p_at_least_one(deck_size, successes, draws);
        let pct = percent(odds);
        row(
            Span::raw(label.to_uppercase()),
            label_width,
            pct,
            bar_width,
            palette.accent,
            vec![Span::raw(format!("{pct}%"))],
        )
    }));
    f.render_widget(Paragraph::new(lines), area);
}

/// A probability as a whole percentage, rounded.
fn percent(probability: f64) -> u32 {
    let clamped = (probability.clamp(0.0, 1.0) * 100.0).round();
    (0..=100_u32)
        .find(|&p| f64::from(p) >= clamped)
        .unwrap_or(100)
}

/// Each warning's message, with its fix buttons under it.
fn draw_warnings(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let focused = view.focused();
    let mut lines: Vec<Line<'static>> = Vec::new();
    let mut focus_line = 0;
    for (i, w) in view.deck.iter().flat_map(|d| &d.warnings).enumerate() {
        lines.push(Line::from(vec![
            Span::styled("! ", palette.warning.bold()),
            Span::styled(w.to_string(), palette.muted),
        ]));
        let Some(fix) = warning::fix(w) else {
            continue;
        };
        let mut buttons = vec![Span::raw("    ")];
        for (button, label) in warning::labels(fix).iter().enumerate() {
            let is_focused = focused == Some(Target::Fix { warning: i, button });
            if is_focused {
                focus_line = lines.len();
            }
            buttons.push(pill(
                label,
                palette.warning,
                PillState {
                    picked: is_focused,
                    focused: is_focused,
                },
            ));
            buttons.push(Span::raw(" "));
        }
        lines.push(Line::from(buttons));
    }
    let scroll = focus_line.saturating_sub(usize::from(area.height).saturating_sub(1));
    f.render_widget(
        Paragraph::new(lines).scroll((u16::try_from(scroll).unwrap_or(u16::MAX), 0)),
        area,
    );
}

/// The mana style for a color chart label such as `green`; multicolor in the warning color, colorless muted.
fn color_style(palette: &Palette, label: &str) -> Style {
    match COLOR_LABELS.iter().position(|l| *l == label) {
        Some(i) => mana::style(palette, i),
        None if label == "multi" => palette.warning,
        None => palette.muted,
    }
}

/// The Clone dialog: its title and body, the name box, and Cancel and Save.
fn draw_clone(f: &mut Frame<'_>, palette: &Palette, clone: &CloneDialog) {
    let body = Paragraph::new("Make an exact copy of your deck under a new name.")
        .wrap(Wrap { trim: true });
    let body_height = height_of(body.line_count(DIALOG_WIDTH));
    let inner = panel::draw(
        f,
        palette,
        f.area(),
        DIALOG_WIDTH,
        body_height.saturating_add(7),
    );
    let [title_area, body_area, input_area, _, buttons_area] = Layout::vertical([
        Constraint::Length(1),
        Constraint::Length(body_height.saturating_add(1)),
        Constraint::Length(3),
        Constraint::Length(1),
        Constraint::Length(1),
    ])
    .areas(inner);
    f.render_widget(
        Line::from("Clone deck").style(palette.accent).bold(),
        title_area,
    );
    f.render_widget(body, body_area);
    clone.name.draw(f, palette, input_area, "Name", !clone.busy);
    if !clone.busy {
        f.set_cursor_position(clone.name.cursor_position(input_area));
    }
    let save = if clone.busy { "Cloning..." } else { "Save" };
    let save_style = if clone.trimmed().is_some() {
        palette.accent
    } else {
        palette.muted
    };
    f.render_widget(
        Line::from(vec![
            pill(
                "Cancel",
                palette.accent,
                PillState {
                    picked: clone.focused == 0,
                    focused: false,
                },
            ),
            Span::raw("  "),
            pill(
                save,
                save_style,
                PillState {
                    picked: clone.focused == 1,
                    focused: false,
                },
            ),
        ]),
        buttons_area,
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn percent_rounds_and_stays_in_range() {
        assert_eq!(percent(0.0), 0);
        assert_eq!(percent(0.394), 39);
        assert_eq!(percent(0.395), 40);
        assert_eq!(percent(1.0), 100);
        assert_eq!(percent(1.7), 100);
        assert_eq!(percent(-0.2), 0);
    }
}
