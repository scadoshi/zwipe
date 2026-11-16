//! About: the full release history with the build's version beside its title, at every width.
//! The page footer under every web screen carries the Privacy policy link and the credits, so
//! About draws neither; the apps, which have no footer, keep both in theirs. The nav's `?`
//! explains the version.
//!
//! The release history is the shared [`components::Changelog`] that zwipe.net and the app draw
//! too: its panel with the chips, each release a panel of its own under it. It draws the
//! compiled copy from the first render and the root cache's server copy once the visit's fetch
//! brings a different one; a failed fetch leaves the compiled copy.
//!
//! What the screen knows and does is `zwipe_ui::screens::about`; this file
//! draws its state and performs its effects.

use components::Changelog as ChangelogView;
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_ui::changelog::ChangelogCache;
use zwipe_ui::screens::about::{self, Action, Destination, Effect, State};
use zwipe_ui::telemetry::vocabulary::Screen;

use crate::config::APP_VERSION;
use crate::inbound::chrome::hint_topic::HintTopic;
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
    let mut hint: Signal<Option<HintTopic>> = use_context();

    rsx! {
        ScreenFrame { title: about::TITLE, on_hint: move |()| hint.set(Some(HintTopic::Version)),
            div { class: "screen-content content-enter",
                div { class: "changelog-screen web-about",
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
