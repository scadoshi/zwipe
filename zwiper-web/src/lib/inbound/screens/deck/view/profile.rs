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

/// Whether the deck has had warnings this visit: true from the first render with any, and kept
/// after a fix clears the last one.
fn use_warned(any: bool) -> bool {
    let mut warned = use_signal(|| any);
    use_effect(use_reactive!(|any| {
        if any {
            warned.set(true);
        }
    }));
    warned() || any
}

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
    // Warnings stay up for the visit once there were any, so a fix that clears the last one
    // leaves the page where it was.
    let warned = use_warned(!warnings.is_empty());
    let Some(info) = info else {
        return rsx! {
            DeckViewGhost {}
        };
    };
    rsx! {
        // The sections settle when one comes, goes or changes height.
        div { class: "content-enter web-deck-view", "data-settle-scope": "",
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
            if warned {
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
    let warned = use_warned(!warnings.is_empty());
    if !loaded {
        return rsx! {
            DeckFootGhost {}
        };
    }
    rsx! {
        div { class: "web-column",
            if warned {
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
