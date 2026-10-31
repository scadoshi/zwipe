//! The changelog: the copy fetched once at launch, and how the Changelog screen files its releases under years, months and minor versions, as the app's `components::changelog` does.
//!
//! Releases list upcoming first, then shipped newest first. Each takes the year and month of its day; an upcoming one with no day takes the newest shipped release's that has one, and one with no readable day files under `Earlier`. The filter narrows from `Latest` to a year, a month, then a minor version.

use chrono::NaiveDate;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

/// The changelog as fetched at launch, logged in or not. Fetched once per launch, with no retry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum ChangelogCache {
    /// The launch fetch is still out.
    #[default]
    Loading,
    /// The fetched changelog.
    Loaded(HttpChangelog),
    /// The fetch failed.
    Failed,
}

/// The year and month key a release with no readable day files under.
const EARLIER: &str = "earlier";

/// The label of the `Earlier` chip.
pub const EARLIER_LABEL: &str = "Earlier";

/// Where a release files: its year (`2026`), month key (`2026-10`) and month label (`Oct`).
#[derive(Debug, Clone, PartialEq, Eq)]
struct Place {
    year: String,
    month: String,
    label: String,
}

impl Place {
    fn of(day: Option<NaiveDate>) -> Self {
        day.map_or_else(
            || Self {
                year: EARLIER.to_owned(),
                month: EARLIER.to_owned(),
                label: EARLIER_LABEL.to_owned(),
            },
            |day| Self {
                year: day.format("%Y").to_string(),
                month: day.format("%Y-%m").to_string(),
                label: day.format("%b").to_string(),
            },
        )
    }
}

/// The tag a card carries.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tag {
    /// A release not shipped yet.
    Upcoming,
    /// The newest shipped release.
    Latest,
}

impl Tag {
    /// The tag's words.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Upcoming => "Upcoming",
            Self::Latest => "Latest",
        }
    }
}

/// What the chips have narrowed the list to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Filter {
    /// The upcoming release and the newest shipped one, the default.
    #[default]
    Latest,
    /// A whole year by its key, or `Earlier`.
    Year(String),
    /// A month by its key, within its year.
    Month {
        /// The year's key.
        year: String,
        /// The month's key, such as `2026-10`.
        month: String,
    },
    /// A minor version within a month.
    Minor {
        /// The year's key.
        year: String,
        /// The month's key, such as `2026-10`.
        month: String,
        /// The minor version, such as `1.12`.
        minor: String,
    },
}

/// One listed release, where it files, and its tag.
#[derive(Debug, Clone)]
pub struct Listed<'log> {
    /// The release.
    pub release: &'log HttpRelease,
    /// Its tag, if any.
    pub tag: Option<Tag>,
    place: Place,
}

/// `1.12` for `1.12.3`; a version with no second dot is its own minor.
#[must_use]
pub fn minor_of(version: &str) -> &str {
    let mut dots = version.match_indices('.');
    match (dots.next(), dots.next()) {
        (Some(_), Some((second, _))) => version.get(..second).unwrap_or(version),
        _ => version,
    }
}

/// Every release in listed order with its place and tag.
pub fn listed(changelog: &HttpChangelog) -> Vec<Listed<'_>> {
    let newest_shipped = changelog.releases.iter().find_map(HttpRelease::released_on);
    let upcoming = changelog.upcoming.iter().map(|release| Listed {
        release,
        tag: Some(Tag::Upcoming),
        place: Place::of(release.released_on().or(newest_shipped)),
    });
    let shipped = changelog
        .releases
        .iter()
        .enumerate()
        .map(|(i, release)| Listed {
            release,
            tag: (i == 0).then_some(Tag::Latest),
            place: Place::of(release.released_on()),
        });
    upcoming.chain(shipped).collect()
}

/// Pushes `item` onto `list` unless it is already there, keeping first-seen order.
fn push_new(list: &mut Vec<(String, String)>, item: (String, String)) {
    if !list.contains(&item) {
        list.push(item);
    }
}

