//! About: the web build's version, its links and credits, then the full
//! release history, at the app's shape. On a wide window the rows and
//! Credits sit side by side under the page head, and the Changelog panel
//! and the release panels run the page's width under them.
//!
//! The top panel holds the Version and Privacy policy rows; the app's
//! Website row has no place on the website itself. Under it
//! a Credits panel holds the Scryfall credit and the Fan Content notice from
//! `zwipe_core::legal`, then the shared [`components::Changelog`] that
//! zwipe.net and the app draw too: its panel with the chips, each release a
//! panel of its own under it. It draws the compiled copy from the first
//! render and the root cache's server copy once the visit's fetch brings a
//! different one; a failed fetch leaves the compiled copy.
//!
//! What the screen knows and does is `zwipe_ui::screens::about`; this file
//! draws its state and performs its effects.

use components::{Button, ButtonVariant, Changelog as ChangelogView, ChangelogLook};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::changelog::ChangelogCache;
use zwipe_ui::screens::about::{self, Action, Destination, Effect, Row, State};
use zwipe_ui::telemetry::vocabulary::Screen;

use crate::config::APP_VERSION;
use crate::inbound::chrome::hint_topic::{HintTopic, InfoButton};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;
use crate::outbound::browser;

/// How many releases the Changelog shows before its "Show more" button, and
/// how many more each click reveals.
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
        let effects = state.update(Action::Load(cache.peek().clone()));
        debug_assert!(effects.is_empty(), "About's Load returns no effects");
        state
    });
    let driver = Driver {
        model,
        navigator: use_navigator(),
    };
    // The visit's fetch can land while the screen is up.
    use_effect(move || driver.dispatch(Action::Load(cache())));

    let data = model.read().changelog().clone();

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
                    ChangelogView { data, look: ChangelogLook::App, page_size: CHANGELOG_PAGE_SIZE }
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
