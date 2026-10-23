//! Shared changelog rendering.
//!
//! The release history, rendered identically on the website (`zite`) and in the
//! app (`zwiper`). The data lives in `zwipe_core::content::changelog` (pure, so
//! the server can serve it too); this crate owns the rendering and styling so
//! the surfaces never drift. Each consumer wraps the [`Changelog`] component in
//! its own chrome (a page with nav/footer on the web, a sheet in the app).
//! Styling is in `assets/magic.css`.
//!
//! The component takes the changelog as a prop, defaulting to the copy compiled
//! into the binary ([`HttpChangelog::current`]). `zite` renders that default;
//! `zwiper` fetches `/api/changelog` and feeds the fresh copy in, so new
//! entries appear without an app resubmit. Passing the same wire type either
//! way keeps one rendering path.
//!
//! The chips narrow the list as a tree. The first row is Latest and then each
//! year, newest first; picking a year adds a row of its months, and picking a
//! month adds a row of its versions. Latest, the default, shows the upcoming
//! release and the newest shipped one. The upcoming release counts in the
//! newest shipped month. The day comes from [`HttpRelease::released_on`], which
//! also reads the display string an older server sends; a release whose day
//! cannot be read goes under a last "Earlier" chip rather than vanishing.
//!
//! Every card carries an id from its version ([`anchor_id`], `v1-12-1` for
//! 1.12.1). zite links each version heading to it and passes the URL fragment
//! in as `anchor`, which selects the chips down to that release; zite then
//! scrolls the card into view.

use dioxus::prelude::*;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

/// The year and month key of releases whose day cannot be read.
const EARLIER: &str = "earlier";

/// Where a release sits in the chip tree: year key (`"2026"`), month key
/// (`"2026-10"`) and month label (`"Oct"`). Releases whose day cannot be read
/// sit under [`EARLIER`] for both keys.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Place {
    year: String,
    month: String,
    month_label: String,
}

impl Place {
    fn earlier() -> Self {
        Self {
            year: EARLIER.to_string(),
            month: EARLIER.to_string(),
            month_label: "Earlier".to_string(),
        }
    }

    /// Where a release goes by its own day, if it can be read.
    fn of(release: &HttpRelease) -> Option<Self> {
        release.released_on().map(|day| Self {
            year: day.format("%Y").to_string(),
            month: day.format("%Y-%m").to_string(),
            month_label: day.format("%b").to_string(),
        })
    }
}

/// What the chips have narrowed the list to. Each level names its parents, so
/// a selection is a path down the tree.
#[derive(Debug, Clone, PartialEq, Eq)]
enum Selection {
    /// The upcoming release and the newest shipped one.
    Latest,
    /// A whole year (or the Earlier bucket).
    Year(String),
    /// One month of a year.
    Month { year: String, month: String },
    /// One release.
    Version {
        year: String,
        month: String,
        version: String,
    },
}

impl Selection {
    /// One level up: a version goes back to its month, a month to its year,
    /// a year to Latest.
    fn parent(&self) -> Self {
        match self {
            Self::Latest | Self::Year(_) => Self::Latest,
            Self::Month { year, .. } => Self::Year(year.clone()),
            Self::Version { year, month, .. } => Self::Month {
                year: year.clone(),
                month: month.clone(),
            },
        }
    }

    fn year(&self) -> Option<&str> {
        match self {
            Self::Latest => None,
            Self::Year(year) | Self::Month { year, .. } | Self::Version { year, .. } => Some(year),
        }
    }

    fn month(&self) -> Option<&str> {
        match self {
            Self::Month { month, .. } | Self::Version { month, .. } => Some(month),
            _ => None,
        }
    }

    fn version(&self) -> Option<&str> {
        match self {
            Self::Version { version, .. } => Some(version),
            _ => None,
        }
    }
}

/// One chip: its label, whether it lights up, and what clicking it selects.
/// Clicking the deepest lit chip goes up a level; clicking any other chip
/// selects it, which clears the levels below.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Chip {
    label: String,
    selected: bool,
    target: Selection,
}

/// The place of each release, in display order: upcoming first, then the
/// shipped releases. An upcoming release has no day of its own, so it counts
/// in the newest shipped month.
fn release_places(data: &HttpChangelog) -> Vec<Place> {
    let newest = data.releases.iter().find_map(Place::of);
    let upcoming = data.upcoming.iter().map(|release| {
        Place::of(release)
            .or_else(|| newest.clone())
            .unwrap_or_else(Place::earlier)
    });
    let shipped = data
        .releases
        .iter()
        .map(|release| Place::of(release).unwrap_or_else(Place::earlier));
    upcoming.chain(shipped).collect()
}

