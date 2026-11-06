//! The deck view's panels: the Profile card with Edit, Clone and Share in
//! its header, and the Budget (with its Buy row), Tags, Distributions,
//! Mana, Draw odds and Warnings sections. Each section is a body the wide
//! window's deck page folds in its own panel, and a `*Section` wrapper
//! that draws the body in the phone's accordion.

use components::icons::{CloneIcon, EditIcon, ShareIcon};
use components::{
    Button, ButtonVariant, ColorChart, DeckCharts, DrawOdds, InfoButton as HintButton, ManaCurve,
    ManaFulfillment,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::{DeckProfile, DeckWarning, deck_tag_label};
use zwipe_ui::colors::ColorCounts;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::warning::{self, Fix};
use zwipe_ui::screens::deck_view::{BUY, EDIT, MoreItem};

use super::dialogs::DeckDialogs;
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

/// Edit, Clone and Share as icons, the actions on the deck as a whole, in
/// the Profile card's header on the phone and the deck page's head on a
/// wide window, where `on_hint` adds the page's `?` after them.
#[component]
pub fn ProfileActions(
    deck_id: Uuid,
    dialogs: DeckDialogs,
    #[props(default)] on_hint: Option<Callback<()>>,
) -> Element {
    let navigator = use_navigator();
    let DeckDialogs {
        mut clone,
        mut share,
        ..
    } = dialogs;
    rsx! {
        div { class: "deck-profile-actions",
            Button {
                variant: ButtonVariant::Action,
                class: "btn-icon",
                onclick: move |_| {
                    navigator.push(Route::EditDeck { deck_id });
                },
                EditIcon {}
                span { class: "sr-only", {EDIT} }
            }
            Button {
                variant: ButtonVariant::Action,
                class: "btn-icon",
                onclick: move |_| clone.set(true),
                CloneIcon {}
                span { class: "sr-only", {MoreItem::CloneDeck.short_label()} }
            }
            Button {
                variant: ButtonVariant::Action,
                class: "btn-icon",
                onclick: move |_| share.set(true),
                ShareIcon {}
                span { class: "sr-only", {MoreItem::ShareDeck.short_label()} }
            }
            if let Some(open) = on_hint {
                HintButton { onclick: move |_| open.call(()) }
            }
        }
    }
}

/// The Profile card: name, card count, format, the command zone and power
/// level, with [`ProfileActions`] in its header. The commander row prefers
/// the fetched card's name.
#[component]
pub fn ProfileCard(
    profile: DeckProfile,
    commander: Option<Card>,
    cards: usize,
    dialogs: DeckDialogs,
) -> Element {
    let deck_id = profile.id;
    let label = Slot::Commander.label(profile.format);
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
                ProfileActions { deck_id, dialogs }
            }
            InfoRow { label: "Name", value: profile.name.to_string() }
            InfoRow { label: "Cards", value: cards.to_string() }
            InfoRow { label: "Format", value: format }
            if profile.format.is_some_and(|f| f.has_commander()) {
                InfoRow { label, value: commander_name }
            }
            if let Some(name) = profile.partner_commander_name.clone() {
                InfoRow { label: Slot::Partner.label(profile.format), value: name }
            }
            if let Some(name) = profile.background_name.clone() {
                InfoRow { label: Slot::Background.label(profile.format), value: name }
            }
            if let Some(name) = profile.signature_spell_name.clone() {
                InfoRow { label: Slot::SignatureSpell.label(profile.format), value: name }
            }
            if let Some(power) = profile.power_level {
                InfoRow { label: "Power level", value: power.display_name() }
            }
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
                div {
                    key: "{each.label()}",
                    class: if shown == each { "chip selected" } else { "chip" },
                    onclick: move |_| currency.set(each),
                    "{each.label()}"
                }
            }
        }
    }
}

/// Budget's body: the total and average price, and the Buy row with the
/// `TCGplayer` and `Card Kingdom` links once `buy` holds them.
#[component]
pub fn BudgetBody(
    profile: DeckProfile,
    metrics: Option<DeckMetrics>,
    buy: Option<(String, String)>,
    currency: Signal<PriceCurrency>,
) -> Element {
    let shown = currency();
    let (total, average) = match (&metrics, shown) {
        (Some(m), PriceCurrency::Eur) => (m.total_price_eur, m.avg_price_eur),
        (Some(m), PriceCurrency::Tix) => (m.total_price_tix, m.avg_price_tix),
        (Some(m), PriceCurrency::Usd) => (m.total_price_usd, m.avg_price_usd),
        (None, _) => (None, None),
    };
    rsx! {
        div { style: "display:flex;flex-direction:column;",
            InfoRow { label: "Total price", value: total_price_line(&profile, total, shown) }
            if metrics.is_some() {
                InfoRow {
                    label: "Average card price",
                    value: average.map_or_else(|| "N/A".to_owned(), |v| shown.format_amount(v)),
                }
            }
            if let Some((tcgplayer, cardkingdom)) = buy {
                div { class: "deck-buy-row",
                    span { class: "stat-chip stat-chip-p1 chip-row-label", {BUY} }
                    BuyLink { href: tcgplayer, label: "TCGplayer" }
                    BuyLink { href: cardkingdom, label: "Card Kingdom" }
                }
            }
        }
    }
}

