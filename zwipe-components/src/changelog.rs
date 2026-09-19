//! Release history, rendered the same on zite and in zwiper. The data lives in
//! `zwipe_core::content::changelog`; zwiper fetches `/api/changelog` so new
//! entries appear without an app resubmit.

use dioxus::prelude::*;
use zwipe_core::http::contracts::changelog::HttpChangelog;

/// The `major.minor` of a version string: "1.3.1" -> "1.3", "1.0.10" -> "1.0".
fn major_minor(version: &str) -> &str {
    version.rsplit_once('.').map_or(version, |(head, _)| head)
}

/// A major.minor chip filter over a newest-first release list. Renders just
/// the content block; wrap it in your own page or sheet chrome. `data`
/// defaults to the compiled-in changelog.
#[component]
pub fn Changelog(#[props(default = HttpChangelog::current())] data: HttpChangelog) -> Element {
    let mut minors: Vec<String> = Vec::new();
    for release in data.upcoming.iter().chain(data.releases.iter()) {
        let key = major_minor(&release.version).to_string();
        if !minors.contains(&key) {
            minors.push(key);
        }
    }

    // `None` is "All". Defaults to the latest released line, not the upcoming teaser.
    let default_line = data
        .releases
        .first()
        .map(|r| major_minor(&r.version).to_string());
    let mut selected = use_signal(|| default_line.clone());
    // Part of each card's key so switching filters remounts and replays the ease-in.
    let filter_key = selected().unwrap_or_else(|| "all".to_string());
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
