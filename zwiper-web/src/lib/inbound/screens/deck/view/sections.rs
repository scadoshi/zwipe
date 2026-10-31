//! The deck view's panels: the Profile card, and the Budget, Tags,
//! Distributions, Mana, Draw odds and Warnings sections.

use components::{
    Button, ButtonVariant, ColorChart, DeckCharts, DrawOdds, ManaCurve, ManaFulfillment,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{DeckProfile, DeckWarning, deck_tag_label};
use zwipe_ui::deck::warning::{self, Fix};

use crate::domain::deck::total_price_line;
use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::section::CollapsibleSection;
use crate::inbound::router::Route;

/// One label and value row.
#[component]
fn InfoRow(label: String, value: String) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", "{label}" }
            span { class: "info-row-value", "{value}" }
        }
    }
}

/// The Profile card: name, card count, format, the command zone and power
/// level. The commander row prefers the fetched card's name.
#[component]
pub fn ProfileCard(profile: DeckProfile, commander: Option<Card>, cards: usize) -> Element {
    let label = if profile.format.is_some_and(|f| f.has_signature_spell()) {
        "Oathbreaker"
    } else {
        "Commander"
    };
    let commander_name = commander
        .map(|c| c.scryfall_data.name)
        .or_else(|| profile.commander_name.clone())
        .unwrap_or_else(|| "None".to_owned());
    let format = profile
        .format
        .map_or_else(|| "None".to_owned(), |f| f.display_name().to_owned());
    rsx! {
        div { class: "settings-list",
            div { class: "settings-list-header",
                span { class: "settings-list-title", "Profile" }
            }
            InfoRow { label: "Name", value: profile.name.to_string() }
            InfoRow { label: "Cards", value: cards.to_string() }
            InfoRow { label: "Format", value: format }
            if profile.format.is_some_and(|f| f.has_commander()) {
                InfoRow { label, value: commander_name }
            }
            if let Some(name) = profile.partner_commander_name.clone() {
                InfoRow { label: "Partner", value: name }
            }
            if let Some(name) = profile.background_name.clone() {
                InfoRow { label: "Background", value: name }
            }
            if let Some(name) = profile.signature_spell_name.clone() {
                InfoRow { label: "Signature spell", value: name }
            }
            if let Some(power) = profile.power_level {
                InfoRow { label: "Power level", value: power.display_name() }
            }
        }
    }
}

/// The Budget section: its currency chips (only with cards to price) and
/// the total and average price.
#[component]
pub fn BudgetSection(
    profile: DeckProfile,
    metrics: Option<DeckMetrics>,
    mut currency: Signal<PriceCurrency>,
    open_section: Signal<Option<String>>,
) -> Element {
    let shown = currency();
    let (total, average) = match (&metrics, shown) {
        (Some(m), PriceCurrency::Eur) => (m.total_price_eur, m.avg_price_eur),
        (Some(m), PriceCurrency::Tix) => (m.total_price_tix, m.avg_price_tix),
        (Some(m), PriceCurrency::Usd) => (m.total_price_usd, m.avg_price_usd),
        (None, _) => (None, None),
    };
    let has_metrics = metrics.is_some();
    let accessory = has_metrics.then(|| {
        rsx! {
            div { class: "chip-row", style: "margin-bottom:0;",
                for each in PriceCurrency::all().iter().copied() {
                    div {
                        key: "{each.label()}",
                        class: if shown == each { "chip selected" } else { "chip" },
                        onclick: move |_| currency.set(each),
                        "{each.label()}"
                    }
                }
            }
        }
    });
    rsx! {
        CollapsibleSection { title: "Budget", open_section, header_accessory: accessory,
            div { style: "display:flex;flex-direction:column;",
                InfoRow { label: "Total price", value: total_price_line(&profile, total, shown) }
                if has_metrics {
                    InfoRow {
                        label: "Average card price",
                        value: average.map_or_else(|| "N/A".to_owned(), |v| shown.format_amount(v)),
                    }
                }
            }
        }
    }
}

