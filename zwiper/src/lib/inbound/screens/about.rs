//! About screen: the app's version, its links and credits, then the full
//! release history.
//!
//! The top panel holds the Version, Website and Privacy policy rows. Under it
//! a Credits panel holds the Scryfall credit and the Fan Content notice from
//! `zwipe_core::legal`, then a Changelog panel holds the shared
//! [`components::ChangelogFilters`] as a row, with each release a panel of
//! its own under it ([`components::ChangelogReleases`]), the same history the
//! website shows: the version filter first, then a release per row, five at
//! a time behind "Show more".
//!
//! The changelog is fetched once in the background at startup and cached for
//! the session (see [`crate::inbound::components::auth::session_upkeep`]).
//! This screen reads that cache: a skeleton while the fetch is in flight, the
//! fetched copy once it lands, and a note when it failed. There is no compiled
//! fallback: the compiled copy is the one that shipped with this build, so it
//! could never show anything the reader has not already got.
//!
//! What the screen knows and does is `zwipe_ui::screens::about`; this file
//! draws its state and performs its effects.

use components::{
    Button, ButtonVariant, ChangelogFilter, ChangelogFilters, ChangelogLook, ChangelogReleases,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::screens::about::{self, Action, Destination, Effect, Row, State, Status};

use crate::inbound::components::auth::session_upkeep::ChangelogCache;
use crate::inbound::components::hint_host::HintTopic;
use crate::inbound::components::info_button::InfoButton;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::router::Router;
use crate::outbound::open_url;

/// App version, baked at compile time.
const APP_VERSION: &str = env!("CARGO_PKG_VERSION");

/// How many releases the Changelog shows before its "Show more" button, and
/// how many more each tap reveals, as the website pages it.
const CHANGELOG_PAGE_SIZE: usize = 5;

/// The screen's model with the handles that perform its effects.
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
                Effect::OpenUrl(url) => open_url::open(&url),
                Effect::Navigate(Destination::Previous) => self.navigator.go_back(),
                Effect::Navigate(Destination::PrivacyPolicy) => {
                    self.navigator.push(Router::PrivacyPolicy {});
                }
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
    let selection = use_signal(|| ChangelogFilter::Latest);

    rsx! {
        div { class: "screen",
            ScreenHeader { title: about::TITLE, back: move |()| driver.dispatch(Action::Back) }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    div { class: "settings-list",
                        div { class: "settings-row",
                            span { style: "display:flex;align-items:center;",
                                span { class: "settings-row-label", {Row::Version.label()} }
                                InfoButton { topic: HintTopic::Version }
                            }
                            div { class: "settings-row-value",
                                span { {about::version_label(APP_VERSION)} }
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
                                    onclick: move |_| driver.dispatch(Action::Website),
                                    {about::WEBSITE_LINK}
                                }
                            }
                        }
                        div { class: "settings-row",
                            span { class: "settings-row-label", {Row::PrivacyPolicy.label()} }
                            div { class: "settings-row-value",
                                Button {
                                    variant: ButtonVariant::Action,
                                    onclick: move |_| driver.dispatch(Action::PrivacyPolicy),
                                    {about::VIEW}
                                }
                            }
                        }
                    }

                    div { class: "settings-list about-panel",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", {about::CREDITS} }
                        }
                        div { class: "about-credits",
                            p { dangerous_inner_html: SCRYFALL_CREDIT_HTML }
                            p { dangerous_inner_html: FAN_CONTENT_HTML }
                        }
                    }

                    div { class: "settings-list about-panel",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", {about::CHANGELOG} }
                        }
                        match (status, &data) {
                            (Status::Loaded, Some(data)) => rsx! {
                                ChangelogFilters { data: data.clone(), selection, look: ChangelogLook::App }
                            },
                            (Status::Failed, _) => rsx! {
                                div { class: "settings-row changelog-filters-row",
                                    span { class: "chip-note", {about::FAILED} }
                                }
                            },
                            (Status::Loading | Status::Loaded, _) => rsx! { ChangelogSkeletonChips {} },
                        }
                    }
                    match (status, data) {
                        (Status::Loaded, Some(data)) => rsx! {
                            ChangelogReleases { data, selection, look: ChangelogLook::App, page_size: CHANGELOG_PAGE_SIZE }
                        },
                        (Status::Failed, _) => rsx! {},
                        (Status::Loading | Status::Loaded, _) => rsx! { ChangelogSkeletonPanels {} },
                    }
                }
            }
        }
    }
}

/// The chips' placeholder while the startup changelog fetch is still in
/// flight: one row of chip-sized bars in the Changelog panel.
#[component]
fn ChangelogSkeletonChips() -> Element {
    rsx! {
        div { class: "settings-row changelog-filters-row",
            div { class: "changelog-skeleton-chips",
                for i in 0..8 {
                    div { key: "{i}", class: "skeleton-bar changelog-skeleton-chip" }
                }
            }
        }
    }
}

/// The releases' placeholder while the fetch is in flight: a panel per
/// release, its header a version and a date, its body the bullets.
#[component]
fn ChangelogSkeletonPanels() -> Element {
    // Bullet counts per placeholder panel, roughly matching a real release.
    const CARD_BULLETS: &[usize] = &[5, 3, 4];

    rsx! {
        div { class: "changelog-panels",
            for (c , bullets) in CARD_BULLETS.iter().enumerate() {
                div { key: "{c}", class: "settings-list changelog-panel",
                    div { class: "settings-list-header changelog-skeleton-row",
                        div { class: "skeleton-bar changelog-skeleton-version" }
                        div { class: "skeleton-bar changelog-skeleton-date" }
                    }
                    div { class: "changelog-skeleton-bullets changelog-panel-body",
                        for b in 0..*bullets {
                            div { key: "{b}", class: "skeleton-bar changelog-skeleton-bullet" }
                        }
                    }
                }
            }
        }
    }
}
