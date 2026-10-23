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
//!
//! Releases are grouped by the month they shipped, newest first. The upcoming
//! release sits with the newest month. Once the history spans more than one
//! year, a row of year chips sits above the months, and the month chips drop
//! the year from their labels.
//!
//! Every card carries an id from its version ([`anchor_id`], `v1-12-1` for
//! 1.12.1). zite links each version heading to it and passes the URL fragment
//! in as `anchor`, which selects the chips holding that release; zite then
//! scrolls the card into view. A release whose day cannot be read (a
//! malformed `date` from some server) goes under a last "Earlier" chip rather
//! than vanishing; the day comes from [`HttpRelease::released_on`], which also
//! reads the display string an older server sends.

use dioxus::prelude::*;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

/// A month chip: `key` filters (`"2026-10"`), `label` shows when the history
/// fits in one year (`"Oct 2026"`), `short` shows under a year chip (`"Oct"`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Month {
    key: String,
    year: String,
    label: String,
    short: String,
}

impl Month {
    /// The chip for releases whose day cannot be read. It has no year of its
    /// own; [`group_by_year`] puts it after the oldest year's months.
    fn earlier() -> Self {
        Self {
            key: "earlier".to_string(),
            year: String::new(),
            label: "Earlier".to_string(),
            short: "Earlier".to_string(),
        }
    }

    /// The month a release shipped in, if its day can be read.
    fn of(release: &HttpRelease) -> Option<Self> {
        release.released_on().map(|day| Self {
            key: day.format("%Y-%m").to_string(),
            year: day.format("%Y").to_string(),
            label: day.format("%b %Y").to_string(),
            short: day.format("%b").to_string(),
        })
    }
}

/// One year chip and its months, newest first.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Year {
    year: String,
    months: Vec<Month>,
}

/// The element id of a release's card: `v` and the version with its dots as
/// dashes, `v1-12-1` for 1.12.1, so it needs no escaping in a CSS selector.
pub fn anchor_id(version: &str) -> String {
    format!("v{}", version.replace('.', "-"))
}

/// The year and month chips that show the release `fragment` names, as
/// `(year, month key)`. Takes the fragment with or without its `#`. `None` when
/// no release has that anchor.
fn anchor_selection(
    fragment: &str,
    data: &HttpChangelog,
    months: &[Month],
    years: &[Year],
) -> Option<(String, String)> {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    let (_, month) = data
        .upcoming
        .iter()
        .chain(data.releases.iter())
        .zip(months)
        .find(|(release, _)| anchor_id(&release.version) == fragment)?;
    let year = years.iter().find(|year| year.months.contains(month))?;
    Some((year.year.clone(), month.key.clone()))
}

/// The month each release sits under, in display order: upcoming first, then
/// the shipped releases. An upcoming release has no day of its own, so it
/// joins the newest shipped month.
fn release_months(data: &HttpChangelog) -> Vec<Month> {
    let newest = data.releases.iter().find_map(Month::of);
    let upcoming = data.upcoming.iter().map(|release| {
        Month::of(release)
            .or_else(|| newest.clone())
            .unwrap_or_else(Month::earlier)
    });
    let shipped = data
        .releases
        .iter()
        .map(|release| Month::of(release).unwrap_or_else(Month::earlier));
    upcoming.chain(shipped).collect()
}

/// Years newest first, each with its months newest first, every month once.
/// "Earlier" goes last, at the end of the oldest year (or alone, in a year
/// with an empty label, when no release has a readable day).
fn group_by_year(months: &[Month]) -> Vec<Year> {
    let earlier = Month::earlier();
    let mut years: Vec<Year> = Vec::new();
    for month in months.iter().filter(|month| **month != earlier) {
        match years.iter_mut().find(|year| year.year == month.year) {
            Some(year) if year.months.contains(month) => {}
            Some(year) => year.months.push(month.clone()),
            None => years.push(Year {
                year: month.year.clone(),
                months: vec![month.clone()],
            }),
        }
    }
    if months.contains(&earlier) {
        match years.last_mut() {
            Some(oldest) => oldest.months.push(earlier),
            None => years.push(Year {
                year: String::new(),
                months: vec![earlier],
            }),
        }
    }
    years
}