/// One stacked tag row: its label (with a `?` when it has one) over chips.
#[component]
fn TagRow(label: String, topic: Option<HintTopic>, chips: Vec<String>) -> Element {
    rsx! {
        div { class: "settings-row info-row-stacked",
            span { style: "display:flex;align-items:center;gap:0.35rem;",
                span { class: "settings-row-label", "{label}" }
                if let Some(topic) = topic {
                    InfoButton { topic }
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

/// The Tags section: deck, oracle and other tags, each row only when it has
/// any.
#[component]
pub fn TagsSection(profile: DeckProfile, open_section: Signal<Option<String>>) -> Element {
    let count = crate::domain::deck::tag_count(&profile);
    rsx! {
        CollapsibleSection { title: "Tags", badge: count.to_string(), open_section,
            div { style: "display:flex;flex-direction:column;",
                if !profile.tags.is_empty() {
                    TagRow {
                        label: "Deck tags",
                        topic: HintTopic::DeckTags,
                        chips: profile.tags.iter().map(|t| deck_tag_label(t)).collect::<Vec<_>>(),
                    }
                }
                if !profile.oracle_tags.is_empty() {
                    TagRow {
                        label: "Oracle tags",
                        topic: HintTopic::OracleTags,
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
}

/// Distributions, Mana and Draw odds. Distributions and Draw odds need a
/// mainboard card; Mana shows with one or with a land target.
#[component]
pub fn StatsSections(
    metrics: Option<DeckMetrics>,
    land_target: Option<i32>,
    open_section: Signal<Option<String>>,
) -> Element {
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let charts = metrics
        .as_ref()
        .filter(|m| m.land_count > 0 || m.nonland_count > 0)
        .cloned();
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    let lands = match land_target {
        Some(target) => format!("{land_count} / {target}"),
        None => land_count.to_string(),
    };
    let average_pt = charts.as_ref().and_then(|m| {
        (m.avg_power.is_some() || m.avg_toughness.is_some()).then(|| {
            let one = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
            format!("{} / {}", one(m.avg_power), one(m.avg_toughness))
        })
    });
    rsx! {
        if let Some(m) = charts.clone() {
            CollapsibleSection { title: "Distributions", open_section,
                if let Some(value) = average_pt {
                    div { class: "settings-row",
                        span { class: "settings-row-label", "Average P/T" }
                        span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
                    }
                }
                DeckCharts {
                    type_bars: m.type_bars(),
                    category_bars: m.card_role_bars(),
                    role_tags: m.card_role_tags.clone(),
                    describe_tag,
                    on_examples,
                    type_hint: rsx! { InfoButton { topic: HintTopic::TypeChart } },
                    role_hint: rsx! { InfoButton { topic: HintTopic::RoleChart } },
                }
            }
        }
        if charts.is_some() || land_target.is_some() {
            CollapsibleSection { title: "Mana", open_section,
                InfoRow { label: "Lands", value: lands }
                if let Some(m) = charts.clone() {
                    InfoRow { label: "Average mana value", value: format!("{:.1}", m.avg_cmc) }
                    ManaCurve {
                        mana_curve_bars: m.mana_curve_bars(),
                        hint: rsx! { InfoButton { topic: HintTopic::ManaCurve } },
                    }
                    ColorChart {
                        color_bars: m.color_bars(),
                        hint: rsx! { InfoButton { topic: HintTopic::ColorChart } },
                    }
                    ManaFulfillment {
                        rows: m.mana_balance_rows(),
                        hint: rsx! { InfoButton { topic: HintTopic::ManaFulfillment } },
                    }
                }
            }
        }
        if let Some(m) = charts {
            CollapsibleSection {
                title: "Draw odds",
                open_section,
                header_accessory: rsx! { InfoButton { topic: HintTopic::DrawOdds } },
                DrawOddsStepper { metrics: m }
            }
        }
    }
}

/// The draw odds with ← and → stepping the turn while one of its `-` and `+`
/// buttons has focus.
#[component]
fn DrawOddsStepper(metrics: DeckMetrics) -> Element {
    let (deck_size, buckets) = metrics.draw_odds_buckets();
    rsx! {
        div {
            id: "web-draw-odds",
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

/// What a warning's fix buttons do, handed in by the screen.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarningFixes {
    /// Sets a card's quantity.
    pub fix_quantity: Callback<(Uuid, i32)>,
    /// Clears the commander.
    pub clear_commander: Callback<()>,
    /// Takes a card out of the deck.
    pub remove: Callback<Uuid>,
}

/// The Warnings section: each warning's message and, on the right, its fix.
#[component]
pub fn WarningsSection(
    deck_id: Uuid,
    warnings: Vec<DeckWarning>,
    fixes: WarningFixes,
    open_section: Signal<Option<String>>,
) -> Element {
    let navigator = use_navigator();
    let count = warnings.len();
    rsx! {
        CollapsibleSection { title: "Warnings", warn: true, badge: count.to_string(), open_section,
            div { style: "display:flex;flex-direction:column;",
                for (i, item) in warnings.into_iter().enumerate() {
                    div { key: "{i}", class: "settings-row", style: "justify-content: space-between; gap: 0.5rem;",
                        span { class: "text-muted", "{item}" }
                        if let Some(fix) = warning::fix(&item) {
                            span { style: "display: flex; gap: 0.5rem;",
                                for (j, label) in warning::labels(fix).into_iter().enumerate() {
                                    Button {
                                        key: "{j}",
                                        variant: ButtonVariant::Small,
                                        style: "color: var(--color-warning); border-color: var(--border-warning); margin-bottom: 0;",
                                        onclick: move |_| match fix {
                                            Fix::BelowMinimum if j == 0 => {
                                                navigator.push(Route::AddDeckCard { deck_id });
                                            }
                                            Fix::BelowMinimum => {
                                                navigator.push(Route::ImportDeck { deck_id });
                                            }
                                            Fix::Quantity { card, quantity } => fixes.fix_quantity.call((card, quantity)),
                                            Fix::ClearCommander => fixes.clear_commander.call(()),
                                            Fix::Remove { card } => fixes.remove.call(card),
                                        },
                                        "{label}"
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }
    }
}
