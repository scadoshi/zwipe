//! About: the web build's version, its links and credits, then the full
//! release history, at the app's shape.
//!
//! The top panel holds the Version, Website and Privacy policy rows. Under it
//! a Credits panel holds the Scryfall credit and the Fan Content notice from
//! `zwipe_core::legal`, then a Changelog panel holds the shared
//! [`components::ChangelogFilters`] as a row, with each release a panel of
//! its own under it ([`components::ChangelogReleases`]). The history comes
//! from the root's cache, fetched once at
//! the start of the visit: a skeleton while it is out, a note if it failed,
//! with no retry and no compiled fallback.
//!
//! What the screen knows and does is `zwipe_ui::screens::about`; this file
//! draws its state and performs its effects.

use components::{
    Button, ButtonVariant, ChangelogFilter, ChangelogFilters, ChangelogLook, ChangelogReleases,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::changelog::ChangelogCache;
use zwipe_ui::screens::about::{self, Action, Destination, Effect, Row, State, Status};

use crate::config::APP_VERSION;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::browser;

/// The wide page head's line under the title.
const DESCRIPTION: &str = "The version, credits and what changed in each release";

/// Bullet bars per skeleton row, roughly a real release's.
const SKELETON_BULLETS: [usize; 3] = [5, 3, 4];

/// Skeleton chips standing in for the filter row.
const SKELETON_CHIPS: usize = 8;

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
                Effect::OpenUrl(url) => browser::open_in_new_tab(&url),
                Effect::Navigate(Destination::Previous) if self.navigator.can_go_back() => {
                    self.navigator.go_back();
                }
                Effect::Navigate(Destination::Previous) => {
                    self.navigator.push(Route::Home {});
                }
                Effect::Navigate(Destination::PrivacyPolicy) => {
                    self.navigator.push(Route::PrivacyPolicy {});
                }
            }
        }
    }
}

/// The About screen, in the menu's footer.
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
    // The visit's fetch can land while the screen is up.
    use_effect(move || driver.dispatch(Action::Load(cache())));

    let (status, data) = {
        let screen = model.read();
        (screen.status(), screen.changelog().cloned())
    };
    let selection = use_signal(|| ChangelogFilter::Latest);

    rsx! {
        ScreenFrame { title: about::TITLE, description: Some(DESCRIPTION.to_owned()),
            div { class: "screen-content content-enter",
                div { class: "changelog-screen web-about",
                    div { class: "settings-list",
                        div { class: "settings-row",
                            RowLabel { label: Row::Version.label(), topic: HintTopic::Version }
                            div { class: "settings-row-value",
                                span { {about::version_label(APP_VERSION)} }
                            }
                        }
                        div { class: "settings-row",
                            RowLabel { label: Row::Website.label(), topic: HintTopic::Website }
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
                            ChangelogReleases { data, selection, look: ChangelogLook::App }
                        },
                        (Status::Failed, _) => rsx! {},
                        (Status::Loading | Status::Loaded, _) => rsx! { ChangelogSkeletonPanels {} },
                    }
                }
            }
        }
    }
}

/// A row label with its `?`.
#[component]
fn RowLabel(label: &'static str, topic: HintTopic) -> Element {
    rsx! {
        span { style: "display:flex;align-items:center;",
            span { class: "settings-row-label", {label} }
            InfoButton { topic }
        }
    }
}

/// The chips' ghost while the visit's fetch is out: one row of chip-sized
/// bars in the Changelog panel.
#[component]
fn ChangelogSkeletonChips() -> Element {
    rsx! {
        div { class: "settings-row changelog-filters-row",
            div { class: "changelog-skeleton-chips",
                for i in 0..SKELETON_CHIPS {
                    div { key: "{i}", class: "skeleton-bar changelog-skeleton-chip" }
                }
            }
        }
    }
}

/// The releases' ghost while the fetch is out: a panel per release, its
/// header a version and a date, its body the bullets.
#[component]
fn ChangelogSkeletonPanels() -> Element {
    rsx! {
        div { class: "changelog-panels",
            for (c , bullets) in SKELETON_BULLETS.into_iter().enumerate() {
                div { key: "{c}", class: "settings-list changelog-panel",
                    div { class: "settings-list-header changelog-skeleton-row",
                        div { class: "skeleton-bar changelog-skeleton-version" }
                        div { class: "skeleton-bar changelog-skeleton-date" }
                    }
                    div { class: "changelog-skeleton-bullets changelog-panel-body",
                        for b in 0..bullets {
                            div { key: "{b}", class: "skeleton-bar changelog-skeleton-bullet" }
                        }
                    }
                }
            }
        }
    }
}