/// The Budget section of the phone's accordion: [`CurrencyChips`] in its
/// header over [`BudgetBody`].
#[component]
pub fn BudgetSection(
    profile: DeckProfile,
    metrics: Option<DeckMetrics>,
    buy: Option<(String, String)>,
    currency: Signal<PriceCurrency>,
    open_section: Signal<Option<String>>,
) -> Element {
    let accessory = metrics.is_some().then(|| {
        rsx! {
            CurrencyChips { currency }
        }
    });
    rsx! {
        CollapsibleSection { title: "Budget", open_section, header_accessory: accessory,
            BudgetBody { profile, metrics, buy, currency }
        }
    }
}

/// One retailer link in the Buy row, drawn as the row's action buttons,
/// with the `↗` that marks it as leaving the site.
#[component]
fn BuyLink(href: String, label: &'static str) -> Element {
    rsx! {
        a {
            class: "btn-action web-buy-link",
            href,
            "{label}"
            span { class: "web-buy-ext", "\u{2197}" }
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

/// The Tags section of the phone's accordion, over [`TagsBody`].
#[component]
pub fn TagsSection(profile: DeckProfile, open_section: Signal<Option<String>>) -> Element {
    let count = crate::domain::deck::tag_count(&profile);
    rsx! {
        CollapsibleSection { title: "Tags", badge: count.to_string(), open_section,
            TagsBody { profile }
        }
    }
}

/// Tags' body: deck, oracle and other tags, each row only when it has any.
#[component]
pub fn TagsBody(profile: DeckProfile) -> Element {
    rsx! {
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

/// The metrics with a card to chart, if any.
#[must_use]
pub fn charted(metrics: Option<&DeckMetrics>) -> Option<DeckMetrics> {
    metrics
        .filter(|m| m.land_count > 0 || m.nonland_count > 0)
        .cloned()
}

/// Distributions' body: the average P/T and the type and role charts.
#[component]
pub fn DistributionsBody(metrics: DeckMetrics) -> Element {
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    let average_pt = (metrics.avg_power.is_some() || metrics.avg_toughness.is_some()).then(|| {
        let one = |v: Option<f64>| v.map_or_else(|| "-".to_owned(), |v| format!("{v:.1}"));
        format!(
            "{} / {}",
            one(metrics.avg_power),
            one(metrics.avg_toughness)
        )
    });
    rsx! {
        if let Some(value) = average_pt {
            div { class: "settings-row",
                span { class: "settings-row-label", "Average P/T" }
                span { class: "info-row-value", style: "white-space:nowrap;", "{value}" }
            }
        }
        DeckCharts {
            type_bars: metrics.type_bars(),
            category_bars: metrics.card_role_bars(),
            role_tags: metrics.card_role_tags.clone(),
            describe_tag,
            on_examples,
            type_hint: rsx! { InfoButton { topic: HintTopic::TypeChart } },
            role_hint: rsx! { InfoButton { topic: HintTopic::RoleChart } },
        }
    }
}

/// Mana's body: the lands against their target, and with a card to chart
/// the average mana value, the curve, the colors and the fulfillment.
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
        InfoRow { label: "Lands", value: lands }
        if let Some(m) = charts {
            InfoRow { label: "Average mana value", value: format!("{:.1}", m.avg_cmc) }
            ManaCurve {
                mana_curve_bars: m.mana_curve_bars(),
                hint: rsx! { InfoButton { topic: HintTopic::ManaCurve } },
            }
            ColorChart {
                bars: ColorCounts::from(&m).bars(),
                hint: rsx! { InfoButton { topic: HintTopic::ColorChart } },
            }
            ManaFulfillment {
                rows: m.mana_balance_rows(),
                hint: rsx! { InfoButton { topic: HintTopic::ManaFulfillment } },
            }
        }
    }
}

/// Distributions, Mana and Draw odds in the phone's accordion.
/// Distributions and Draw odds need a mainboard card; Mana shows with one
/// or with a land target.
#[component]
pub fn StatsSections(
    metrics: Option<DeckMetrics>,
    land_target: Option<i32>,
    open_section: Signal<Option<String>>,
) -> Element {
    let charts = charted(metrics.as_ref());
    let land_count = metrics.as_ref().map_or(0, |m| m.land_count);
    rsx! {
        if let Some(m) = charts.clone() {
            CollapsibleSection { title: "Distributions", open_section,
                DistributionsBody { metrics: m }
            }
        }
        if charts.is_some() || land_target.is_some() {
            CollapsibleSection { title: "Mana", open_section,
                ManaBody { charts: charts.clone(), land_count, land_target }
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
pub fn DrawOddsStepper(metrics: DeckMetrics) -> Element {
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

/// The Warnings section: [`WarningsBody`] in the phone's accordion.
#[component]
pub fn WarningsSection(
    deck_id: Uuid,
    warnings: Vec<DeckWarning>,
    fixes: WarningFixes,
    open_section: Signal<Option<String>>,
) -> Element {
    let count = warnings.len();
    rsx! {
        CollapsibleSection { title: "Warnings", warn: true, badge: count.to_string(), open_section,
            WarningsBody { deck_id, warnings, fixes }
        }
    }
}

/// Warnings' body: each warning's message and, on the right, its fix.
#[component]
pub fn WarningsBody(deck_id: Uuid, warnings: Vec<DeckWarning>, fixes: WarningFixes) -> Element {
    let navigator = use_navigator();
    rsx! {
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
