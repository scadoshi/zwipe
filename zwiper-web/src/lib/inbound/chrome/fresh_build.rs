//! Moves to a newer build at the next page change, and tells a page that
//! stays put that one is live. The deploy bakes its commit into the bundle
//! (`ZWIPER_BUILD`) and writes the same commit to `/version.json`.
//!
//! After a move the leave guard lets through, the app fetches that file, at
//! most once per five minutes; once it names another build, the page the
//! move went to loads in full, so the browser fetches the new bundle.
//!
//! A page that stays also asks as the window takes focus or comes back into
//! view, and every five minutes while it shows, at most once a minute
//! ([`use_new_build_watch`]). Once a newer build is known,
//! [`NewBuildNotice`] offers it: "New version available" with Reload, until
//! it is dismissed or used. It never takes focus and never reloads by
//! itself. A build without `ZWIPER_BUILD` (a local one) never asks.

use components::Banner;
use dioxus::prelude::*;

/// The build this bundle is, baked in by the deploy.
const RUNNING_BUILD: Option<&str> = option_env!("ZWIPER_BUILD");

/// Called after every move the leave guard lets through, once the URL is
/// the move's: reloads it in full when a newer build is known, or asks
/// whether one is live when the last ask is five minutes old. Off the
/// browser it does nothing.
pub(crate) fn after_move() {
    #[cfg(target_arch = "wasm32")]
    if let Some(running) = RUNNING_BUILD {
        browser::after_move(running);
    }
}

/// The newer build the notice offers, once one is known. `Copy`: one signal,
/// from [`use_new_build_watch`].
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct NewBuild(Signal<Option<String>>);

/// Fires whenever the page should ask, while it shows: as the window takes
/// focus, as the tab comes back into view, and every five minutes.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "the browser build installs it")
)]
const WATCH_JS: &str = "const ask = () => { if (document.visibilityState === 'visible') dioxus.send(1); }; window.addEventListener('focus', ask); document.addEventListener('visibilitychange', ask); setInterval(ask, 300000); await new Promise(() => {});";

/// Watches for a newer build while the page stays put and provides what
/// [`NewBuildNotice`] shows. Mount once, at the app's root.
pub fn use_new_build_watch() -> NewBuild {
    let live = use_signal(|| None);
    use_hook(move || {
        #[cfg(target_arch = "wasm32")]
        if let Some(running) = RUNNING_BUILD {
            browser::watch(running, live);
        }
    });
    use_context_provider(|| NewBuild(live))
}

/// "New version available" with Reload, in the corner where announcements
/// show, while a build newer than this one is live. Its × dismisses it for
/// that build: the banner is keyed by the build, so a later one shows again.
#[component]
pub fn NewBuildNotice() -> Element {
    let NewBuild(live) = use_context();
    let Some(build) = rules::notice_for(RUNNING_BUILD.unwrap_or_default(), live().as_deref())
    else {
        return rsx! {};
    };
    // A key only remounts inside a list, so the one banner is a list of one.
    rsx! {
        div { class: "banner-stack new-build-notice", role: "status",
            for build in [build] {
                Banner { key: "{build}",
                    "New version available"
                    button {
                        r#type: "button",
                        class: "btn-action new-build-reload",
                        onclick: move |_| {
                            document::eval("location.reload()");
                        },
                        "Reload"
                    }
                }
            }
        }
    }
}

/// When to ask and what an answer means, apart from the browser so the
/// native build tests it.
#[cfg_attr(
    not(any(target_arch = "wasm32", test)),
    expect(
        dead_code,
        reason = "the browser build calls these; the native build only tests them"
    )
)]
mod rules {
    use serde::Deserialize;

    /// The least time between two fetches of the version file: five minutes.
    const CHECK_EVERY_MS: f64 = 300_000.0;

    /// How long after a move its own answer may still reload the page: an
    /// answer later than this waits for the next move.
    const RELOAD_WITHIN_MS: f64 = 5_000.0;

    /// `/version.json` as the deploy writes it.
    #[derive(Debug, Deserialize)]
    struct VersionFile {
        /// The live build's commit.
        build: String,
    }

    /// The least time between two asks from a page that stays: a minute.
    const POLL_EVERY_MS: f64 = 60_000.0;

    /// Whether a page that stays asks at `now_ms` (a focus, a return to view,
    /// the five-minute tick), given the last fetch at `last_ms`.
    pub(super) fn poll_due(last_ms: Option<f64>, now_ms: f64) -> bool {
        last_ms.is_none_or(|last| now_ms - last >= POLL_EVERY_MS)
    }

    /// The build the notice offers: `live` when it is known and is not
    /// `running`.
    pub(super) fn notice_for(running: &str, live: Option<&str>) -> Option<String> {
        live.filter(|live| *live != running).map(str::to_owned)
    }

    /// Whether a move at `now_ms` fetches the version file, given the last
    /// fetch at `last_ms`.
    pub(super) fn check_due(last_ms: Option<f64>, now_ms: f64) -> bool {
        last_ms.is_none_or(|last| now_ms - last >= CHECK_EVERY_MS)
    }

