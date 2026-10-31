//! Changelog: the chip rows as a sticky filter column beside the release
//! cards on a wide window, above them below the breakpoint.
//!
//! The history comes from the root's cache, fetched once at the start of the
//! visit: a skeleton while it is out, a note if it failed, with no retry and
//! no compiled fallback.

use components::{ActionBar, Button, ButtonVariant, Changelog as ChangelogContent};
use dioxus::prelude::*;

use crate::inbound::caches::{ChangelogCache, Load};
use crate::inbound::shell::ScreenFrame;

/// Bullet bars per skeleton card, roughly a real release's.
const SKELETON_BULLETS: [usize; 3] = [5, 3, 4];

/// Skeleton chips standing in for the filter row.
const SKELETON_CHIPS: usize = 8;

/// The release history, from Account. Starts on `Latest` on every visit.
#[component]
pub fn Changelog() -> Element {
    let navigator = use_navigator();
    let ChangelogCache(cache) = use_context();

    rsx! {
        ScreenFrame { title: "Changelog",
            div { class: "screen-content content-enter",
                div { class: "changelog-screen web-changelog",
                    match cache() {
                        Load::Loading => rsx! { ChangelogSkeleton {} },
                        Load::Loaded(data) => rsx! { ChangelogContent { data } },
                        Load::Failed => rsx! {
                            span { class: "chip-note", "Could not load the changelog" }
                        },
                    }
                }
            }
            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }
        }
    }
}

/// The app's ghost of the chips and three release cards, all bars.
#[component]
fn ChangelogSkeleton() -> Element {
    rsx! {
        div { class: "changelog-skeleton-chips",
            for i in 0..SKELETON_CHIPS {
                div { key: "{i}", class: "skeleton-bar changelog-skeleton-chip" }
            }
        }
        div { class: "changelog-list",
            for (c , bullets) in SKELETON_BULLETS.into_iter().enumerate() {
                div { key: "{c}", class: "changelog-card",
                    div { class: "changelog-skeleton-row",
                        div { class: "skeleton-bar changelog-skeleton-version" }
                        div { class: "skeleton-bar changelog-skeleton-date" }
                    }
                    div { class: "changelog-skeleton-bullets",
                        for b in 0..bullets {
                            div { key: "{b}", class: "skeleton-bar changelog-skeleton-bullet" }
                        }
                    }
                }
            }
        }
    }
}
