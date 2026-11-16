//! The changelog every client draws: the compiled copy on the first render, the server's
//! swapped in when its fetch lands with a different copy, and the compiled one kept when
//! the fetch fails.

#![expect(
    clippy::unwrap_used,
    reason = "test helpers: a failed read or render fails the test"
)]

use components::Changelog;
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
    fetched: Option<HttpChangelog>,
    version: Option<String>,
}

/// A client's changelog with its `version`, whose fetch answers `fetched` once the first
/// render has drawn.
#[expect(non_snake_case, reason = "a Dioxus component")]
fn Client(props: ClientProps) -> Element {
    let ClientProps { fetched, version } = props;
    let mut cache = use_signal(ChangelogCache::default);
    use_effect(move || {
        let next = cache.peek().after_fetch(fetched.clone());
        if let Some(next) = next {
            cache.set(next);
        }
    });
    let data = cache.read().shown().clone();
    rsx! { Changelog { data, version } }
}

/// The first render, and the render once the fetch's answer has been applied.
fn renders(fetched: Option<HttpChangelog>, version: Option<&str>) -> (String, String) {
    let version = version.map(str::to_owned);
    let mut dom = VirtualDom::new_with_props(Client, ClientProps { fetched, version });
    dom.rebuild_in_place();
    let first = dioxus_ssr::render(&dom);
    for _ in 0..8 {
        dom.process_events();
        dom.render_immediate(&mut NoOpMutations);
    }
    (first, dioxus_ssr::render(&dom))
}

#[test]
fn the_compiled_copy_renders_first() {
    let (first, _) = renders(Some(server()), None);
    assert!(first.contains(&compiled_line()), "{first}");
    assert!(!first.contains(SERVER_ONLY), "{first}");
}

#[test]
fn the_servers_copy_replaces_the_compiled_one_when_it_lands() {
    let (_, landed) = renders(Some(server()), None);
    assert!(landed.contains(SERVER_ONLY), "{landed}");
    assert!(landed.contains(&compiled_line()), "{landed}");
}

#[test]
fn a_failed_fetch_keeps_the_compiled_copy() {
    let (first, after) = renders(None, None);
    assert_eq!(first, after);
    assert!(after.contains(&compiled_line()), "{after}");
}

#[test]
fn a_server_copy_that_matches_the_compiled_one_draws_the_same_page() {
    let (first, after) = renders(Some(HttpChangelog::current()), None);
    assert_eq!(first, after);
}

#[test]
fn every_client_draws_one_titled_panel_with_no_hero_or_tagline() {
    let (page, _) = renders(None, None);
    assert!(
        page.contains("settings-list about-panel changelog-panel"),
        "{page}"
    );
    assert!(page.contains("settings-list-header"), "{page}");
    assert!(page.contains("changelog-panels"), "{page}");
    assert!(!page.contains("page-header"), "{page}");
    assert!(!page.contains("<h1"), "{page}");
    assert!(!page.contains("tagline"), "{page}");
}

#[test]
fn a_clients_version_is_a_chip_beside_the_title_and_zite_has_none() {
    let (app, _) = renders(None, Some("v1.21.9"));
    assert!(
        app.contains(r#"<span class="stat-chip">v1.21.9</span>"#),
        "{app}"
    );
    let (site, _) = renders(None, None);
    assert!(!site.contains("stat-chip"), "{site}");
}
