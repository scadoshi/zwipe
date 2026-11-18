//! The deck view's panels: the Profile card with Edit, Clone and Share in
//! its header, and the Budget (with its Buy row), Tags, Distributions,
//! Mana, Draw odds and Warnings sections. Each section is a body the wide
//! window's deck page folds in its own panel, and a `*Section` wrapper
//! that draws the body in the phone's accordion.

use components::{
    BudgetBody, Button, ButtonVariant, Chip, CurrencyChips, DistributionsBody, DrawOddsStepper,
    InfoRow, ManaBody, TagsBody, charted,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::DeckWarning;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::profile::DeckInfo;
use zwipe_ui::deck::view::StatsProfile;
use zwipe_ui::deck::warning::{self, Fix};
use zwipe_ui::screens::deck_view;

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

/// The warning color the warnings' buttons take.
const WARNING_BUTTON_STYLE: &str = "color: var(--ink-color-warning, var(--color-warning)); border-color: var(--border-warning); margin-bottom: 0;";

/// Warnings' Unpriced entry: the deck's cards with no price in the shown currency, and what a
/// tap on one of them does (its details open, to pick a priced printing).
#[derive(Debug, Clone, PartialEq)]
pub struct UnpricedEntry {
    /// The cards, from [`zwipe_ui::deck::warning::unpriced`].
    pub cards: Vec<Card>,
    /// The currency they have no price in.
    pub currency: PriceCurrency,
    /// Opens a card's details by its printing ID.
    pub open: Callback<Uuid>,
}

/// How many entries Warnings holds: the warnings, and the Unpriced entry while it has a card.
#[must_use]
pub fn warnings_count(warnings: &[DeckWarning], unpriced: Option<&UnpricedEntry>) -> usize {
    let unpriced = unpriced.is_some_and(|u| !u.cards.is_empty());
    warnings.len().saturating_add(usize::from(unpriced))
}

/// The Warnings section: [`WarningsBody`] in the phone's accordion.
#[component]
pub fn WarningsSection(
    warnings: Vec<DeckWarning>,
    actions: Option<WarningActions>,
    #[props(default)] unpriced: Option<UnpricedEntry>,
    open_section: Signal<Option<String>>,
) -> Element {
    let count = warnings_count(&warnings, unpriced.as_ref());
    rsx! {
        CollapsibleSection { title: "Warnings", warn: true, badge: count.to_string(), open_section,
            WarningsBody { warnings, actions, unpriced }
        }
    }
}

/// Warnings' body: the Unpriced entry first while the deck has an unpriced card, then each
/// warning's message and, on the right, its fix where the owner's `actions` are given. Once a fix
/// has cleared the last one it says so, so the panel stays where it was.
#[component]
pub fn WarningsBody(
    warnings: Vec<DeckWarning>,
    actions: Option<WarningActions>,
    #[props(default)] unpriced: Option<UnpricedEntry>,
) -> Element {
    let navigator = use_navigator();
    let unpriced = unpriced.filter(|u| !u.cards.is_empty());
    let none_left = warnings.is_empty() && unpriced.is_none();
    rsx! {
        div { style: "display:flex;flex-direction:column;",
                if let Some(entry) = unpriced {
                    UnpricedRow { entry }
                }
                if none_left {
                    div { class: "settings-row deck-warning-row",
                        span { class: "text-muted", "{deck_view::NO_WARNINGS_LEFT}" }
                    }
                }
                for (i, item) in warnings.into_iter().enumerate() {
                    div { key: "{i}", class: "settings-row deck-warning-row",
                        span { class: "text-muted", "{item}" }
                        if let (Some(fix), Some(WarningActions { deck_id, fixes })) = (warning::fix(&item), actions) {
                            span { class: "deck-warning-fixes",
                                for (j, label) in warning::labels(fix).into_iter().enumerate() {
                                    Button {
                                        key: "{j}",
                                        variant: ButtonVariant::Small,
                                        style: WARNING_BUTTON_STYLE,
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

/// The Unpriced entry: a chip that says how many cards have no price and, selected, lays them out
/// as chips under it in its row, as a role's tag chips reveal theirs. It starts unselected.
#[component]
fn UnpricedRow(entry: UnpricedEntry) -> Element {
    let mut shown = use_signal(|| false);
    let UnpricedEntry {
        cards,
        currency,
        open,
    } = entry;
    let line = warning::unpriced_line(cards.len(), currency);
    rsx! {
        // One row: the chips wrap onto their own line under the count, inside it.
        div { class: "settings-row deck-warning-row",
            Chip { selected: shown(), onclick: move |_| shown.toggle(), "{line}" }
            // The tag chips' reveal, so the cards ease open and shut; shut, they take no focus.
            div {
                class: if shown() { "keyword-reveal open deck-unpriced-fold" } else { "keyword-reveal deck-unpriced-fold" },
                inert: (!shown()).then_some("true"),
                div { class: "keyword-reveal-inner",
                    div { class: "chip-row deck-unpriced-chips",
                        for card in cards {
                            Chip {
                                key: "{card.scryfall_data.id}",
                                selected: false,
                                onclick: move |_| open.call(card.scryfall_data.id),
                                "{card.scryfall_data.name}"
                            }
                        }
                    }
                }
            }
        }
    }
}
