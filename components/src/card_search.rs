//! The card search bar over zwipe-ui's [`model`], the same in every host: the box, the ✕ once
//! it holds text, and the matches as chips under it, a line in their place while a lookup is
//! out, matches nothing, or fails.
//!
//! It sits in one of two places ([`SearchPlace`]). On a wide screen it is the nav's box: 120px
//! at rest with the search icon and the placeholder, the icon alone where the nav leaves it
//! less room, growing over the nav's links while it has focus, with the chips in a dropdown
//! under it. On a phone it fills Home's Card search panel ([`CardSearchPanel`]), the chips in
//! the page's flow. `/` focuses whichever bar shows, unless a text field has focus or the
//! screen answers its own keys.
//!
//! A host runs the model's effects through [`use_card_search`] with its own [`SearchHost`]: its
//! clock, its lookup and its routes.

use std::future::Future;
use std::time::Duration;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::card::HttpCardLookup;
use zwipe_ui::screens::card_search::{
    self as model, Action, Destination, Effect, Failure, State, Status, Step,
};
use zwipe_ui::shortcuts;

use crate::ClearWrap;
use crate::keys::use_page_keys;

/// The search icon: a lens and its handle, Lucide's `search` drawn as one path (ISC,
/// `components/LICENSE-LUCIDE`).
pub const SEARCH_ICON: &str = "M11 3a8 8 0 1 0 0 16 8 8 0 1 0 0-16M21 21l-4.3-4.3";

/// The nav box's width while it has focus, when the nav has that much room before its links.
pub const NAV_GROWN_PX: u32 = 320;

/// The room kept between the grown box and the first link it leaves showing.
const NAV_GAP_PX: u32 = 16;

/// The least room before the first link the box grows into without covering the links.
const NAV_ROOM_MIN_PX: u32 = 240;

/// Focuses the first card search box on the page that shows: the nav's above its fold, Home's
/// panel below it.
const FOCUS_SHOWN_JS: &str = "[...document.querySelectorAll('.card-search-input')].find((el) => el.offsetParent !== null)?.focus();";

/// Answers whether any card search box shows on the page for `/` to focus, the test
/// [`FOCUS_SHOWN_JS`] focuses by.
pub(crate) const ANY_SHOWN_JS: &str = "return [...document.querySelectorAll('.card-search-input')].some((el) => el.offsetParent !== null);";

/// Where the bar sits.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SearchPlace {
    /// The wide nav's box, the chips in a dropdown under it.
    Nav,
    /// Home's Card search panel on a phone, the chips in the page's flow.
    Panel,
}

/// What a host does for the bar: waits, looks up and opens.
pub trait SearchHost: Copy + 'static {
    /// Resolves after `wait`.
    fn sleep(self, wait: Duration) -> impl Future<Output = ()>;

    /// Sends `lookup`, with the session's token when one is held.
    ///
    /// # Errors
    ///
    /// The model's [`Failure`]: rate limited with the server's wait, or any other failure.
    fn lookup(self, lookup: HttpCardLookup) -> impl Future<Output = Result<Vec<Card>, Failure>>;

    /// Opens `to`: a card page or the results page, as the host's route or link.
    fn open(self, to: Destination);
}

/// The bar's model and the host that performs its effects. Copy, so every handler and task
/// holds its own.
#[derive(Clone, Copy)]
pub struct CardSearch<H> {
    model: Signal<State>,
    host: H,
}

impl<H> std::fmt::Debug for CardSearch<H> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("CardSearch").finish_non_exhaustive()
    }
}

impl<H: SearchHost> CardSearch<H> {
    /// The model, for the bar to draw.
    #[must_use]
    pub fn model(self) -> Signal<State> {
        self.model
    }

    /// Applies `action` to the model and performs what it returns.
    pub fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    fn perform(self, effect: Effect) {
        match effect {
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    self.host.sleep(wait).await;
                    self.dispatch(Action::Debounced(ticket));
                });
            }
            Effect::Lookup { ticket, lookup } => {
                spawn(async move {
                    let reply = match self.host.lookup(lookup).await {
                        Ok(cards) => Action::Loaded { ticket, cards },
                        Err(failure) => Action::Failed { ticket, failure },
                    };
                    self.dispatch(reply);
                });
            }
            Effect::Cooldown { ticket, wait } => {
                spawn(async move {
                    self.host.sleep(wait).await;
                    self.dispatch(Action::CooledDown(ticket));
                });
            }
            Effect::Navigate(to) => self.host.open(to),
        }
    }
}

