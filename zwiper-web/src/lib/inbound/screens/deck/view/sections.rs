//! The deck view's panels: the Profile card with Edit, Clone and Share in
//! its header, and the Budget (with its Buy row), Tags, Distributions,
//! Mana, Draw odds and Warnings sections. Each section is a body the wide
//! window's deck page folds in its own panel, and a `*Section` wrapper
//! that draws the body in the phone's accordion.

use components::{
    BudgetBody, Button, ButtonVariant, CurrencyChips, DistributionsBody, DrawOddsStepper, InfoRow,
    ManaBody, TagsBody, charted,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::DeckWarning;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::profile::DeckInfo;
use zwipe_ui::deck::view::StatsProfile;
use zwipe_ui::deck::warning::{self, Fix};

use crate::inbound::cards::otag::{OtagDescribe, OtagExamplesOpen};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::chrome::section::CollapsibleSection;
use crate::inbound::router::Route;

/// The Profile card: name, card count, format, the command zone and power
/// level, and who shares a shared deck. The deck's own actions sit in the
/// screen's dock.
#[component]
pub fn ProfileCard(info: DeckInfo) -> Element {
    let label = Slot::Commander.label(info.format);
    let commander_name = info.commander.clone().unwrap_or_else(|| "None".to_owned());
    let format = info
        .format
        .map_or_else(|| "None".to_owned(), |f| f.display_name().to_owned());
    rsx! {
        div { class: "settings-list",
            div { class: "settings-list-header",
                span { class: "settings-list-title", "Profile" }
            }
            InfoRow { label: "Name", value: info.name.clone() }
            if let Some(owner) = info.owner.clone() {
                InfoRow { label: "Shared by", value: owner }
            }
            InfoRow { label: "Cards", value: info.cards.to_string() }
            InfoRow { label: "Format", value: format }
            if info.format.is_some_and(|f| f.has_commander()) {
                InfoRow { label, value: commander_name }
            }
            if let Some(name) = info.partner.clone() {
                InfoRow { label: Slot::Partner.label(info.format), value: name }
            }
            if let Some(name) = info.background.clone() {
                InfoRow { label: Slot::Background.label(info.format), value: name }
            }
            if let Some(name) = info.signature_spell.clone() {
                InfoRow { label: Slot::SignatureSpell.label(info.format), value: name }
            }
            if let Some(power) = info.power_level {
                InfoRow { label: "Power level", value: power.display_name() }
            }
        }
    }
}

/// The Budget section of the phone's accordion: [`CurrencyChips`] in its
/// header over [`BudgetBody`].
#[component]
pub fn BudgetSection(
    stats: StatsProfile,
    metrics: Option<DeckMetrics>,
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
            BudgetBody { profile: stats, metrics, currency }
        }
    }
}

/// The Tags section of the phone's accordion, over [`TagsBody`].
#[component]
pub fn TagsSection(stats: StatsProfile, open_section: Signal<Option<String>>) -> Element {
    let count = stats.tag_count();
    rsx! {
        CollapsibleSection { title: "Tags", badge: count.to_string(), open_section,
            TagsBody { profile: stats }
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
    let describe_tag = try_use_context::<OtagDescribe>().map(|d| d.0);
    let on_examples = try_use_context::<OtagExamplesOpen>().map(|e| e.0);
    rsx! {
        if let Some(m) = charts.clone() {
            CollapsibleSection { title: "Distributions", open_section,
                DistributionsBody { metrics: m, describe_tag, on_examples }
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

/// The owner's deck and its warnings' fixes; a shared deck has none, and its
/// warnings read only.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct WarningActions {
    /// The deck the fixes change.
    pub deck_id: Uuid,
    /// What the fix buttons do.
    pub fixes: WarningFixes,
}

/// The Warnings section: [`WarningsBody`] in the phone's accordion.
#[component]
pub fn WarningsSection(
    warnings: Vec<DeckWarning>,
    actions: Option<WarningActions>,
    open_section: Signal<Option<String>>,
) -> Element {
    let count = warnings.len();
    rsx! {
        CollapsibleSection { title: "Warnings", warn: true, badge: count.to_string(), open_section,
            WarningsBody { warnings, actions }
        }
    }
}

/// Warnings' body: each warning's message and, on the right, its fix where
/// the owner's `actions` are given.
#[component]
pub fn WarningsBody(warnings: Vec<DeckWarning>, actions: Option<WarningActions>) -> Element {
    let navigator = use_navigator();
    rsx! {
        div { style: "display:flex;flex-direction:column;",
                for (i, item) in warnings.into_iter().enumerate() {
                    div { key: "{i}", class: "settings-row deck-warning-row",
                        span { class: "text-muted", "{item}" }
                        if let (Some(fix), Some(WarningActions { deck_id, fixes })) = (warning::fix(&item), actions) {
                            span { class: "deck-warning-fixes",
                                for (j, label) in warning::labels(fix).into_iter().enumerate() {
                                    Button {
                                        key: "{j}",
                                        variant: ButtonVariant::Small,
                                        style: "color: var(--ink-color-warning, var(--color-warning)); border-color: var(--border-warning); margin-bottom: 0;",
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
