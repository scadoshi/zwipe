//! About screen: the app's version, its links and credits, then the full
//! release history.
//!
//! The top panel holds the rows Account used to carry under About (Version,
//! zwipe.net and Privacy policy), with their labels from
//! `zwipe_ui::screens::account`. Under it sit the Scryfall credit and the Fan
//! Content notice from `zwipe_core::legal`, then the shared
//! [`components::Changelog`], the same history the website shows.
//!
//! The changelog is fetched once in the background at startup and cached for
//! the session (see [`crate::inbound::components::auth::session_upkeep`]).
//! This screen reads that cache: a skeleton while the fetch is in flight, the
//! fetched copy once it lands, and a note when it failed. There is no compiled
//! fallback: the compiled copy is the one that shipped with this build, so it
//! could never show anything the reader has not already got.
//!
//! What the changelog part knows and does is `zwipe_ui::screens::changelog`;
//! this file draws its state and performs its effects.

use components::{Button, ButtonVariant, Changelog as ChangelogContent};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::domain::site::WEB_BASE;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::screens::account::{self, Row};
use zwipe_ui::screens::changelog::{self as model, Action, Destination, Effect, State, Status};

use crate::inbound::components::auth::session_upkeep::ChangelogCache;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::router::Router;
use crate::outbound::open_url;

/// App version, baked at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// The changelog part's model with the handles that perform its effects.
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

/// The About screen, in the main menu.
#[component]
pub fn About() -> Element {
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
            ScreenHeader { title: account::ABOUT, back: move |()| driver.dispatch(Action::Back) }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    div { class: "settings-list",
                        div { class: "settings-row",
                            span { style: "display:flex;align-items:center;",
                                span { class: "settings-row-label", {Row::Version.label()} }
                                InfoButton { topic: HintTopic::Version }
                            }
                            div { class: "settings-row-value",
                                span { {account::version_label(APP_VERSION)} }
                            }
                        }
                        div { class: "settings-row",
                            span { style: "display:flex;align-items:center;",
                                span { class: "settings-row-label", {Row::Website.label()} }
                                InfoButton { topic: HintTopic::Website }
                            }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| open_url::open(WEB_BASE),
                                    {account::WEBSITE_LINK}
                                }
                            }
                        }
                        div { class: "settings-row",
                            span { class: "settings-row-label", {Row::PrivacyPolicy.label()} }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| {
                                        driver.navigator.push(Router::PrivacyPolicy {});
                                    },
                                    {account::VIEW}
                                }
                            }
                        }
                    }

                    div { class: "about-credits",
                        p { dangerous_inner_html: SCRYFALL_CREDIT_HTML }
                        p { dangerous_inner_html: FAN_CONTENT_HTML }
                    }

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