/// The year chips in first-seen order, which is newest first while releases list newest first, `Earlier` last when anything files there: each a key and a label.
#[must_use]
pub fn years(listed: &[Listed<'_>]) -> Vec<(String, String)> {
    let mut years: Vec<(String, String)> = Vec::new();
    for item in listed.iter().filter(|l| l.place.year != EARLIER) {
        push_new(
            &mut years,
            (item.place.year.clone(), item.place.year.clone()),
        );
    }
    if listed.iter().any(|l| l.place.year == EARLIER) {
        years.push((EARLIER.to_owned(), EARLIER_LABEL.to_owned()));
    }
    years
}

/// The month chips in `year` in first-seen order, newest first while releases list newest first: each a key and a label.
#[must_use]
pub fn months(listed: &[Listed<'_>], year: &str) -> Vec<(String, String)> {
    let mut months: Vec<(String, String)> = Vec::new();
    for item in listed.iter().filter(|l| l.place.year == year) {
        push_new(
            &mut months,
            (item.place.month.clone(), item.place.label.clone()),
        );
    }
    months
}

/// The minor version chips in `month`, in listed order.
#[must_use]
pub fn minors(listed: &[Listed<'_>], month: &str) -> Vec<String> {
    let mut minors: Vec<String> = Vec::new();
    for item in listed.iter().filter(|l| l.place.month == month) {
        let minor = minor_of(&item.release.version).to_owned();
        if !minors.contains(&minor) {
            minors.push(minor);
        }
    }
    minors
}

/// The releases `filter` shows: for `Latest`, every upcoming one and the newest shipped one.
#[must_use]
pub fn visible<'log>(listed: &[Listed<'log>], filter: &Filter) -> Vec<Listed<'log>> {
    listed
        .iter()
        .filter(|l| match filter {
            Filter::Latest => l.tag.is_some(),
            Filter::Year(year) => l.place.year == *year,
            Filter::Month { month, .. } => l.place.month == *month,
            Filter::Minor { month, minor, .. } => {
                l.place.month == *month && minor_of(&l.release.version) == minor
            }
        })
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn release(version: &str, day: Option<&str>) -> HttpRelease {
        HttpRelease {
            version: version.to_owned(),
            date: String::new(),
            released_on: day.map(|d| NaiveDate::parse_from_str(d, "%Y-%m-%d").unwrap()),
            entries: vec![format!("Entry for {version}")],
        }
    }

    fn changelog() -> HttpChangelog {
        HttpChangelog {
            upcoming: vec![release("1.16.0", None)],
            releases: vec![
                release("1.15.1", Some("2026-10-27")),
                release("1.15.0", Some("2026-10-20")),
                release("1.14.0", Some("2026-09-02")),
                release("1.0.0", Some("2025-12-01")),
                HttpRelease {
                    date: "someday".to_owned(),
                    ..release("0.9", None)
                },
            ],
        }
    }

    #[test]
    fn latest_shows_the_upcoming_and_the_newest_shipped() {
        let log = changelog();
        let listed = listed(&log);
        let shown = visible(&listed, &Filter::Latest);
        let versions: Vec<&str> = shown.iter().map(|l| l.release.version.as_str()).collect();
        assert_eq!(versions, ["1.16.0", "1.15.1"]);
        assert_eq!(shown[0].tag, Some(Tag::Upcoming));
        assert_eq!(shown[1].tag.map(Tag::label), Some("Latest"));
    }

    #[test]
    fn years_months_and_minors_file_newest_first_with_earlier_last() {
        let log = changelog();
        let listed = listed(&log);
        let labels = |chips: Vec<(String, String)>| -> Vec<String> {
            chips.into_iter().map(|(_, l)| l).collect()
        };
        assert_eq!(labels(years(&listed)), ["2026", "2025", "Earlier"]);
        assert_eq!(labels(months(&listed, "2026")), ["Oct", "Sep"]);
        assert_eq!(
            minors(&listed, "2026-10"),
            ["1.16", "1.15"],
            "upcoming takes the newest day"
        );
        let october = Filter::Minor {
            year: "2026".to_owned(),
            month: "2026-10".to_owned(),
            minor: "1.15".to_owned(),
        };
        assert_eq!(visible(&listed, &october).len(), 2);
        assert_eq!(visible(&listed, &Filter::Year(EARLIER.to_owned())).len(), 1);
    }

    #[test]
    fn a_minor_is_the_first_two_parts() {
        assert_eq!(minor_of("1.12.3"), "1.12");
        assert_eq!(minor_of("1.12"), "1.12");
        assert_eq!(minor_of("2"), "2");
    }

    #[test]
    fn upcoming_files_under_the_newest_shipped_release_with_a_day() {
        let log = HttpChangelog {
            upcoming: vec![release("1.17.0", None)],
            releases: vec![
                release("1.16.1", None),
                release("1.16.0", Some("2026-10-20")),
            ],
        };
        let listed = listed(&log);
        assert_eq!(
            listed[0].place.month, "2026-10",
            "as in the app, not Earlier"
        );
        assert_eq!(listed[1].place.month, EARLIER);
    }

    #[test]
    fn chips_keep_first_seen_order_as_the_app_does() {
        let log = HttpChangelog {
            upcoming: Vec::new(),
            releases: vec![
                release("1.0.0", Some("2025-12-01")),
                release("1.1.0", Some("2026-01-05")),
            ],
        };
        let listed = listed(&log);
        let keys: Vec<String> = years(&listed).into_iter().map(|(k, _)| k).collect();
        assert_eq!(keys, ["2025", "2026"]);
    }
}
