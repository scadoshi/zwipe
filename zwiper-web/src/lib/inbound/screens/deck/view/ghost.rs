//! The deck page while the deck loads, in the loaded page's own elements:
//! on a wide window the head, the controls, the bands, the folded stats and
//! the foot; on the phone the Profile list, Budget, the shut sections and
//! the foot. Nothing in them works.

use dioxus::prelude::*;

use crate::inbound::cards::buy_links::GhostBuyLink;
use crate::inbound::chrome::ghost::{
    GhostActionRow, GhostChip, GhostChipRow, GhostFold, GhostIconButton, GhostInfoRow, GhostList,
    GhostSection, GhostText,
};

/// Ghost rows in each band, the Commander band's one then a full group.
const BAND_ROWS: [usize; 2] = [1, 5];

/// The deck's icons: edit, clone, share and, on a wide window, the `?`.
#[component]
fn GhostActions(count: usize) -> Element {
    rsx! {
        div { class: "deck-profile-actions",
            for i in 0..count {
                GhostIconButton { key: "{i}" }
            }
        }
    }
}

/// The head: the deck name's bar over the facts as tinted chips, then one
/// featured card with its name and role chips. The icons sit in the dock.
#[component]
pub fn DeckHeadGhost() -> Element {
    rsx! {
        div { class: "deck-head",
            div { class: "deck-head-text",
                div { class: "deck-head-title-row",
                    div { class: "deck-head-title", GhostText { chars: 12 } }
                }
                div { class: "deck-head-meta",
                    for (i, (tone, chars)) in [("p5", 8), ("p3", 9), ("p3", 15), ("money", 7)].into_iter().enumerate() {
                        span { key: "{i}", class: "stat-chip stat-chip-{tone} web-ghost-chip", GhostText { chars } }
                    }
                }
            }
            div { class: "deck-head-featured",
                div { class: "deck-head-card",
                    div { class: "deck-head-card-image",
                        div { class: "skeleton-bar web-ghost-card" }
                    }
                    div { class: "deck-head-card-chips",
                        span { class: "stat-chip deck-head-card-name web-ghost-chip", GhostText { chars: 15 } }
                        span { class: "stat-chip deck-head-card-role-zone web-ghost-chip", GhostText { chars: 9 } }
                    }
                }
            }
        }
    }
}

/// The controls: Quick add, Group by, Boards, then Show with the fold-all
/// button.
#[component]
pub fn DeckControlsGhost() -> Element {
    rsx! {
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label web-ghost-chip", GhostText { chars: 9 } }
            div {
                class: "input-clear-wrap web-ghost-field",
                div {
                    class: "input input-compact input-placeholder",
                    style: "line-height: normal;",
                    GhostText { chars: 20 }
                }
            }
        }
        GhostChipRow { label: 8, chips: vec![4, 10, 5, 9] }
        GhostChipRow { label: 6, chips: vec![4, 5, 4] }
        GhostChipRow { label: 4, chips: vec![12, 3], trailing: rsx! { GhostIconButton {} } }
    }
}

/// The bands: a group's fold with its card rows.
#[component]
pub fn DeckBandsGhost() -> Element {
    rsx! {
        for (i, rows) in BAND_ROWS.into_iter().enumerate() {
            GhostFold { key: "{i}", chars: 12, hint: false,
                div { class: "deck-band-rows",
                    for r in 0..rows {
                        div { key: "{r}", class: "skeleton-card-row web-ghost-card-row",
                            div { class: "skeleton-bar skeleton-card-thumb" }
                            div { class: "skeleton-bar skeleton-card-bar-row" }
                        }
                    }
                }
            }
        }
    }
}

/// The currency chips in Budget's header.
#[component]
fn CurrencyGhost() -> Element {
    rsx! {
        div { class: "chip-row", style: "margin-bottom:0;",
            for i in 0..3 {
                GhostChip { key: "{i}", chars: 3 }
            }
        }
    }
}

/// The stats, shut: Budget with its currency chips, Distributions, Mana
/// and Draw odds with its `?`.
#[component]
pub fn DeckStatsGhost() -> Element {
    rsx! {
        GhostFold { chars: 6, folded: true, accessory: rsx! { CurrencyGhost {} } }
        for (i, chars) in [13u8, 4, 9].into_iter().enumerate() {
            GhostFold { key: "{i}", chars, hint: i == 2, folded: true }
        }
    }
}

/// The Data list: Import and Export, a button each.
#[component]
fn DataGhost() -> Element {
    rsx! {
        GhostList { chars: 4,
            GhostActionRow { label: 12, button: 6 }
            GhostActionRow { label: 12, button: 6 }
        }
    }
}

/// The Danger zone list: Clear skips and Delete deck, a button each.
#[component]
fn DangerGhost() -> Element {
    rsx! {
        GhostList { chars: 11, class: "danger-zone",
            GhostActionRow { label: 11, button: 5 }
            GhostActionRow { label: 11, button: 6 }
        }
    }
}

/// The wide page's foot: Data in the left column, the Danger zone beside it.
#[component]
pub fn DeckFootGhost() -> Element {
    rsx! {
        div { class: "web-column", DataGhost {} }
        DangerGhost {}
    }
}

/// The phone's stats while the deck's cards load: Distributions, Mana and
/// Draw odds, shut.
#[component]
pub fn StatsSectionsGhost() -> Element {
    rsx! {
        for (i, chars) in [13u8, 4, 9].into_iter().enumerate() {
            GhostSection { key: "{i}", chars, open: false, hint: false }
        }
    }
}

/// The phone's page while the deck loads: Profile with its icons and five
/// rows (name, cards, format, commander, power level), Budget open with its
/// currency chips and Buy row, the shut stats, then Data and the Danger
/// zone.
#[component]
pub fn DeckViewGhost() -> Element {
    rsx! {
        div { class: "content-enter web-deck-view",
            GhostList { chars: 7, header: rsx! { GhostActions { count: 3 } },
                GhostInfoRow { label: 4, value: 11 }
                GhostInfoRow { label: 5, value: 2 }
                GhostInfoRow { label: 6, value: 9 }
                GhostInfoRow { label: 9, value: 15 }
                GhostInfoRow { label: 11, value: 13 }
            }
            GhostSection {
                chars: 6,
                open: true,
                accessory: rsx! { CurrencyGhost {} },
                div { style: "display:flex;flex-direction:column;",
                    GhostInfoRow { label: 11, value: 7 }
                    GhostInfoRow { label: 18, value: 6 }
                    div { class: "deck-buy-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label web-ghost-chip", GhostText { chars: 3 } }
                        GhostBuyLink { label: "TCGplayer", button: "btn-action" }
                        GhostBuyLink { label: "Card Kingdom", button: "btn-action" }
                    }
                }
            }
            StatsSectionsGhost {}
            DataGhost {}
            DangerGhost {}
        }
    }
}