/// Every release in display order with its place.
fn listed(data: &HttpChangelog) -> Vec<(&HttpRelease, Place)> {
    data.upcoming
        .iter()
        .chain(data.releases.iter())
        .zip(release_places(data))
        .collect()
}

/// Each distinct item, in first-seen order. Listed releases run newest first,
/// so this is newest first too.
fn distinct<T: Clone + PartialEq>(items: impl Iterator<Item = T>) -> Vec<T> {
    let mut seen: Vec<T> = Vec::new();
    for item in items {
        if !seen.contains(&item) {
            seen.push(item);
        }
    }
    seen
}

/// The chip rows for a selection: Latest and the years (Earlier last, when
/// any release landed there), then the selected year's months, then the
/// selected month's versions.
fn chip_rows(listed: &[(&HttpRelease, Place)], selection: &Selection) -> Vec<Vec<Chip>> {
    let chip = |label: String, own: Selection, lit: bool| Chip {
        label,
        selected: lit,
        target: if lit && own == *selection {
            own.parent()
        } else {
            own
        },
    };

    let mut years = distinct(
        listed
            .iter()
            .map(|(_, place)| place.year.clone())
            .filter(|year| year != EARLIER),
    );
    if listed.iter().any(|(_, place)| place.year == EARLIER) {
        years.push(EARLIER.to_string());
    }
    let mut first = vec![chip(
        "Latest".to_string(),
        Selection::Latest,
        *selection == Selection::Latest,
    )];
    first.extend(years.into_iter().map(|year| {
        let label = if year == EARLIER {
            "Earlier".to_string()
        } else {
            year.clone()
        };
        let lit = selection.year() == Some(year.as_str());
        chip(label, Selection::Year(year), lit)
    }));
    let mut rows = vec![first];

    let Some(year) = selection.year() else {
        return rows;
    };
    let months = distinct(
        listed
            .iter()
            .filter(|(_, place)| place.year == year)
            .map(|(_, place)| (place.month.clone(), place.month_label.clone())),
    );
    rows.push(
        months
            .into_iter()
            .map(|(month, label)| {
                let lit = selection.month() == Some(month.as_str());
                let own = Selection::Month {
                    year: year.to_string(),
                    month,
                };
                chip(label, own, lit)
            })
            .collect(),
    );

    let Some(month) = selection.month() else {
        return rows;
    };
    rows.push(
        listed
            .iter()
            .filter(|(_, place)| place.month == month)
            .map(|(release, _)| {
                let lit = selection.version() == Some(release.version.as_str());
                let own = Selection::Version {
                    year: year.to_string(),
                    month: month.to_string(),
                    version: release.version.clone(),
                };
                chip(release.version.clone(), own, lit)
            })
            .collect(),
    );
    rows
}

/// Which listed releases a selection shows, parallel to `listed`. Latest is
/// every upcoming release and the first shipped one.
fn visible(listed: &[(&HttpRelease, Place)], upcoming: usize, selection: &Selection) -> Vec<bool> {
    listed
        .iter()
        .enumerate()
        .map(|(i, (release, place))| match selection {
            Selection::Latest => i <= upcoming,
            Selection::Year(year) => place.year == *year,
            Selection::Month { month, .. } => place.month == *month,
            Selection::Version { version, .. } => release.version == *version,
        })
        .collect()
}

/// The element id of a release's card: `v` and the version with its dots as
/// dashes, `v1-12-1` for 1.12.1, so it needs no escaping in a CSS selector.
pub fn anchor_id(version: &str) -> String {
    format!("v{}", version.replace('.', "-"))
}

/// The selection down to the release `fragment` names. Takes the fragment
/// with or without its `#`. `None` when no release has that anchor.
fn anchor_selection(fragment: &str, listed: &[(&HttpRelease, Place)]) -> Option<Selection> {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    listed
        .iter()
        .find(|(release, _)| anchor_id(&release.version) == fragment)
        .map(|(release, place)| Selection::Version {
            year: place.year.clone(),
            month: place.month.clone(),
            version: release.version.clone(),
        })
}

