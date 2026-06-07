use dioxus::prelude::*;

#[component]
pub(crate) fn DeckStatsSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-stats",
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-bar-label" }
                div { class: "skeleton-bar skeleton-bar-row" }
                div { class: "skeleton-bar skeleton-bar-row" }
                div { class: "skeleton-bar skeleton-bar-row" }
                div { class: "skeleton-bar skeleton-bar-row" }
            }
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-bar-label" }
                div { class: "skeleton-block skeleton-block-chart" }
            }
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-bar-label" }
                div { class: "skeleton-block skeleton-block-chart" }
            }
        }
    }
}

#[component]
pub(crate) fn DeckCardListSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-card-list",
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header" }
                div { class: "skeleton-card-row" }
            }
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header skeleton-card-group-header-wide" }
                for i in 0..6 {
                    div { key: "{i}", class: "skeleton-card-row" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DeckListSkeleton(#[props(default = 5)] rows: usize) -> Element {
    rsx! {
        div { class: "skeleton-deck-list",
            for i in 0..rows {
                div { key: "{i}", class: "skeleton-deck-list-item",
                    div { class: "skeleton-bar skeleton-deck-list-title" }
                    div { class: "skeleton-bar skeleton-deck-list-meta" }
                }
            }
        }
    }
}
