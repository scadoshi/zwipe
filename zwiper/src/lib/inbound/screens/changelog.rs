//! Changelog screen.
//!
//! Wraps the shared [`zwipe_components::Changelog`] (the same release history
//! shown on the website) in the app's screen chrome. Reached from Profile.
//!
//! The changelog is fetched once in the background at startup and cached for the
//! session (see [`crate::inbound::components::auth::session_upkeep`]). This
//! screen reads that cache: a skeleton while the fetch is still in flight, the
//! fetched copy once it lands, or the copy compiled into the binary if the fetch
//! failed, so it always renders and degrades to offline behavior.

use crate::inbound::components::{
    auth::session_upkeep::ChangelogCache, screen_header::ScreenHeader,
};
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant, Changelog as ChangelogContent};
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// Full release history, reachable from the Profile screen.
#[component]
pub fn Changelog() -> Element {
    let navigator = use_navigator();
    let cache: Signal<ChangelogCache> = use_context();

    rsx! {
        div { class: "screen",
            ScreenHeader { title: "Changelog" }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    match cache() {
                        ChangelogCache::Loading => rsx! { ChangelogSkeleton {} },
                        ChangelogCache::Loaded(data) => rsx! { ChangelogContent { data } },
                        ChangelogCache::Failed => rsx! {
                            ChangelogContent { data: HttpChangelog::current() }
                        },
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }
        }
    }
}

/// Placeholder shown while the startup changelog fetch is still in flight.
/// Mirrors the real changelog's filter chips + release-card layout.
#[component]
fn ChangelogSkeleton() -> Element {
    // Bullet counts per placeholder card, roughly matching a real release.
    const CARD_BULLETS: &[usize] = &[5, 3, 4];

    rsx! {
        div { class: "changelog-skeleton-chips",
            for i in 0..8 {
                div { key: "{i}", class: "skeleton-bar changelog-skeleton-chip" }
            }
        }
        div { class: "changelog-list",
            for (c , bullets) in CARD_BULLETS.iter().enumerate() {
                div { key: "{c}", class: "changelog-card",
                    div { class: "changelog-skeleton-row",
                        div { class: "skeleton-bar changelog-skeleton-version" }
                        div { class: "skeleton-bar changelog-skeleton-date" }
                    }
                    div { class: "changelog-skeleton-bullets",
                        for b in 0..*bullets {
                            div { key: "{b}", class: "skeleton-bar changelog-skeleton-bullet" }
                        }
                    }
                }
            }
        }
    }
}
