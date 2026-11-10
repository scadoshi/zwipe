//! About: the web build's version, its links and credits, then the full
//! release history, at the app's shape. On a wide window the rows and
//! Credits sit side by side under the page head, and the Changelog panel
//! and the release panels run the page's width under them.
//!
//! The top panel holds the Version and Privacy policy rows; the app's
//! Website row has no place on the website itself. Under it
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
    Button, ButtonVariant, ChangelogCollapseAll, ChangelogFilter, ChangelogFilters, ChangelogLook,
    ChangelogReleases, use_collapse_set,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::changelog::ChangelogCache;
use zwipe_ui::screens::about::{self, Action, Destination, Effect, Row, State, Status};
use zwipe_ui::telemetry::vocabulary::Screen;

use crate::config::APP_VERSION;
use crate::inbound::chrome::ghost::{GhostChip, GhostText, GhostWords};
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;
use crate::outbound::browser;

/// How many releases the Changelog shows before its "Show more" button, and
/// how many more each click reveals.
const CHANGELOG_PAGE_SIZE: usize = 5;

/// Bullet bars per skeleton row, roughly a real release's.
const SKELETON_BULLETS: [usize; 3] = [5, 3, 4];

/// The filter row's ghost chips, as wide as the loaded `Latest` and year.
const SKELETON_CHIPS: [u8; 2] = [6, 4];

/// A ghost bullet's length in characters, by its place in the release.
const SKELETON_BULLET_CHARS: [u16; 5] = [88, 64, 102, 57, 76];

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
                Effect::OpenUrl(url) => browser::open_here(&url),
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
    telemetry::use_telemetry_screen(Screen::Changelog);
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
    // Every release starts open; a header or Collapse all shuts them.
    let collapse = use_collapse_set();

    rsx! {
        ScreenFrame { title: about::TITLE,
            div { class: "screen-content content-enter",
                div { class: "changelog-screen web-about",
                    div { class: "web-columns",
                    div { class: "web-column",
                    div { class: "settings-list",
                        div { class: "settings-row",
                            RowLabel { label: Row::Version.label(), topic: HintTopic::Version }
                            div { class: "settings-row-value",
                                span { {about::version_label(APP_VERSION)} }
                            }
                        }
                        div { class: "settings-row web-about-privacy",
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
                    }
                    div { class: "web-column about-credits-column",
                    div { class: "settings-list about-panel",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", {about::CREDITS} }
                        }
                        div { class: "about-credits",
                            p { dangerous_inner_html: SCRYFALL_CREDIT_HTML }
                            p { dangerous_inner_html: FAN_CONTENT_HTML }
                        }
                    }
                    }
                    }
                    div { class: "settings-list about-panel",
                        div { class: "settings-list-header",
                            span { class: "settings-list-title", {about::CHANGELOG} }
                            if let (Status::Loaded, Some(data)) = (status, &data) {
                                ChangelogCollapseAll { data: data.clone(), selection, set: collapse }
                            }
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
                            ChangelogReleases { data, selection, collapse, look: ChangelogLook::App, page_size: CHANGELOG_PAGE_SIZE }
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

/// The chips' ghost while the visit's fetch is out: the loaded row's empty
/// chips in the Changelog panel.
#[component]
fn ChangelogSkeletonChips() -> Element {
    rsx! {
        div { class: "settings-row changelog-filters-row",
            div { class: "changelog-filter",
                for (i, chars) in SKELETON_CHIPS.into_iter().enumerate() {
                    GhostChip { key: "{i}", chars, button: true }
                }
            }
        }
    }
}

/// The releases' ghost while the fetch is out: a panel per release in the
/// loaded panel's elements, its header the version, the date and the
/// status tag, its body the bullets.
#[component]
fn ChangelogSkeletonPanels() -> Element {
    rsx! {
        div { class: "changelog-panels",
            for (c , bullets) in SKELETON_BULLETS.into_iter().enumerate() {
                div { key: "{c}", class: "release-card",
                    div { class: "release-card-head",
                        h2 { class: "release-card-version", GhostText { chars: 6 } }
                        span { class: "release-card-date", GhostText { chars: 11 } }
                        span { class: "status-tag web-ghost-chip release-card-tag", GhostText { chars: 6 } }
                    }
                    ul { class: "release-card-notes",
                        for chars in SKELETON_BULLET_CHARS.into_iter().take(bullets) {
                            li { key: "{chars}", GhostWords { chars } }
                        }
                    }
                }
            }
        }
    }
}
