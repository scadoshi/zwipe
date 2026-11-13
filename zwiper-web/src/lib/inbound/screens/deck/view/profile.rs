//! A deck's Profile and foot, the same for the owner's deck and a shared deck. On a phone the
//! Profile is the page, as the app's: the Profile card, the accordion with `Budget` open, then
//! Warnings, Data and, for the owner, Danger zone. On a wide window the foot holds Warnings and
//! Data beside the owner's Danger zone. A shared deck passes no warning actions, so its warnings
//! read only, and its Data offers Export alone.

use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::DeckWarning;
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_ui::deck::profile::DeckInfo;
use zwipe_ui::deck::view::StatsProfile;

use super::ghost::{DeckFootGhost, DeckViewGhost, StatsSectionsGhost};
use super::sections::{
    BudgetSection, ProfileCard, StatsSections, TagsSection, WarningActions, WarningsBody,
    WarningsSection,
};

/// The Profile on a phone; see the module's doc. Until `info` is in it draws its ghost, and
/// while the deck's cards are still loading (`metrics_loading`) the stats draw theirs.
#[component]
pub(crate) fn DeckProfilePhone(
    // The Profile card's rows; `None` until the deck is in.
    info: Option<DeckInfo>,
    // The stat panels' inputs.
    stats: StatsProfile,
    metrics: Option<DeckMetrics>,
    #[props(default)] metrics_loading: bool,
    currency: Signal<PriceCurrency>,
    // The deck's warnings, and the owner's fixes for them.
    warnings: Vec<DeckWarning>,
    actions: Option<WarningActions>,
    // The Data panel, and the owner's Danger zone.
    data: Element,
    #[props(default)] danger: Option<Element>,
) -> Element {
    let open_section: Signal<Option<String>> = use_signal(|| Some("Budget".to_owned()));
    let Some(info) = info else {
        return rsx! {
            DeckViewGhost {}
        };
    };
    rsx! {
        div { class: "content-enter web-deck-view",
            ProfileCard { info }
            if stats.has_budget(metrics.is_some()) {
                BudgetSection {
                    stats: stats.clone(),
                    metrics: metrics.clone(),
                    currency,
                    open_section,
                }
            }
            if stats.tag_count() > 0 {
                TagsSection { stats: stats.clone(), open_section }
            }
            if metrics.is_none() && metrics_loading {
                StatsSectionsGhost {}
            } else {
                StatsSections { metrics, land_target: stats.land_target, open_section }
            }
            if !warnings.is_empty() {
                WarningsSection { warnings, actions, open_section }
            }
            {data}
            {danger}
        }
    }
}

/// The wide page's foot: Warnings (with the owner's fixes) and the Data panel in a column, then
/// the owner's Danger zone beside it. Until the deck is `loaded` it draws its ghost.
#[component]
pub(crate) fn DeckFoot(
    loaded: bool,
    warnings: Vec<DeckWarning>,
    actions: Option<WarningActions>,
    data: Element,
    #[props(default)] danger: Option<Element>,
) -> Element {
    if !loaded {
        return rsx! {
            DeckFootGhost {}
        };
    }
    rsx! {
        div { class: "web-column",
            if !warnings.is_empty() {
                div { class: "web-section open warn web-deck-warnings", "data-settle-key": "warnings",
                    div { class: "web-section-head",
                        div { class: "web-section-title",
                            span { class: "settings-list-title", "Warnings" }
                            span { class: "web-section-badge", "{warnings.len()}" }
                        }
                    }
                    hr { class: "web-section-rule" }
                    WarningsBody { warnings, actions }
                }
            }
            {data}
        }
        {danger}
    }
}
