//! Shared changelog rendering.
//!
//! The release history, rendered identically on the website (`zite`) and in the
//! app (`zwiper`). The data lives in `zwipe_core::content::changelog` (pure, so
//! the server can serve it too); this crate owns the rendering and styling so
//! the surfaces never drift. Each consumer wraps the [`Changelog`] component in
//! its own chrome (a page with nav/footer on the web, a sheet in the app).
//! Styling is in `assets/components.css`.
//!
//! The component takes the changelog as a prop, defaulting to the copy compiled
//! into the binary ([`HttpChangelog::current`]). `zite` renders that default;
//! `zwiper` fetches `/api/changelog` and feeds the fresh copy in, so new
//! entries appear without an app resubmit. Passing the same wire type either
//! way keeps one rendering path.

use dioxus::prelude::*;
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// The `major.minor` of a version string, e.g. "1.3.1" -> "1.3", "1.0.10" ->
/// "1.0". Slices the input, so a `&str` in yields a `&str` borrowed from it.
fn major_minor(version: &str) -> &str {
    version.rsplit_once('.').map_or(version, |(head, _)| head)
}

/// The release history: a major.minor chip filter over a newest-first list of
/// versions, each with its date and notes. Defaults to the latest release's
/// line; "All" shows everything. Renders just the content block (chips + list),
/// so wrap it in your own page or sheet chrome.
///
/// `data` defaults to the compiled-in changelog; pass a fetched
/// [`HttpChangelog`] to render a server-updated copy.
#[component]
pub fn Changelog(#[props(default = HttpChangelog::current())] data: HttpChangelog) -> Element {
    // major.minor keys in display order (newest first), deduped, for the
    // filter. Owned so the chip click handlers can capture them.
    let mut minors: Vec<String> = Vec::new();
    for release in data.upcoming.iter().chain(data.releases.iter()) {
        let key = major_minor(&release.version).to_string();
        if !minors.contains(&key) {
            minors.push(key);
        }
    }

    // None = "All"; Some(key) narrows to one line. Defaults to the latest
    // released line (not the upcoming teaser); "All" stays an option.
    let default_line = data
        .releases
        .first()
        .map(|r| major_minor(&r.version).to_string());
    let mut selected = use_signal(|| default_line.clone());
    // Included in each card's key so switching filters remounts the visible
    // cards, replaying their ease-in animation.
    let filter_key = selected().unwrap_or_else(|| "all".to_string());
    // Upcoming entries render first with an "Upcoming" badge; the first released
    // entry after them is the "Latest".
    let upcoming_count = data.upcoming.len();

    rsx! {
        div { class: "changelog-filter",
            button {
                class: if selected().is_none() { "chip selected" } else { "chip" },
                onclick: move |_| selected.set(None),
                "All"
            }
            for key in minors {
                {
                    let is_selected = selected().as_deref() == Some(key.as_str());
                    let label = key.clone();
                    rsx! {
                        button {
                            class: if is_selected { "chip selected" } else { "chip" },
                            onclick: move |_| selected.set(Some(key.clone())),
                            "{label}"
                        }
                    }
                }
            }
        }
        div { class: "changelog-list",
            for (i, release) in data.upcoming.iter().chain(data.releases.iter()).enumerate() {
                if selected().as_deref().is_none_or(|key| key == major_minor(&release.version)) {
                    div { key: "{filter_key}-{release.version}", class: "changelog-card",
                        div { class: "changelog-version-row",
                            h2 { class: "changelog-version", "{release.version}" }
                            span { class: "changelog-date", "{release.date}" }
                            if i < upcoming_count {
                                span { class: "status-tag status-doing", "Upcoming" }
                            } else if i == upcoming_count {
                                span { class: "status-tag status-done", "Latest" }
                            }
                        }
                        ul { class: "changelog-bullets",
                            for entry in release.entries.iter() {
                                li { "{entry}" }
                            }
                        }
                    }
                }
            }
        }
    }
}
