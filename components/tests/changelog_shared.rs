//! The changelog every client draws: the compiled copy on the first render, the server's
//! swapped in when its fetch lands with a different copy, and the compiled one kept when
//! the fetch fails.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use components::{Changelog, ChangelogLook};
use dioxus::core::NoOpMutations;
use dioxus::prelude::*;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};
use zwipe_ui::changelog::ChangelogCache;

/// A line only the server's copy carries.
const SERVER_ONLY: &str = "A line only the server has";

/// The compiled copy with one more upcoming release, as a server deployed after this build
/// serves it.
fn server() -> HttpChangelog {
    let mut changelog = HttpChangelog::current();
    changelog.upcoming.insert(
        0,
        HttpRelease {
            version: "99.0.0".to_owned(),
            date: String::new(),
            released_on: None,
            entries: vec![SERVER_ONLY.to_owned()],
        },
    );
    changelog
}

/// The newest compiled release's first line.
fn compiled_line() -> String {
    let compiled = HttpChangelog::current();
    compiled
        .upcoming
        .iter()
        .chain(&compiled.releases)
        .find_map(|release| release.entries.first().cloned())
        .unwrap()
}

#[derive(Clone, PartialEq, Props)]
struct ClientProps {
    look: ChangelogLook,
    fetched: Option<HttpChangelog>,
}

/// A client's changelog under `look`, whose fetch answers `fetched` once the first render
/// has drawn.
#[expect(non_snake_case, reason = "a Dioxus component")]
fn Client(props: ClientProps) -> Element {
    let ClientProps { look, fetched } = props;
    let mut cache = use_signal(ChangelogCache::default);
    use_effect(move || {
        let next = cache.peek().after_fetch(fetched.clone());
        if let Some(next) = next {
            cache.set(next);
        }
    });
    let data = cache.read().shown().clone();
    rsx! { Changelog { data, look } }
}

/// The first render, and the render once the fetch's answer has been applied.
fn renders(look: ChangelogLook, fetched: Option<HttpChangelog>) -> (String, String) {
    let mut dom = VirtualDom::new_with_props(Client, ClientProps { look, fetched });
    dom.rebuild_in_place();
    let first = dioxus_ssr::render(&dom);
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    (first, dioxus_ssr::render(&dom))
}

#[test]
fn every_look_renders_the_compiled_copy_first() {
    for look in [ChangelogLook::Site, ChangelogLook::App] {
        let (first, _) = renders(look, Some(server()));
        assert!(first.contains(&compiled_line()), "{look:?}: {first}");
        assert!(!first.contains(SERVER_ONLY), "{look:?}: {first}");
    }
}

#[test]
fn the_servers_copy_replaces_the_compiled_one_when_it_lands() {
    for look in [ChangelogLook::Site, ChangelogLook::App] {
        let (_, landed) = renders(look, Some(server()));
        assert!(landed.contains(SERVER_ONLY), "{look:?}: {landed}");
        assert!(landed.contains(&compiled_line()), "{look:?}: {landed}");
    }
}

#[test]
fn a_failed_fetch_keeps_the_compiled_copy() {
    for look in [ChangelogLook::Site, ChangelogLook::App] {
        let (first, after) = renders(look, None);
        assert_eq!(first, after, "{look:?}");
        assert!(after.contains(&compiled_line()), "{look:?}: {after}");
    }
}

#[test]
fn a_server_copy_that_matches_the_compiled_one_draws_the_same_page() {
    let (first, after) = renders(ChangelogLook::Site, Some(HttpChangelog::current()));
    assert_eq!(first, after);
}

#[test]
fn the_site_look_keeps_the_pages_hero_and_the_app_look_its_panel() {
    let (site, _) = renders(ChangelogLook::Site, None);
    assert!(site.contains("page-header"), "{site}");
    assert!(site.contains("<h1"), "{site}");
    assert!(site.contains("Every release, newest first."), "{site}");
    let (app, _) = renders(ChangelogLook::App, None);
    assert!(app.contains("settings-list about-panel"), "{app}");
    assert!(app.contains("settings-list-header"), "{app}");
    assert!(!app.contains("<h1"), "{app}");
}
