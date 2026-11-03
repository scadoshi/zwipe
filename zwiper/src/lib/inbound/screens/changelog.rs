//! Changelog screen.
//!
//! Wraps the shared [`components::Changelog`] (the same release history
//! shown on the website) in the app's screen chrome. Reached from Account.
//!
//! The changelog is fetched once in the background at startup and cached for
//! the session (see [`crate::inbound::components::auth::session_upkeep`]).
//! This screen reads that cache: a skeleton while the fetch is in flight, the
//! fetched copy once it lands, and a note when it failed.
//!
//! There is deliberately no compiled fallback. Reaching this screen means
//! opening an app whose every other surface needs the same server, and the
//! compiled copy is by definition the one that shipped with this build, so it
//! could never show anything the reader has not already got.
//!
//! What the screen knows and does is `zwipe_ui::screens::changelog`; this
//! file draws its state and performs its effects. The chips themselves are
//! drawn and pressed inside [`components::Changelog`].

use components::Changelog as ChangelogContent;
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_ui::screens::changelog::{self as model, Action, Destination, Effect, State, Status};

use crate::inbound::components::auth::session_upkeep::ChangelogCache;
use crate::inbound::components::screen_header::ScreenHeader;

/// The Changelog's model with the handles that perform its effects.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            match effect {
                Effect::Navigate(Destination::Previous) => self.navigator.go_back(),
            }
        }
    }
}

/// Full release history, reachable from the Account screen.
#[component]
pub fn Changelog() -> Element {
    let cache: Signal<ChangelogCache> = use_context();
    // Loads inside the initializer so the first render already shows the
    // cache as it stands.
    let model = use_signal(|| {
        let mut state = State::new();
        let _ = state.update(Action::Load(cache.peek().clone()));
        state
    });
    let driver = Driver {
        model,
        navigator: use_navigator(),
    };
    // The launch fetch can land while the screen is up.
    use_effect(move || driver.dispatch(Action::Load(cache())));

    let (status, data) = {
        let screen = model.read();
        (screen.status(), screen.changelog().cloned())
    };

    rsx! {
        div { class: "screen",
            ScreenHeader { title: model::TITLE, back: move |()| driver.dispatch(Action::Back) }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    match (status, data) {
                        (Status::Loaded, Some(data)) => rsx! { ChangelogContent { data } },
                        (Status::Failed, _) => rsx! {
                            span { class: "chip-note", {model::FAILED} }
                        },
                        (Status::Loading | Status::Loaded, _) => rsx! { ChangelogSkeleton {} },
                    }
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
