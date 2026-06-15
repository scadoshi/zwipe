use dioxus::prelude::*;

/// One bordered "info-list"-shaped skeleton with `rows` row placeholders inside.
#[component]
fn SkeletonInfoList(rows: usize) -> Element {
    rsx! {
        div { class: "skeleton-info-list",
            for i in 0..rows {
                div { key: "{i}", class: "skeleton-info-row",
                    div { class: "skeleton-bar skeleton-info-row-label" }
                    div { class: "skeleton-bar skeleton-info-row-value" }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DeckStatsSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-stats",
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-section-label" }
                div { class: "skeleton-chip-row",
                    div { class: "skeleton-chip skeleton-chip-active" }
                    div { class: "skeleton-chip" }
                    div { class: "skeleton-chip" }
                    div { class: "skeleton-chip" }
                }
                SkeletonInfoList { rows: 5 }
            }
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-section-label" }
                div { class: "skeleton-block skeleton-block-chart" }
            }
            div { class: "skeleton-stats-section",
                div { class: "skeleton-bar skeleton-section-label" }
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
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header" }
                }
                div { class: "skeleton-card-row",
                    div { class: "skeleton-bar skeleton-card-bar-row" }
                }
            }
            div { class: "skeleton-card-group",
                div { class: "skeleton-card-group-header",
                    div { class: "skeleton-bar skeleton-card-bar-header skeleton-card-bar-header-wide" }
                }
                for i in 0..6 {
                    div { key: "{i}", class: "skeleton-card-row",
                        div { class: "skeleton-bar skeleton-card-bar-row" }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn EditDeckSkeleton(#[props(default = 3)] fields: usize) -> Element {
    rsx! {
        div { class: "skeleton-edit-deck",
            for i in 0..fields {
                div { key: "{i}", class: "skeleton-edit-deck-field",
                    div { class: "skeleton-bar skeleton-edit-deck-label" }
                    div { class: "skeleton-edit-deck-input",
                        div { class: "skeleton-bar skeleton-edit-deck-value" }
                    }
                }
            }
        }
    }
}

#[component]
pub(crate) fn DeckProfileSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-profile",
            div { class: "skeleton-bar skeleton-section-label" }
            SkeletonInfoList { rows: 3 }
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
                    div { class: "skeleton-bar skeleton-deck-list-meta skeleton-deck-list-meta-2" }
                }
            }
        }
    }
}