    /// The build `/version.json` names, or `None` for anything else (a
    /// missing file answered with the app's own page, an empty name).
    pub(super) fn live_build(body: &str) -> Option<String> {
        let file: VersionFile = serde_json::from_str(body).ok()?;
        let build = file.build.trim();
        (!build.is_empty()).then(|| build.to_owned())
    }

    /// Whether `live` names a build other than `running`.
    pub(super) fn is_newer(running: &str, live: Option<&str>) -> bool {
        live.is_some_and(|live| live != running)
    }

    /// Whether an answer that names a newer build reloads the page now: the
    /// page is still the one the move went to (`moved_to`, `on`), the answer
    /// came `elapsed_ms` after the move, and no screen holds unsaved
    /// changes (`armed`). Otherwise the next move loads in full.
    pub(super) fn reload_on_answer(moved_to: &str, on: &str, elapsed_ms: f64, armed: bool) -> bool {
        moved_to == on && elapsed_ms <= RELOAD_WITHIN_MS && !armed
    }
}

/// The browser half: the clock, the fetch and the reload.
#[cfg(target_arch = "wasm32")]
mod browser {
    use std::cell::Cell;

    use dioxus::prelude::*;
    use wasm_bindgen::JsCast;
    use wasm_bindgen_futures::JsFuture;
    use web_sys::{RequestCache, RequestInit, Response, Window};

    use super::rules;
    use crate::inbound::chrome::leave_guard;

    /// The deploy's version file, beside `index.html`.
    const VERSION_PATH: &str = "/version.json";

    thread_local! {
        /// When the version file was last fetched, in ms since the epoch.
        static LAST_ASKED: Cell<Option<f64>> = const { Cell::new(None) };
        /// Whether a newer build is known to be live.
        static NEWER_LIVE: Cell<bool> = const { Cell::new(false) };
        /// Where a newer build goes for the notice, once the root watches.
        static NOTICE: Cell<Option<Signal<Option<String>>>> = const { Cell::new(None) };
    }

    /// Tells the notice that `live` is the newer build.
    fn offer(live: String) {
        if let Some(mut notice) = NOTICE.get() {
            notice.set(Some(live));
        }
    }

    /// [`super::use_new_build_watch`] for the bundle built as `running`: hands
    /// the notice its signal and asks each time [`super::WATCH_JS`] fires.
    pub(super) fn watch(running: &'static str, notice: Signal<Option<String>>) {
        NOTICE.set(Some(notice));
        spawn(async move {
            let mut fires = document::eval(super::WATCH_JS);
            while fires.recv::<u8>().await.is_ok() {
                poll(running);
            }
        });
    }

    /// One ask from a page that stays, at most once a minute.
    fn poll(running: &'static str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        let asked_at = js_sys::Date::now();
        if !rules::poll_due(LAST_ASKED.get(), asked_at) {
            return;
        }
        LAST_ASKED.set(Some(asked_at));
        wasm_bindgen_futures::spawn_local(async move {
            let live = fetch_live(&window).await;
            if rules::is_newer(running, live.as_deref())
                && let Some(live) = live
            {
                NEWER_LIVE.set(true);
                offer(live);
            }
        });
    }

    /// [`super::after_move`] for the bundle built as `running`.
    pub(super) fn after_move(running: &'static str) {
        let Some(window) = web_sys::window() else {
            return;
        };
        if NEWER_LIVE.get() {
            if !leave_guard::is_armed() {
                // The URL is already the move's, so this loads its page.
                let _ = window.location().reload();
            }
            return;
        }
        let asked_at = js_sys::Date::now();
        if !rules::check_due(LAST_ASKED.get(), asked_at) {
            return;
        }
        LAST_ASKED.set(Some(asked_at));
        let moved_to = window.location().href().unwrap_or_default();
        wasm_bindgen_futures::spawn_local(async move {
            let live = fetch_live(&window).await;
            if !rules::is_newer(running, live.as_deref()) {
                return;
            }
            NEWER_LIVE.set(true);
            if let Some(live) = live {
                offer(live);
            }
            let on = window.location().href().unwrap_or_default();
            let elapsed = js_sys::Date::now() - asked_at;
            if rules::reload_on_answer(&moved_to, &on, elapsed, leave_guard::is_armed()) {
                let _ = window.location().reload();
            }
        });
    }

    /// The build the version file names, fetched past every cache, or
    /// `None` when the fetch fails or the file names none.
    async fn fetch_live(window: &Window) -> Option<String> {
        let init = RequestInit::new();
        init.set_cache(RequestCache::NoStore);
        let answer = JsFuture::from(window.fetch_with_str_and_init(VERSION_PATH, &init))
            .await
            .ok()?;
        let response: Response = answer.dyn_into().ok()?;
        if !response.ok() {
            return None;
        }
        let body = JsFuture::from(response.text().ok()?).await.ok()?;
        rules::live_build(&body.as_string()?)
    }
}

#[cfg(test)]
mod notice_tests {
    use dioxus::core::Mutations;
    use dioxus::prelude::*;