/// The release history: chip rows that narrow a newest-first list of
/// versions, each with its date and notes. Starts on Latest. Renders just the
/// content block (chips + list), so wrap it in your own page or sheet chrome.
///
/// `data` defaults to the compiled-in changelog; pass a fetched
/// [`HttpChangelog`] to render a server-updated copy. `anchor` is a card id
/// (see [`anchor_id`]) to select down to; an unknown one leaves Latest.
/// `link_versions` makes each version heading a link to its card.
#[component]
pub fn Changelog(
    #[props(default = HttpChangelog::current())] data: HttpChangelog,
    #[props(default)] anchor: Option<String>,
    #[props(default)] link_versions: bool,
) -> Element {
    let listed = listed(&data);
    let mut selection = use_signal(|| Selection::Latest);
    let target = anchor
        .as_deref()
        .and_then(|fragment| anchor_selection(fragment, &listed));
    use_effect(use_reactive!(|target| {
        if let Some(target) = target {
            selection.set(target);
        }
    }));

    let current = selection();
    let rows = chip_rows(&listed, &current);
    let shown = visible(&listed, data.upcoming.len(), &current);
    // Included in each card's key so switching filters remounts the visible
    // cards, replaying their ease-in animation.
    let filter_key = format!("{current:?}");
    // Upcoming entries render first with an "Upcoming" badge; the first released
    // entry after them is the "Latest".
    let upcoming_count = data.upcoming.len();

    rsx! {
        div { class: "changelog-filters",
            for (r , row) in rows.into_iter().enumerate() {
                div { key: "{r}", class: "changelog-filter",
                    for chip in row {
                        {
                            let Chip { label, selected, target } = chip;
                            rsx! {
                                button {
                                    key: "{label}",
                                    class: if selected { "chip selected" } else { "chip" },
                                    onclick: move |_| selection.set(target.clone()),
                                    "{label}"
                                }
                            }
                        }
                    }
                }
            }
        }
        div { class: "changelog-list",
            for (i , ((release , _) , show)) in listed.iter().zip(shown).enumerate() {
                if show {
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

    /// Dec 2026 into Jan 2027, with an upcoming release.
    fn two_years() -> HttpChangelog {
        changelog(
            &["1.16.0"],
            &[
                ("1.15.1", "Jan 20, 2027"),
                ("1.15.0", "Jan 4, 2027"),
                ("1.14.2", "Dec 30, 2026"),
                ("1.14.1", "Dec 9, 2026"),
                ("1.14.0", "Nov 2, 2026"),
            ],
        )
    }

    fn year(year: &str) -> Selection {
        Selection::Year(year.to_string())
    }

    fn month(year: &str, month: &str) -> Selection {
        Selection::Month {
            year: year.to_string(),
            month: month.to_string(),
        }
    }

    fn version(year: &str, month: &str, version: &str) -> Selection {
        Selection::Version {
            year: year.to_string(),
            month: month.to_string(),
            version: version.to_string(),
        }
    }

    fn labels(row: &[Chip]) -> Vec<&str> {
        row.iter().map(|chip| chip.label.as_str()).collect()
    }

    fn lit(rows: &[Vec<Chip>]) -> Vec<&str> {
        rows.iter()
            .flatten()
            .filter(|chip| chip.selected)
            .map(|chip| chip.label.as_str())
            .collect()
    }

    fn shown<'a>(data: &'a HttpChangelog, selection: &Selection) -> Vec<&'a str> {
        let listed = listed(data);
        listed
            .iter()
            .zip(visible(&listed, data.upcoming.len(), selection))
            .filter(|(_, show)| *show)
            .map(|((release, _), _)| release.version.as_str())
            .collect()
    }

    fn rows(data: &HttpChangelog, selection: &Selection) -> Vec<Vec<Chip>> {
        chip_rows(&listed(data), selection)
    }

    #[test]
    fn latest_shows_upcoming_and_the_newest_release() {
        let data = two_years();
        let rows = rows(&data, &Selection::Latest);
        assert_eq!(rows.len(), 1);
        assert_eq!(labels(&rows[0]), ["Latest", "2027", "2026"]);
        assert_eq!(lit(&rows), ["Latest"]);
        assert_eq!(shown(&data, &Selection::Latest), ["1.16.0", "1.15.1"]);
    }

    #[test]
    fn one_year_still_gets_a_year_chip() {
        let data = HttpChangelog::current();
        let chips = rows(&data, &Selection::Latest);
        assert_eq!(labels(&chips[0]), ["Latest", "2026"]);
        assert_eq!(
            shown(&data, &Selection::Latest).len(),
            data.upcoming.len() + 1
        );
    }

    #[test]
    fn a_year_adds_its_months_and_shows_the_year() {
        let data = two_years();
        let rows = rows(&data, &year("2026"));
        assert_eq!(rows.len(), 2);
        assert_eq!(labels(&rows[1]), ["Dec", "Nov"]);
        assert_eq!(lit(&rows), ["2026"]);
        assert_eq!(shown(&data, &year("2026")), ["1.14.2", "1.14.1", "1.14.0"]);
    }

    #[test]
    fn upcoming_counts_in_the_newest_month() {
        let data = two_years();
        let rows = rows(&data, &month("2027", "2027-01"));
        assert_eq!(rows.len(), 3);
        assert_eq!(labels(&rows[1]), ["Jan"]);
        assert_eq!(labels(&rows[2]), ["1.16.0", "1.15.1", "1.15.0"]);
        assert_eq!(lit(&rows), ["2027", "Jan"]);
    }

    #[test]
    fn a_month_adds_its_versions_and_shows_the_month() {
        let data = two_years();
        let selection = month("2026", "2026-12");
        let rows = rows(&data, &selection);
        assert_eq!(labels(&rows[2]), ["1.14.2", "1.14.1"]);
        assert_eq!(shown(&data, &selection), ["1.14.2", "1.14.1"]);
    }

    #[test]
    fn a_version_shows_one_card() {
        let data = two_years();
        let selection = version("2026", "2026-12", "1.14.1");
        let rows = rows(&data, &selection);
        assert_eq!(rows.len(), 3);
        assert_eq!(lit(&rows), ["2026", "Dec", "1.14.1"]);
        assert_eq!(shown(&data, &selection), ["1.14.1"]);
    }

    #[test]
    fn higher_chips_clear_the_levels_below() {
        let data = two_years();
        let rows = rows(&data, &version("2026", "2026-12", "1.14.1"));
        // Latest, the other year, the lit year, the other month.
        assert_eq!(rows[0][0].target, Selection::Latest);
        assert_eq!(rows[0][1].target, year("2027"));
        assert_eq!(rows[0][2].target, year("2026"));
        assert_eq!(rows[1][1].target, month("2026", "2026-11"));
        assert_eq!(rows[1][0].target, month("2026", "2026-12"));
    }

    #[test]
    fn the_deepest_lit_chip_goes_up_a_level() {
        let data = two_years();
        let rows_v = rows(&data, &version("2026", "2026-12", "1.14.1"));
        assert_eq!(rows_v[2][1].target, month("2026", "2026-12"));
        let rows_m = rows(&data, &month("2026", "2026-12"));
        assert_eq!(rows_m[1][0].target, year("2026"));
        let rows_y = rows(&data, &year("2026"));
        assert_eq!(rows_y[0][2].target, Selection::Latest);
        let rows_l = rows(&data, &Selection::Latest);
        assert_eq!(rows_l[0][0].target, Selection::Latest);
    }

    #[test]
    fn the_compiled_in_history_has_its_months() {
        let data = HttpChangelog::current();
        let rows = rows(&data, &year("2026"));
        assert_eq!(labels(&rows[1]), ["Oct", "Sep", "Aug", "Jul", "Jun"]);
    }

    #[test]
    fn unreadable_dates_go_under_a_last_earlier_chip() {
        let data = changelog(
            &[],
            &[
                ("1.3.0", "Jan 3, 2027"),
                ("1.2.0", "someday"),
                ("1.1.0", "Sep 3, 2026"),
            ],
        );
        let chips = rows(&data, &year(EARLIER));
        assert_eq!(labels(&chips[0]), ["Latest", "2027", "2026", "Earlier"]);
        assert_eq!(shown(&data, &year(EARLIER)), ["1.2.0"]);
        let readable = rows(&two_years(), &Selection::Latest);
        assert!(!labels(&readable[0]).contains(&"Earlier"));
    }

    #[test]
    fn anchor_ids_are_selector_safe() {
        assert_eq!(anchor_id("1.12.1"), "v1-12-1");
        assert_eq!(anchor_id("1.0.10"), "v1-0-10");
    }

    #[test]
    fn an_anchor_selects_down_to_its_release() {
        let data = two_years();
        let all = listed(&data);
        assert_eq!(
            anchor_selection("#v1-14-1", &all),
            Some(version("2026", "2026-12", "1.14.1"))
        );
        assert_eq!(
            anchor_selection("v1-16-0", &all),
            Some(version("2027", "2027-01", "1.16.0"))
        );
        let current = HttpChangelog::current();
        assert_eq!(
            anchor_selection("v1-9-1", &listed(&current)),
            Some(version("2026", "2026-08", "1.9.1"))
        );
    }

    #[test]
    fn an_unknown_anchor_selects_nothing() {
        let data = HttpChangelog::current();
        let all = listed(&data);
        assert_eq!(anchor_selection("", &all), None);
        assert_eq!(anchor_selection("v9-9-9", &all), None);
        assert_eq!(anchor_selection("1.12.1", &all), None);
    }
}
