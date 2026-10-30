//! One deck: its profile beside the app's stat sections, one tab at a time.

use ratatui::Frame;
use ratatui::layout::{Constraint, Layout, Rect};
use ratatui::style::{Style, Stylize};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, Paragraph, Tabs, Wrap};
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::domain::deck::draw_odds;

use super::{DeckView, Section};
use crate::domain::theme::Palette;
use crate::inbound::app::Shared;
use crate::inbound::widgets::charts::{Column, columns, row};
use crate::inbound::widgets::chips::{ChipRow, Pick};
use crate::inbound::widgets::{action_bar, mana};

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
        Line::from(Span::styled(view.profile.name.to_string(), palette.accent).bold()),
        header,
    );

    let [profile_area, stats_area] =
        Layout::horizontal([Constraint::Length(PROFILE_WIDTH), Constraint::Fill(1)]).areas(body);
    draw_profile(f, palette, view, profile_area);
    draw_stats(f, palette, view, stats_area);

    f.render_widget(
        Line::from(shared.status.clone()).style(palette.muted),
        status,
    );
    let mut keys_shown = vec![("Tab", "section")];
    match view.section {
        Section::Budget => keys_shown.push(("c", "currency")),
        Section::DrawOdds => keys_shown.push(("+/-", "turn")),
        _ => {}
    }
    if view.deck.is_some() {
        keys_shown.push(("s", "swipe"));
    }
    keys_shown.extend([("r", "reload"), ("Esc", "decks"), ("q", "quit")]);
    f.render_widget(action_bar::line(palette, &keys_shown), keys);
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
    let cards = view.metrics.as_ref().map_or(profile.card_count, |m| {
        i64::try_from(m.total_cards).unwrap_or(i64::MAX)
    });
    field("Name", profile.name.to_string());
    field("Cards", cards.to_string());
    field(
        "Format",
        profile
            .format
            .map_or_else(|| "None".to_owned(), |f| f.display_name().to_owned()),
    );
    if let Some(format) = profile
        .format
        .filter(zwipe_core::domain::deck::Format::has_commander)
    {
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

fn draw_stats(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let placeholder = if let Some(error) = &view.error {
        Some(Line::from(error.clone()).style(palette.error))
    } else if view.deck.is_none() {
        Some(Line::from("Loading deck...").style(palette.muted))
    } else if view.sections().is_empty() {
        Some(Line::from("Stats fill in as the deck grows.").style(palette.muted))
    } else {
        None
    };
    if let Some(placeholder) = placeholder {
        f.render_widget(
            Paragraph::new(placeholder).block(Block::bordered().border_style(palette.border)),
            area,
        );
        return;
    }

    let sections = view.sections();
    let warnings = view.deck.as_ref().map_or(0, |d| d.warnings.len());
    let titles: Vec<Line<'static>> = sections
        .iter()
        .map(|section| match section {
            Section::Warnings => {
                Line::from(format!("Warnings ({warnings})")).style(palette.warning)
            }
            other => Line::from(other.title()),
        })
        .collect();
    let selected = sections.iter().position(|s| *s == view.section);
    let [tabs_area, content] =
        Layout::vertical([Constraint::Length(1), Constraint::Fill(1)]).areas(area);
    f.render_widget(
        Tabs::new(titles)
            .select(selected)
            .style(palette.muted)
            .highlight_style(palette.accent.bold().underlined()),
        tabs_area,
    );

    let block = Block::bordered()
        .title(format!(" {} ", view.section.title()))
        .border_style(palette.border);
    let inner = block.inner(content);
    f.render_widget(block, content);
    match view.section {
        Section::Budget => draw_budget(f, palette, view, inner),
        Section::Tags => draw_tags(f, palette, view, inner),
        Section::Distributions => {
            if let Some(m) = view.charted() {
                draw_distributions(f, palette, m, inner);
            }
        }
        Section::Mana => draw_mana(f, palette, view, inner),
        Section::DrawOdds => {
            if let Some(m) = view.charted() {
                draw_draw_odds(f, palette, m, view.turn, inner);
            }
        }
        Section::Warnings => draw_warnings(f, palette, view, inner),
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
    let labels = CURRENCIES.map(|c| c.label());
    let picked = CURRENCIES.iter().position(|c| *c == currency).unwrap_or(0);
    let chips = ChipRow::new(&labels, Pick::One)
        .with_picked(picked)
        .line(palette, false);

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
    let lines = vec![
        chips,
        Line::default(),
        stat(palette, "Total price", total_text),
        stat(
            palette,
            "Average card price",
            average.map_or_else(|| "N/A".to_owned(), |a| currency.format_amount(a)),
        ),
    ];
    f.render_widget(Paragraph::new(lines), area);
}

fn draw_tags(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let profile = &view.profile;
    let mut lines = Vec::new();
    let mut group = |label: &str, tags: Vec<String>| {
        if !tags.is_empty() {
            lines.push(Line::from(Span::styled(label.to_owned(), palette.muted)));
            lines.push(Line::from(tags.join(", ")));
            lines.push(Line::default());
        }
    };
    group(
        "Deck tags",
        profile.tags.iter().map(|t| deck_tag_label(t)).collect(),
    );
    group("Oracle tags", profile.oracle_tags.clone());
    group(
        "Other tags",
        profile
            .other_tags
            .iter()
            .map(|t| t.display_name().to_owned())
            .collect(),
    );
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

fn draw_distributions(f: &mut Frame<'_>, palette: &Palette, metrics: &DeckMetrics, area: Rect) {
    let one_decimal = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
    let [pt, charts] = Layout::vertical([Constraint::Length(2), Constraint::Fill(1)]).areas(area);
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

    let roles = metrics.card_role_bars();
    let [types_area, roles_area] = if roles.is_some() {
        Layout::vertical([Constraint::Max(CHART_HEIGHT), Constraint::Fill(1)]).areas(charts)
    } else {
        let [types, _] =
            Layout::vertical([Constraint::Max(CHART_HEIGHT), Constraint::Fill(1)]).areas(charts);
        [types, Rect::default()]
    };
    let types: Vec<Column> = metrics
        .type_bars()
        .into_iter()
        .map(|(label, count, _)| Column {
            label: label.to_owned(),
            count,
            style: palette.accent,
        })
        .collect();
    columns(f, palette, "Types", &types, types_area);

    if let Some(roles) = roles {
        let label_width = roles.iter().map(|(l, _, _)| l.len()).max().unwrap_or(0);
        let bar_width = usize::from(roles_area.width).saturating_sub(label_width.saturating_add(8));
        let lines: Vec<Line<'static>> = roles
            .into_iter()
            .map(|(label, count, pct)| {
                row(
                    Span::raw(label),
                    label_width,
                    pct,
                    bar_width,
                    palette.accent,
                    vec![Span::raw(count.to_string())],
                )
            })
            .collect();
        f.render_widget(
            Paragraph::new(lines).block(Block::new().title(" Roles ").style(palette.base)),
            roles_area,
        );
    }
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
        Constraint::Length(3),
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
    let mut lines = vec![Line::from(heading).bold(), Line::default()];
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

fn draw_warnings(f: &mut Frame<'_>, palette: &Palette, view: &DeckView, area: Rect) {
    let lines: Vec<Line<'static>> = view
        .deck
        .iter()
        .flat_map(|d| &d.warnings)
        .map(|w| {
            Line::from(vec![
                Span::styled("! ", palette.warning.bold()),
                Span::raw(w.to_string()),
            ])
        })
        .collect();
    f.render_widget(Paragraph::new(lines).wrap(Wrap { trim: true }), area);
}

/// The mana style for a color chart label such as `green`; multicolor in the accent, colorless muted.
fn color_style(palette: &Palette, label: &str) -> Style {
    match COLOR_LABELS.iter().position(|l| *l == label) {
        Some(i) => mana::style(palette, i),
        None if label == "multi" => palette.warning,
        None => palette.muted,
    }
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