    use super::{NewBuild, NewBuildNotice};

    /// The notice under a root that offers `bbb`.
    fn app() -> Element {
        let live = use_signal(|| Some("bbb".to_owned()));
        use_context_provider(|| NewBuild(live));
        rsx! { NewBuildNotice {} }
    }

    /// The click handlers `edits` attaches: the banner's × and Reload each
    /// time a banner mounts.
    fn clicks_attached(edits: &Mutations) -> usize {
        edits
            .edits
            .iter()
            .filter(|edit| {
                matches!(edit, dioxus::core::Mutation::NewEventListener { name, .. } if name == "click")
            })
            .count()
    }

    #[test]
    fn a_later_build_mounts_a_fresh_banner_so_a_dismissed_one_shows_again() {
        let mut dom = VirtualDom::new(app);
        let first = dom.rebuild_to_vec();
        assert_eq!(clicks_attached(&first), 2, "the banner's × and Reload");

        dom.in_scope(ScopeId::APP, || {
            let NewBuild(mut live) = consume_context();
            live.set(Some("ccc".to_owned()));
        });
        let later = dom.render_immediate_to_vec();
        assert_eq!(
            clicks_attached(&later),
            2,
            "a new build mounts a new banner, whose dismissal starts over: {:#?}",
            later.edits
        );

        dom.in_scope(ScopeId::APP, || {
            let NewBuild(mut live) = consume_context();
            live.set(Some("ccc".to_owned()));
        });
        let again = dom.render_immediate_to_vec();
        assert_eq!(
            clicks_attached(&again),
            0,
            "the same build asked again keeps its banner, dismissed or not"
        );
    }
}

#[cfg(test)]
mod tests {
    use super::rules::*;

    /// The headers Cloudflare Pages serves app.zwipe.net with.
    const HEADERS: &str = include_str!("../../../../deploy/_headers");

    #[test]
    fn pages_revalidates_every_page_and_never_stores_the_version_file() {
        assert!(
            HEADERS.contains("/*\n  Cache-Control: no-cache\n"),
            "index.html and every app route revalidate"
        );
        assert!(
            HEADERS.contains("/version.json\n  ! Cache-Control\n  Cache-Control: no-store\n"),
            "the version file is never stored"
        );
    }

    #[test]
    fn a_page_that_stays_asks_on_focus_at_most_once_a_minute() {
        assert!(poll_due(None, 0.0), "the first focus asks");
        assert!(
            !poll_due(Some(1_000.0), 1_000.0 + 59_999.0),
            "not again within the minute"
        );
        assert!(poll_due(Some(1_000.0), 1_000.0 + 60_000.0), "at a minute");
        assert!(
            !poll_due(Some(5_000.0), 4_000.0),
            "a clock that went back waits"
        );
    }

    #[test]
    fn the_notice_offers_only_a_build_other_than_this_one() {
        assert_eq!(
            notice_for("aaa", Some("bbb")).as_deref(),
            Some("bbb"),
            "a newer build is offered"
        );
        assert_eq!(
            notice_for("aaa", Some("aaa")),
            None,
            "the running build is not"
        );
        assert_eq!(
            notice_for("aaa", None),
            None,
            "nothing known, nothing offered"
        );
    }

    #[test]
    fn the_first_move_asks_and_then_once_per_five_minutes() {
        assert!(check_due(None, 0.0), "the first move asks");
        assert!(
            !check_due(Some(1_000.0), 1_000.0 + 299_999.0),
            "not before five minutes"
        );
        assert!(
            check_due(Some(1_000.0), 1_000.0 + 300_000.0),
            "at five minutes"
        );
        assert!(
            !check_due(Some(5_000.0), 4_000.0),
            "a clock that went back waits"
        );
    }

    #[test]
    fn the_version_file_names_a_build_and_nothing_else_does() {
        assert_eq!(
            live_build(r#"{"build":"abc123"}"#).as_deref(),
            Some("abc123")
        );
        assert_eq!(
            live_build(r#"{"build":" abc123\n"}"#).as_deref(),
            Some("abc123")
        );
        assert_eq!(
            live_build("<!doctype html><html></html>"),
            None,
            "the app's page"
        );
        assert_eq!(live_build(r#"{"build":""}"#), None, "an empty name");
        assert_eq!(live_build(r#"{"other":"x"}"#), None);
    }

    #[test]
    fn only_another_named_build_is_newer() {
        assert!(is_newer("abc", Some("def")));
        assert!(!is_newer("abc", Some("abc")));
        assert!(!is_newer("abc", None), "no answer is no news");
    }

    #[test]
    fn an_answer_reloads_only_the_page_moved_to_soon_and_without_unsaved_changes() {
        assert!(reload_on_answer("/decks", "/decks", 300.0, false));
        assert!(
            !reload_on_answer("/decks", "/decks/new", 300.0, false),
            "moved on since"
        );
        assert!(
            !reload_on_answer("/decks", "/decks", 5_001.0, false),
            "too late"
        );
        assert!(
            !reload_on_answer("/decks", "/decks", 300.0, true),
            "a form holds changes"
        );
    }
}
