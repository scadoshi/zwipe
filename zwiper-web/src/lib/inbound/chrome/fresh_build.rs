//! Moves to a newer build at the next page change. The deploy bakes its
//! commit into the bundle (`ZWIPER_BUILD`) and writes the same commit to
//! `/version.json`. After a move the leave guard lets through, the app
//! fetches that file, at most once per five minutes; once it names another
//! build, the page the move went to loads in full, so the browser fetches
//! the new bundle. Nothing polls: only a page change asks. A build without
//! `ZWIPER_BUILD` (a local one) never asks.

/// The build this bundle is, baked in by the deploy.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "the browser build reads it")
)]
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