/// Puts the cursor in whichever card search box shows, the nav's or a panel's, for `/` on a
/// screen whose own keys answer before the bar's.
pub fn focus_card_search() {
    document::eval(FOCUS_SHOWN_JS);
}

/// A fresh bar driven by `host`, for one mount of the nav or the panel.
pub fn use_card_search<H: SearchHost>(host: H) -> CardSearch<H> {
    CardSearch {
        model: use_signal(State::new),
        host,
    }
}

/// The bar: the box and what shows under it. `input_id` names the box, unique on the page.
/// `on_action` takes everything the bar does, for [`CardSearch::dispatch`].
#[component]
#[expect(
    clippy::wildcard_enum_match_arm,
    reason = "keyboard_types::Key is a foreign enum of every key; only the keys handled act"
)]
pub fn CardSearchBar(
    model: Signal<State>,
    on_action: Callback<Action>,
    place: SearchPlace,
    input_id: String,
) -> Element {
    // `/` puts the cursor in whichever card search shows, the nav's box or Home's panel, while
    // no field has focus and the screen answers no keys of its own; a screen that does
    // registers after the bar, so its keys win.
    use_page_keys(Callback::new(move |key: String| {
        if key == shortcuts::CARD_SEARCH_KEY {
            focus_card_search();
        }
    }));

    let state = model.read();
    let query = state.query().to_owned();
    let status = state.status();
    let lit = state.highlight();
    let results: Vec<(Uuid, String)> = state
        .results()
        .iter()
        .map(|c| (c.scryfall_data.id, c.scryfall_data.name.clone()))
        .collect();
    drop(state);
    let line = match status {
        Status::Idle | Status::Results => None,
        Status::Loading => Some(model::SEARCHING),
        Status::NoMatch => Some(model::NO_MATCH),
        Status::RateLimited => Some(model::RATE_LIMITED),
        Status::Failed => Some(model::FAILED),
    };
    let filled = !query.is_empty();
    let blur_id = input_id.clone();
    // Opening a card from the nav lets the box go, so it shrinks back over the links.
    let opened = use_callback(move |()| {
        if place == SearchPlace::Nav {
            document::eval(&blur_js(&blur_id));
        }
    });
    let grow_id = format!("{input_id}-box");
    let box_id = grow_id.clone();

    let field = rsx! {
        ClearWrap {
            filled,
            on_clear: move |()| on_action.call(Action::Clear),
            label: "Clear the search".to_string(),
            input {
                class: "input card-search-input",
                id: "{input_id}",
                r#type: "text",
                placeholder: model::PLACEHOLDER,
                "aria-label": model::PANEL_TITLE,
                value: "{query}",
                autocapitalize: "none",
                autocorrect: "off",
                autocomplete: "off",
                spellcheck: "false",
                oninput: move |event| on_action.call(Action::Type(event.value())),
                onkeydown: move |event: Event<KeyboardData>| {
                    let action = match event.key() {
                        Key::ArrowDown => Action::Step(Step::Down),
                        Key::ArrowUp => Action::Step(Step::Up),
                        Key::Enter => Action::Enter,
                        // Escape empties a box with text and leaves the cursor in it, so the
                        // next letter starts a new search; on an empty nav box it lets the box
                        // go, so it shrinks back off the links.
                        Key::Escape => {
                            event.prevent_default();
                            event.stop_propagation();
                            if model.peek().query().is_empty() {
                                opened.call(());
                            } else {
                                on_action.call(Action::Clear);
                            }
                            return;
                        }
                        _ => return,
                    };
                    event.prevent_default();
                    let enter = action == Action::Enter;
                    on_action.call(action);
                    // An Enter that opened something emptied the box.
                    if enter && model.peek().query().is_empty() {
                        opened.call(());
                    }
                },
            }
        }
    };
    let shows_under = line.is_some() || !results.is_empty();
    let under = rsx! {
        if let Some(line) = line {
            span { class: "chip-unselected card-search-line", {line} }
            if status == Status::Failed {
                button {
                    r#type: "button",
                    class: "chip-unselected card-search-chip",
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| on_action.call(Action::Retry),
                    "Retry"
                }
            }
        } else {
            for (i, (id, name)) in results.into_iter().enumerate() {
                button {
                    key: "{id}",
                    r#type: "button",
                    class: if lit == Some(i) { "chip-unselected card-search-chip card-search-lit" } else { "chip-unselected card-search-chip" },
                    style: "animation-delay: {i.saturating_mul(40)}ms;",
                    // The box keeps focus, so the nav's dropdown stays open under the tap.
                    onmousedown: move |event| event.prevent_default(),
                    onclick: move |_| {
                        on_action.call(Action::Pick(id));
                        opened.call(());
                    },
                    "{name}"
                }
            }
        }
    };

    match place {
        SearchPlace::Nav => rsx! {
            div {
                class: if filled { "card-search card-search-nav has-text" } else { "card-search card-search-nav" },
                id: "{box_id}",
                onfocusin: move |_| {
                    document::eval(&grow_js(&grow_id));
                },
                div { class: "card-search-field",
                    svg {
                        class: "card-search-icon",
                        view_box: "0 0 24 24",
                        xmlns: "http://www.w3.org/2000/svg",
                        "aria-hidden": "true",
                        path { d: SEARCH_ICON }
                    }
                    {field}
                }
                if filled && shows_under {
                    div { class: "card-search-results card-search-drop", {under} }
                }
            }
        },
        SearchPlace::Panel => rsx! {
            div { class: "card-search card-search-in-panel",
                {field}
                if filled && shows_under {
                    div { class: "card-search-results", {under} }
                }
            }
        },
    }
}

