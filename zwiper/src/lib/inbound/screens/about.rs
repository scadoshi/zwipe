//! About screen: the app's links and credits, then the full release history
//! with the app's version beside its title.
//!
//! The top panel holds the Website and Privacy policy rows; the header's `?`
//! explains the version. Under it
//! a Credits panel holds the Scryfall credit and the Fan Content notice from
//! `zwipe_core::legal`, then the shared [`components::Changelog`] that
//! zwipe.net and the web app draw too: its panel with the chips, then a
//! release per panel, five at a time behind "Show more".
//!
//! It draws the session's cache (see
//! [`crate::inbound::components::auth::session_upkeep`]): the compiled copy
//! from the first render, and the server's once the startup fetch brings a
//! different one, so entries added after this build show without a store
//! update. A failed fetch leaves the compiled copy.
//!
//! What the screen knows and does is `zwipe_ui::screens::about`; this file
//! draws its state and performs its effects.

use components::{Button, ButtonVariant, Changelog as ChangelogView};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_core::legal::{FAN_CONTENT_HTML, SCRYFALL_CREDIT_HTML};
use zwipe_ui::screens::about::{self, Action, Destination, Effect, Row, State};

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
        let effects = state.update(Action::Load(cache.peek().clone()));
        debug_assert!(effects.is_empty(), "About's Load returns no effects");
        state
    });
    let driver = Driver {
        model,
        navigator: use_navigator(),
    };
    // The launch fetch can land while the screen is up.
    use_effect(move || driver.dispatch(Action::Load(cache())));

    let data = model.read().changelog().clone();
    // The header's `?` opens the Version hint the dialog host draws.
    let mut hint_open = use_signal(|| false);
    let mut hint: Signal<Option<HintTopic>> = use_context();
    use_effect(move || {
        if hint_open() {
            hint_open.set(false);
            hint.set(Some(HintTopic::Version));
        }
    });

    rsx! {
        div { class: "screen",
            ScreenHeader { title: about::TITLE, hint: hint_open, back: move |()| driver.dispatch(Action::Back) }

            div { class: "screen-content content-enter",
                div { class: "changelog-screen",
                    div { class: "settings-list",
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

                    ChangelogView {
                        data,
                        page_size: CHANGELOG_PAGE_SIZE,
                        version: about::version_label(APP_VERSION),
                    }
                }
            }
        }
    }
}