/// The release history: month chips over a newest-first list of versions,
/// each with its date and notes, with a row of year chips above once the
/// history spans more than one year. Defaults to the newest month; "All" shows
/// everything. Renders just the content block (chips + list), so wrap it in
/// your own page or sheet chrome.
///
/// `data` defaults to the compiled-in changelog; pass a fetched
/// [`HttpChangelog`] to render a server-updated copy. `anchor` is a card id
/// (see [`anchor_id`]) whose chips to select; an unknown one leaves the
/// default. `link_versions` makes each version heading a link to its card.
#[component]
pub fn Changelog(
    #[props(default = HttpChangelog::current())] data: HttpChangelog,
    #[props(default)] anchor: Option<String>,
    #[props(default)] link_versions: bool,
) -> Element {
    // The month of each release, parallel to the upcoming-then-shipped list.
    let months = release_months(&data);
    let years = group_by_year(&months);
    let by_year = years.len() > 1;

    // The year whose months the month row shows, and the month the list
    // narrows to (None = "All"). Both default to the newest.
    let newest_year = years
        .first()
        .map(|year| year.year.clone())
        .unwrap_or_default();
    let newest_month = years
        .first()
        .and_then(|year| year.months.first())
        .map(|month| month.key.clone());
    let mut selected_year = use_signal(|| newest_year.clone());
    let mut selected = use_signal(|| newest_month.clone());
    // Selects the chips holding the anchored release whenever the anchor
    // names one.
    let target = anchor
        .as_deref()
        .and_then(|fragment| anchor_selection(fragment, &data, &months, &years));
    use_effect(use_reactive!(|target| {
        if let Some((year, month)) = target {
            selected_year.set(year);
            selected.set(Some(month));
        }
    }));
    // Included in each card's key so switching filters remounts the visible
    // cards, replaying their ease-in animation.
    let filter_key = selected().unwrap_or_else(|| "all".to_string());
    // Upcoming entries render first with an "Upcoming" badge; the first released
    // entry after them is the "Latest".
    let upcoming_count = data.upcoming.len();

    let month_chips: Vec<Month> = years
        .iter()
        .find(|year| !by_year || year.year == selected_year())
        .map(|year| year.months.clone())
        .unwrap_or_default();

    let all_chip = rsx! {
        button {
            class: if selected().is_none() { "chip selected" } else { "chip" },
            onclick: move |_| selected.set(None),
            "All"
        }
    };

    rsx! {
        if by_year {
            div { class: "changelog-filter changelog-years",
                {all_chip.clone()}
                for year in years.iter().cloned() {
                    {
                        let is_selected = selected().is_some() && selected_year() == year.year;
                        let label = year.year.clone();
                        let first = year.months.first().map(|month| month.key.clone());
                        rsx! {
                            button {
                                class: if is_selected { "chip selected" } else { "chip" },
                                onclick: move |_| {
                                    selected_year.set(year.year.clone());
                                    selected.set(first.clone());
                                },
                                "{label}"
                            }
                        }
                    }
                }
            }
        }
        div { class: "changelog-filter",
            if !by_year {
                {all_chip}
            }
            for month in month_chips {
                {
                    let is_selected = selected().as_deref() == Some(month.key.as_str());
                    let label = if by_year { month.short.clone() } else { month.label.clone() };
                    let key = month.key;
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
            for (i, (release, month)) in data.upcoming.iter().chain(data.releases.iter()).zip(months.iter()).enumerate() {
                if selected().as_deref().is_none_or(|key| key == month.key) {
                    div {
                        key: "{filter_key}-{release.version}",
                        id: anchor_id(&release.version),
                        class: "changelog-card",
                        div { class: "changelog-version-row",
                            h2 { class: "changelog-version",
                                if link_versions {
                                    a {
                                        class: "changelog-anchor",
                                        href: "#{anchor_id(&release.version)}",
                                        "{release.version}"
                                    }
                                } else {
                                    "{release.version}"
                                }
                            }
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

#[cfg(test)]
#[allow(clippy::indexing_slicing)]
mod tests {
    use super::*;

    fn release(version: &str, date: &str) -> HttpRelease {
        HttpRelease {
            version: version.to_string(),
            date: date.to_string(),
            released_on: None,
            entries: Vec::new(),
        }
    }

    fn changelog(upcoming: &[&str], releases: &[(&str, &str)]) -> HttpChangelog {
        HttpChangelog {
            upcoming: upcoming
                .iter()
                .map(|version| release(version, ""))
                .collect(),
            releases: releases
                .iter()
                .map(|(version, date)| release(version, date))
                .collect(),
        }
    }

    fn keys(months: &[Month]) -> Vec<&str> {
        months.iter().map(|month| month.key.as_str()).collect()
    }

    fn year_names(years: &[Year]) -> Vec<&str> {
        years.iter().map(|year| year.year.as_str()).collect()
    }

    #[test]
    fn one_year_is_one_group_of_months_newest_first() {
        let data = changelog(
            &["1.14.0"],
            &[
                ("1.13.0", "Oct 22, 2026"),
                ("1.12.0", "Oct 1, 2026"),
                ("1.11.0", "Sep 30, 2026"),
                ("1.0.0", "Jun 6, 2026"),
            ],
        );
        let months = release_months(&data);
        assert_eq!(
            keys(&months),
            ["2026-10", "2026-10", "2026-10", "2026-09", "2026-06"]
        );
        let years = group_by_year(&months);
        assert_eq!(year_names(&years), ["2026"]);
        let labels: Vec<_> = years[0].months.iter().map(|m| m.label.as_str()).collect();
        assert_eq!(labels, ["Oct 2026", "Sep 2026", "Jun 2026"]);
    }

    #[test]
    fn two_years_group_newest_first() {
        let data = changelog(
            &["1.16.0"],
            &[
                ("1.15.1", "Jan 20, 2027"),
                ("1.15.0", "Jan 4, 2027"),
                ("1.14.1", "Dec 30, 2026"),
                ("1.14.0", "Dec 2, 2026"),
            ],
        );
        let months = release_months(&data);
        // Upcoming joins the newest month, in the newest year.
        assert_eq!(keys(&months)[0], "2027-01");
        let years = group_by_year(&months);
        assert_eq!(year_names(&years), ["2027", "2026"]);
        assert_eq!(keys(&years[0].months), ["2027-01"]);
        assert_eq!(keys(&years[1].months), ["2026-12"]);
        assert_eq!(years[0].months[0].short, "Jan");
        assert_eq!(years[1].months[0].short, "Dec");
    }

    #[test]
    fn upcoming_joins_the_newest_month() {
        let data = changelog(&["1.14.0"], &[("1.13.0", "Sep 2, 2026")]);
        assert_eq!(keys(&release_months(&data)), ["2026-09", "2026-09"]);
    }

    #[test]
    fn the_compiled_in_history_fits_in_2026() {
        let data = HttpChangelog::current();
        let months = release_months(&data);
        assert_eq!(months.len(), data.upcoming.len() + data.releases.len());
        assert!(!months.contains(&Month::earlier()));
        let years = group_by_year(&months);
        assert_eq!(year_names(&years), ["2026"]);
        assert_eq!(
            keys(&years[0].months),
            ["2026-10", "2026-09", "2026-08", "2026-07", "2026-06"]
        );
    }

    #[test]
    fn unreadable_dates_go_last_under_earlier() {
        let data = changelog(
            &[],
            &[
                ("1.3.0", "Jan 3, 2027"),
                ("1.2.0", "someday"),
                ("1.1.0", "Sep 3, 2026"),
            ],
        );
        let years = group_by_year(&release_months(&data));
        assert_eq!(year_names(&years), ["2027", "2026"]);
        assert_eq!(keys(&years[1].months), ["2026-09", "earlier"]);
    }

    #[test]
    fn anchor_ids_are_selector_safe() {
        assert_eq!(anchor_id("1.12.1"), "v1-12-1");
        assert_eq!(anchor_id("1.0.10"), "v1-0-10");
    }

    fn select(fragment: &str, data: &HttpChangelog) -> Option<(String, String)> {
        let months = release_months(data);
        let years = group_by_year(&months);
        anchor_selection(fragment, data, &months, &years)
    }

    fn picked(year: &str, month: &str) -> (String, String) {
        (year.to_string(), month.to_string())
    }

    #[test]
    fn an_anchor_selects_its_year_and_month() {
        let data = changelog(
            &["1.16.0"],
            &[
                ("1.15.0", "Jan 4, 2027"),
                ("1.14.1", "Dec 30, 2026"),
                ("1.14.0", "Nov 2, 2026"),
                ("1.13.9", "someday"),
            ],
        );
        assert_eq!(select("v1-14-0", &data), Some(picked("2026", "2026-11")));
        assert_eq!(select("#v1-14-1", &data), Some(picked("2026", "2026-12")));
        assert_eq!(select("v1-16-0", &data), Some(picked("2027", "2027-01")));
        assert_eq!(select("v1-13-9", &data), Some(picked("2026", "earlier")));
    }

    #[test]
    fn an_unknown_anchor_selects_nothing() {
        let data = HttpChangelog::current();
        assert_eq!(select("", &data), None);
        assert_eq!(select("v9-9-9", &data), None);
        assert_eq!(select("1.12.1", &data), None);
        assert_eq!(select("v1-12-1", &data), Some(picked("2026", "2026-10")));
    }

    #[test]
    fn nothing_readable_is_one_earlier_group() {
        let data = changelog(&["2.0.0"], &[("1.0.0", "someday")]);
        let years = group_by_year(&release_months(&data));
        assert_eq!(year_names(&years), [""]);
        assert_eq!(keys(&years[0].months), ["earlier"]);
    }
}