/// Home's Card search panel on a phone: the panel's title over the bar alone. `class` adds the
/// host's own classes, such as the one that hides it where the nav holds the box.
#[component]
pub fn CardSearchPanel(
    model: Signal<State>,
    on_action: Callback<Action>,
    input_id: String,
    #[props(default)] class: String,
) -> Element {
    rsx! {
        div { class: "container-sm flex-col home-flavor home-card-search {class}",
            div { class: "settings-list-header home-panel-header-centered",
                div { class: "home-panel-header-title",
                    span { class: "settings-list-title", {model::PANEL_TITLE} }
                }
            }
            div { class: "home-card-search-body",
                CardSearchBar { model, on_action, place: SearchPlace::Panel, input_id }
            }
        }
    }
}

/// Sizes the nav box with id `box_id` for focus: [`NAV_GROWN_PX`] when the nav has that much
/// room before its first link on the box's row, all of that room when it is at least
/// [`NAV_ROOM_MIN_PX`], else out to the last link's end, over the links, marking the nav so the
/// links it covers fade.
fn grow_js(box_id: &str) -> String {
    format!(
        r"(() => {{
const box = document.getElementById('{box_id}');
if (!box) return;
const nav = box.closest('nav');
const items = nav ? [...nav.querySelectorAll('.nav-links > li')] : [];
const first = items[0];
const last = items[items.length - 1];
const b = box.getBoundingClientRect();
let grown = {NAV_GROWN_PX};
let covers = false;
if (first && last) {{
    const f = first.getBoundingClientRect();
    const l = last.getBoundingClientRect();
    const room = Math.round(f.left - {NAV_GAP_PX} - b.left);
    const sameRow = f.top < b.bottom && f.bottom > b.top;
    if (sameRow && room < {NAV_GROWN_PX}) {{
        if (room >= {NAV_ROOM_MIN_PX}) {{
            grown = room;
        }} else {{
            grown = Math.max(Math.round(l.right - b.left), b.width);
            covers = true;
        }}
    }}
}}
box.style.setProperty('--card-search-grown', grown + 'px');
if (nav) nav.classList.toggle('card-search-covers', covers);
}})()"
    )
}

/// Takes focus off the box with id `input_id`.
fn blur_js(input_id: &str) -> String {
    format!("document.getElementById('{input_id}')?.blur();")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_grow_script_names_the_box_and_the_grown_width() {
        let js = grow_js("nav-card-search-box");
        assert!(js.contains("getElementById('nav-card-search-box')"), "{js}");
        assert!(js.contains("let grown = 320;"), "{js}");
        assert!(js.contains("f.left - 16 - b.left"), "{js}");
        assert!(js.contains("room < 320"), "{js}");
        assert!(js.contains("room >= 240"), "{js}");
        assert!(js.contains("card-search-covers"), "{js}");
    }

    #[test]
    fn the_blur_script_names_the_box() {
        assert_eq!(
            blur_js("nav-card-search"),
            "document.getElementById('nav-card-search')?.blur();"
        );
    }
}
