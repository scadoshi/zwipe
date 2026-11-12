//! A deck page's stat panels' bodies, shared by every client that shows a deck: Budget's prices,
//! the deck's tags, Distributions' charts, Mana's lands and charts, and Draw odds. The host draws
//! each panel's frame (a fold on a wide page, an accordion section on a phone) around them.
//!
//! A body's `?` buttons come from the host through [`StatHints`] in context, by [`StatTopic`], so
//! each client opens its own hints; without it the bodies draw none.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::deck_tag_label;
use zwipe_ui::colors::ColorCounts;
use zwipe_ui::deck::view::{StatsProfile, with_unpriced};

use crate::{Chip, ColorChart, DeckCharts, DrawOdds, ManaCurve, ManaFulfillment};

/// What a stat panel's `?` explains.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum StatTopic {
    /// The deck's own tags.
    DeckTags,
    /// The deck's Oracle tags.
    OracleTags,
    /// The type chart.
    TypeChart,
    /// The card role chart.
    RoleChart,
    /// The Oracle tag chart.
    OracleTagChart,
    /// The mana curve.
    ManaCurve,
    /// The color chart.
    ColorChart,
    /// The mana fulfillment rows.
    ManaFulfillment,
    /// The draw odds.
    DrawOdds,
}

/// The host's `?` buttons for the stat panels: the button that opens a topic's hint.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct StatHints(pub Callback<StatTopic, Element>);

/// The `?` for `topic` from the [`StatHints`] in context; nothing without one.
fn hint(topic: StatTopic) -> Element {
    try_use_context::<StatHints>().map_or_else(|| rsx! {}, |hints| hints.0.call(topic))
}

/// A label and its value on one row.
#[component]
pub fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", "{label}" }
            span { class: "info-row-value", "{value}" }
        }
    }
}

/// The currency chips in Budget's header, once there are cards to price.
#[component]
pub fn CurrencyChips(mut currency: Signal<PriceCurrency>) -> Element {
    let shown = currency();
    rsx! {
        div { class: "chip-row", style: "margin-bottom:0;",
            for each in PriceCurrency::all().iter().copied() {
                Chip {
                    key: "{each.label()}",
                    selected: shown == each,
                    onclick: move |_| currency.set(each),
                    "{each.label()}"
                }
            }
        }
    }
}

/// Budget's body: the total and average price.
#[component]
pub fn BudgetBody(
    profile: StatsProfile,
    metrics: Option<DeckMetrics>,
    currency: Signal<PriceCurrency>,
) -> Element {
    let shown = currency();
    let (total, average) = match (&metrics, shown) {
        (Some(m), PriceCurrency::Eur) => (m.total_price_eur, m.avg_price_eur),
        (Some(m), PriceCurrency::Tix) => (m.total_price_tix, m.avg_price_tix),
        (Some(m), PriceCurrency::Usd) => (m.total_price_usd, m.avg_price_usd),
        (None, _) => (None, None),
    };
    let unpriced = metrics.as_ref().map_or(0, |m| m.unpriced(shown));
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            InfoRow {
                label: "Total price",
                value: with_unpriced(profile.total_price_line(total, shown), unpriced),
            }
            if metrics.is_some() {
                InfoRow {
                    label: "Average card price",
                    value: average
                        .map_or_else(
                            || "N/A".to_owned(),
                            |v| with_unpriced(shown.format_amount(v), unpriced),
                        ),
                }
            }
        }
    }
}

/// One stacked tag row: its label (with a `?` when it has one) over chips.
#[component]
fn TagRow(label: String, topic: Option<StatTopic>, chips: Vec<String>) -> Element {
    rsx! {
        div { class: "settings-row info-row-stacked",
            span { style: "display:flex;align-items:center;gap:0.35rem;",
                span { class: "settings-row-label", "{label}" }
                if let Some(topic) = topic {
                    {hint(topic)}
                }
            }
            span { class: "info-row-value info-row-tags",
                for chip in chips {
                    span { key: "{chip}", class: "stat-chip stat-chip-muted", "{chip}" }
                }
            }
        }
    }
}

