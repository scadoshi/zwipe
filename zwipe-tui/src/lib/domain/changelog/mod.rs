//! The changelog: the copy fetched once at launch, and how the Changelog screen files its releases under years, months and minor versions, as the app's `components::changelog` does.
//!
//! Releases list upcoming first, then shipped newest first. Each takes the year and month of its day; an upcoming one with no day takes the newest shipped release's, and one with no readable day files under `Earlier`. The filter narrows from `Latest` to a year, a month, then a minor version.

use chrono::NaiveDate;
use zwipe_core::http::contracts::changelog::{HttpChangelog, HttpRelease};

/// The changelog as fetched at launch, logged in or not. Fetched once per launch, with no retry.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum ChangelogCache {
    #[default]
    Loading,
    Loaded(HttpChangelog),
    Failed,
}

/// The year and month key a release with no readable day files under.
const EARLIER: &str = "earlier";

/// The label of the `Earlier` chip.
pub(crate) const EARLIER_LABEL: &str = "Earlier";

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
pub(crate) enum Tag {
    Upcoming,
    Latest,
}

impl Tag {
    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::Upcoming => "Upcoming",
            Self::Latest => "Latest",
        }
    }
}

/// What the chips have narrowed the list to.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub(crate) enum Filter {
    #[default]
    Latest,
    Year(String),
    /// A month by its key, within its year.
    Month {
        year: String,
        month: String,
    },
    /// A minor version within a month.
    Minor {
        year: String,
        month: String,
        minor: String,
    },
}

/// One listed release, where it files, and its tag.
#[derive(Debug, Clone)]
pub(crate) struct Listed<'log> {
    pub(crate) release: &'log HttpRelease,
    pub(crate) tag: Option<Tag>,
    place: Place,
}

/// `1.12` for `1.12.3`; a version with no second dot is its own minor.
pub(crate) fn minor_of(version: &str) -> &str {
    let mut dots = version.match_indices('.');
    match (dots.next(), dots.next()) {
        (Some(_), Some((second, _))) => version.get(..second).unwrap_or(version),
        _ => version,
    }
}

/// Every release in listed order with its place and tag.
pub(crate) fn listed(changelog: &HttpChangelog) -> Vec<Listed<'_>> {
    let newest_shipped = changelog
        .releases
        .first()
        .and_then(HttpRelease::released_on);
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

/// The year chips, newest first, `Earlier` last when anything files there: each a key and a label.
pub(crate) fn years(listed: &[Listed<'_>]) -> Vec<(String, String)> {
    let mut years: Vec<(String, String)> = Vec::new();
    for item in listed.iter().filter(|l| l.place.year != EARLIER) {
        push_new(
            &mut years,
            (item.place.year.clone(), item.place.year.clone()),
        );
    }
    years.sort_by(|a, b| b.0.cmp(&a.0));
    if listed.iter().any(|l| l.place.year == EARLIER) {
        years.push((EARLIER.to_owned(), EARLIER_LABEL.to_owned()));
    }
    years
}

/// The month chips in `year`, newest first: each a key and a label.
pub(crate) fn months(listed: &[Listed<'_>], year: &str) -> Vec<(String, String)> {
    let mut months: Vec<(String, String)> = Vec::new();
    for item in listed.iter().filter(|l| l.place.year == year) {
        push_new(
            &mut months,
            (item.place.month.clone(), item.place.label.clone()),
        );
    }
    months.sort_by(|a, b| b.0.cmp(&a.0));
    months
}

/// The minor version chips in `month`, in listed order.
pub(crate) fn minors(listed: &[Listed<'_>], month: &str) -> Vec<String> {
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
pub(crate) fn visible<'log>(listed: &[Listed<'log>], filter: &Filter) -> Vec<Listed<'log>> {
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
}