/// Tags' body: deck, oracle and other tags, each row only when it has any.
#[component]
pub fn TagsBody(profile: StatsProfile) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;",
                if !profile.tags.is_empty() {
                    TagRow {
                        label: "Deck tags",
                        topic: StatTopic::DeckTags,
                        chips: profile.tags.iter().map(|t| deck_tag_label(t)).collect::<Vec<_>>(),
                    }
                }
                if !profile.oracle_tags.is_empty() {
                    TagRow {
                        label: "Oracle tags",
                        topic: StatTopic::OracleTags,
                        chips: profile.oracle_tags.clone(),
                    }
                }
                if !profile.other_tags.is_empty() {
                    TagRow {
                        label: "Other tags",
                        chips: profile.other_tags.iter().map(|t| t.display_name().to_owned()).collect::<Vec<_>>(),
                    }
                }
        }
    }
}

/// The metrics with a card to chart, if any.
#[must_use]
pub fn charted(metrics: Option<&DeckMetrics>) -> Option<DeckMetrics> {
    metrics
        .filter(|m| m.land_count > 0 || m.nonland_count > 0)
        .cloned()
}

/// Distributions' body: the average P/T and the type and role charts. `describe_tag` and
/// `on_examples` are the host's Oracle tag lookups, handed to the charts.
#[component]
pub fn DistributionsBody(
    metrics: DeckMetrics,
    #[props(default)] describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] on_examples: Option<Callback<String>>,
) -> Element {
    let average_pt = (metrics.avg_power.is_some() || metrics.avg_toughness.is_some()).then(|| {
        let one = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
        format!(
            "{} / {}",
            one(metrics.avg_power),
            one(metrics.avg_toughness)
        )
    });
    rsx! {
        div { class: "deck-stat-body",
            if let Some(value) = average_pt {
                div { class: "settings-row",
                    span { class: "settings-row-label", "Average P/T" }
                    span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
                }
            }
            DeckCharts {
                type_bars: metrics.type_bars(),
                category_bars: metrics.card_role_bars(),
                oracle_tag_bars: metrics.oracle_tag_bars(),
                role_tags: metrics.card_role_tags.clone(),
                describe_tag,
                on_examples,
                type_hint: hint(StatTopic::TypeChart),
                role_hint: hint(StatTopic::RoleChart),
                oracle_tag_hint: hint(StatTopic::OracleTagChart),
            }
        }
    }
}

/// Mana's body: the lands against their target, and with a card to chart the average mana
/// value, the curve, the colors and the fulfillment.
#[component]
pub fn ManaBody(
    charts: Option<DeckMetrics>,
    land_count: usize,
    land_target: Option<i32>,
) -> Element {
    let lands = match land_target {
        Some(target) => format!("{land_count} / {target}"),
        None => land_count.to_string(),
    };
    rsx! {
        div { class: "deck-stat-body",
            InfoRow { label: "Lands", value: lands }
            if let Some(m) = charts {
                InfoRow { label: "Average mana value", value: format!("{:.1}", m.avg_cmc) }
                ManaCurve {
                    mana_curve_bars: m.mana_curve_bars(),
                    hint: hint(StatTopic::ManaCurve),
                }
                ColorChart {
                    bars: ColorCounts::from(&m).bars(),
                    hint: hint(StatTopic::ColorChart),
                }
                ManaFulfillment {
                    rows: m.mana_balance_rows(),
                    hint: hint(StatTopic::ManaFulfillment),
                }
            }
        }
    }
}

/// The draw odds with ← and → stepping the turn while one of its `-` and `+` buttons has focus.
#[component]
pub fn DrawOddsStepper(metrics: DeckMetrics) -> Element {
    let (deck_size, buckets) = metrics.draw_odds_buckets();
    rsx! {
        div {
            id: "web-draw-odds",
            class: "deck-stat-body",
            onkeydown: move |event: Event<KeyboardData>| {
                let at = match event.key() {
                    Key::ArrowLeft => 0,
                    Key::ArrowRight => 1,
                    _ => return,
                };
                event.prevent_default();
                document::eval(&format!(
                    "document.querySelectorAll('#web-draw-odds .stepper-btn')[{at}]?.click()"
                ));
            },
            DrawOdds { deck_size, buckets }
        }
    }
}

/// The `?` for the Draw odds panel's header, from the [`StatHints`] in context.
#[must_use]
pub fn draw_odds_hint() -> Option<Element> {
    try_use_context::<StatHints>().map(|hints| hints.0.call(StatTopic::DrawOdds))
}
