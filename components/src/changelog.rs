//! Shared changelog rendering.
//!
//! The release history, rendered identically on the website (`zite`) and in the
//! app (`zwiper`). The data lives in `zwipe_core::content::changelog` (pure, so
//! the server can serve it too); this crate owns the rendering and styling so
//! the surfaces never drift. Each consumer wraps the two components in its
//! own chrome (a page with nav/footer on the web, the About screen in the app).
//! Styling is in `assets/magic.css`.
//!
//! The component takes the changelog as a prop, defaulting to the copy compiled
//! into the binary ([`HttpChangelog::current`]). `zite` renders that default;
//! `zwiper` fetches `/api/changelog` and feeds the fresh copy in, so new
//! entries appear without an app resubmit. Passing the same wire type either
//! way keeps one rendering path.
//!
//! The filing (where each release goes, which chips each row holds, what a
//! filter shows) is zwipe-ui's `changelog`, which every client shares; this
//! module draws it. The chips narrow the list as a tree. The first row is Latest and then each
//! year, newest first; picking a year adds a row of its months, and picking a
//! month adds a row of its minor versions (`1.12` for 1.12.0 to 1.12.2), so
//! the row stays short however many patches ship. Picking a minor shows its
//! releases in that month. Latest, the default, shows the upcoming
//! release and the newest shipped one. The upcoming release counts in the
//! newest shipped month. The day comes from [`HttpRelease::released_on`], which
//! also reads the display string an older server sends; a release whose day
//! cannot be read goes under a last "Earlier" chip rather than vanishing.
//!
//! Every card carries an id from its version ([`anchor_id`], `v1-12-1` for
//! 1.12.1). zite links each version heading to it and passes the URL fragment
//! in as `anchor`, which selects the chips down to that release's minor; zite
//! then scrolls the card into view.
//!
//! The chips and the releases are two components, [`ChangelogFilters`] and
//! [`ChangelogReleases`], sharing a `Signal<Filter>` the caller owns, so the
//! chips can sit in the caller's titled panel while the releases stack under
//! it on their own. Two looks share the one rendering path, [`ChangelogLook`]:
//! zite's floating cards and the app's panels. Each version's caret shuts
//! its notes, and one Collapse all over the list shuts or opens them all.

use std::ptr;

use dioxus::prelude::*;
use orangui::{CollapseAll, CollapseBody, CollapseCaret, use_collapse_set};
use zwipe_core::http::contracts::changelog::HttpChangelog;
pub use zwipe_ui::changelog::Filter;
use zwipe_ui::changelog::{Listed, Tag, listed, minors, months, visible, years};

/// The label of the chip that goes back to [`Filter::Latest`].
const LATEST_LABEL: &str = "Latest";

/// One chip: its label, whether it lights up, and what clicking it selects.
/// Clicking the deepest lit chip goes up a level; clicking any other chip
/// selects it, which clears the levels below.
#[derive(Debug, Clone, PartialEq, Eq)]
struct Chip {
    label: String,
    selected: bool,
    target: Filter,
}

/// The chip rows for a selection: Latest and the years (Earlier last, when
/// any release landed there), then the selected year's months, then the
/// selected month's minor versions.
fn chip_rows(listed: &[Listed<'_>], selection: &Filter) -> Vec<Vec<Chip>> {
    let chip = |label: String, own: Filter, lit: bool| Chip {
        label,
        selected: lit,
        target: if lit && own == *selection {
            own.parent()
        } else {
            own
        },
    };

    let mut first = vec![chip(
        LATEST_LABEL.to_string(),
        Filter::Latest,
        *selection == Filter::Latest,
    )];
    first.extend(years(listed).into_iter().map(|(year, label)| {
        let lit = selection.year() == Some(year.as_str());
        chip(label, Filter::Year(year), lit)
    }));
    let mut rows = vec![first];

    let Some(year) = selection.year() else {
        return rows;
    };
    rows.push(
        months(listed, year)
            .into_iter()
            .map(|(month, label)| {
                let lit = selection.month() == Some(month.as_str());
                let own = Filter::Month {
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
        minors(listed, month)
            .into_iter()
            .map(|minor| {
                let lit = selection.minor() == Some(minor.as_str());
                let own = Filter::Minor {
                    year: year.to_string(),
                    month: month.to_string(),
                    minor: minor.clone(),
                };
                chip(minor, own, lit)
            })
            .collect(),
    );
    rows
}

/// The element id of a release's card: `v` and the version with its dots as
/// dashes, `v1-12-1` for 1.12.1, so it needs no escaping in a CSS selector.
pub(crate) fn anchor_id(version: &str) -> String {
    format!("v{}", version.replace('.', "-"))
}

/// The selection down to the minor of the release `fragment` names, in that
/// release's month. Takes the fragment with or without its `#`. `None` when
/// no release has that anchor.
fn anchor_selection(fragment: &str, listed: &[Listed<'_>]) -> Option<Filter> {
    let fragment = fragment.strip_prefix('#').unwrap_or(fragment);
    listed
        .iter()
        .find(|item| anchor_id(&item.release.version) == fragment)
        .map(Listed::minor_filter)
}

/// Which chrome the changelog's two halves take.
#[derive(Clone, Copy, Debug, Default, PartialEq, Eq)]
pub enum ChangelogLook {
    /// zite's: the chip rows sit in the page's own hero panel and each
    /// release is a floating card.
    #[default]
    Site,
    /// The app's: the chip rows are a `settings-row` of the Changelog panel
    /// the caller wraps them in, and each release is a `settings-list` panel
    /// of its own, the version row as its header.
    App,
}

/// The chip rows that narrow the release list, drawn for the [`Filter`] in
/// `selection`, which [`ChangelogReleases`] reads. Starts wherever the caller
/// started the signal, [`Filter::Latest`] as a rule.
///
/// `data` defaults to the compiled-in changelog; pass a fetched
/// [`HttpChangelog`] to render a server-updated copy. `anchor` is a card id
/// (see [`anchor_id`]) to select down to; an unknown one leaves the
/// selection alone.
#[component]
pub fn ChangelogFilters(
    #[props(default = HttpChangelog::current())] data: HttpChangelog,
    mut selection: Signal<Filter>,
    #[props(default)] anchor: Option<String>,
    #[props(default)] look: ChangelogLook,
) -> Element {
    let listed = listed(&data);
    let target = anchor
        .as_deref()
        .and_then(|fragment| anchor_selection(fragment, &listed));
    use_effect(use_reactive!(|target| {
        if let Some(target) = target {
            selection.set(target);
        }
    }));

    let chip_rows = chip_rows(&listed, &selection());
    let class = match look {
        ChangelogLook::Site => "changelog-filters",
        ChangelogLook::App => "settings-row changelog-filters-row",
    };

    rsx! {
        div { class,
            for (r , row) in chip_rows.into_iter().enumerate() {
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
    }
}

/// The newest-first list of the releases `selection` shows, each with its
/// date and notes. `data` is the same copy [`ChangelogFilters`] draws.
/// `link_versions` makes each version heading a link to its card.
/// `page_size` caps the list at that many releases, with a "Show more"
/// button under it that reveals that many more; a new selection starts
/// back at one page. `None` shows every release.
#[component]
pub fn ChangelogReleases(
    #[props(default = HttpChangelog::current())] data: HttpChangelog,
    selection: Signal<Filter>,
    #[props(default)] link_versions: bool,
    #[props(default)] look: ChangelogLook,
    #[props(default)] page_size: Option<usize>,
) -> Element {
    // The selection the reveal was made under, and how many it reveals.
    let mut revealed = use_signal(|| None::<(Filter, usize)>);
    // Every version starts open; a caret or Collapse all shuts them.
    let collapse = use_collapse_set();
    let listed = listed(&data);
    let current = selection();
    let mut shown = visible(&listed, &current);
    let cap = page_size.map(|page| match revealed() {
        Some((filter, count)) if filter == current => count,
        _ => page,
    });
    let more = cap.filter(|&cap| shown.len() > cap);
    if let Some(cap) = cap {
        shown.truncate(cap);
    }
    // Every listed release keeps its slot, shown or not, so a filter change
    // toggles cards in place.
    let show = |item: &Listed<'_>| shown.iter().any(|s| ptr::eq(s.release, item.release));
    // Included in each card's key so switching filters remounts the visible
    // cards, replaying their ease-in animation.
    let filter_key = format!("{current:?}");
    let shown_versions: Vec<String> = shown.iter().map(|s| s.release.version.clone()).collect();
    let (list_class, release_class, header_class, body_class) = match look {
        ChangelogLook::Site => (
            "changelog-list",
            "changelog-card",
            "changelog-version-row",
            "changelog-bullets",
        ),
        ChangelogLook::App => (
            "changelog-panels",
            "settings-list changelog-panel",
            "settings-list-header changelog-version-row",
            "changelog-bullets changelog-panel-body",
        ),
    };

    rsx! {
        if !shown_versions.is_empty() {
            div { class: "changelog-collapse-row",
                CollapseAll { set: collapse, keys: shown_versions }
            }
        }
        div { class: list_class,
            for (item , release) in listed.iter().map(|item| (item, item.release)) {
                if show(item) {
                    div {
                        key: "{filter_key}-{release.version}",
                        id: anchor_id(&release.version),
                        class: release_class,
                        div { class: header_class,
                            CollapseCaret {
                                set: collapse,
                                section: release.version.clone(),
                                label: release.version.clone(),
                            }
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
                            if item.tag == Some(Tag::Upcoming) {
                                span { class: "status-tag status-doing", "Upcoming" }
                            } else if item.tag == Some(Tag::Latest) {
                                span { class: "status-tag status-done", "Latest" }
                            }
                        }
                        CollapseBody { set: collapse, section: release.version.clone(),
                            ul { class: body_class,
                                for entry in release.entries.iter() {
                                    li { "{entry}" }
                                }
                            }
                        }
                    }
                }
            }
            if let (Some(cap), Some(page)) = (more, page_size) {
                button {
                    class: "chip changelog-more",
                    onclick: move |_| revealed.set(Some((current.clone(), cap.saturating_add(page)))),
                    "Show more"
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::changelog::HttpRelease;
    use zwipe_ui::changelog::{EARLIER_LABEL, minor_of};

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

    fn year(year: &str) -> Filter {
        Filter::Year(year.to_string())
    }

    fn month(year: &str, month: &str) -> Filter {
        Filter::Month {
            year: year.to_string(),
            month: month.to_string(),
        }
    }

    fn minor(year: &str, month: &str, minor: &str) -> Filter {
        Filter::Minor {
            year: year.to_string(),
            month: month.to_string(),
            minor: minor.to_string(),
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

    fn shown<'a>(data: &'a HttpChangelog, selection: &Filter) -> Vec<&'a str> {
        visible(&listed(data), selection)
            .into_iter()
            .map(|item| item.release.version.as_str())
            .collect()
    }

    fn rows(data: &HttpChangelog, selection: &Filter) -> Vec<Vec<Chip>> {
        chip_rows(&listed(data), selection)
    }

    #[test]
    fn latest_shows_upcoming_and_the_newest_release() {
        let data = two_years();
        let rows = rows(&data, &Filter::Latest);
        assert_eq!(rows.len(), 1);
        assert_eq!(labels(&rows[0]), ["Latest", "2027", "2026"]);
        assert_eq!(lit(&rows), ["Latest"]);
        assert_eq!(shown(&data, &Filter::Latest), ["1.16.0", "1.15.1"]);
    }

    #[test]
    fn one_year_still_gets_a_year_chip() {
        let data = HttpChangelog::current();
        let chips = rows(&data, &Filter::Latest);
        assert_eq!(labels(&chips[0]), ["Latest", "2026"]);
        assert_eq!(shown(&data, &Filter::Latest).len(), data.upcoming.len() + 1);
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
        assert_eq!(labels(&rows[2]), ["1.16", "1.15"]);
        assert_eq!(lit(&rows), ["2027", "Jan"]);
    }

    #[test]
    fn a_month_adds_its_minors_and_shows_the_month() {
        let data = two_years();
        let selection = month("2026", "2026-12");
        let rows = rows(&data, &selection);
        assert_eq!(labels(&rows[2]), ["1.14"]);
        assert_eq!(shown(&data, &selection), ["1.14.2", "1.14.1"]);
    }

    #[test]
    fn a_minor_shows_its_patches_in_that_month() {
        let data = two_years();
        let selection = minor("2026", "2026-12", "1.14");
        let rows = rows(&data, &selection);
        assert_eq!(rows.len(), 3);
        assert_eq!(lit(&rows), ["2026", "Dec", "1.14"]);
        assert_eq!(shown(&data, &selection), ["1.14.2", "1.14.1"]);
        // 1.14.0 shipped in November, so it shows under November's 1.14.
        assert_eq!(shown(&data, &minor("2026", "2026-11", "1.14")), ["1.14.0"]);
    }

    #[test]
    fn the_compiled_in_months_list_each_minor_once() {
        let data = HttpChangelog::current();
        // The newest month holds the upcoming release, so its minor leads.
        let all = listed(&data);
        let newest = all[0].minor_filter().parent();
        let rows = chip_rows(&all, &newest);
        let minors = labels(&rows[2]);
        assert_eq!(minors.first(), Some(&minor_of(&all[0].release.version)));
        assert!(minors.iter().all(|label| label.matches('.').count() == 1));
        let unique: std::collections::BTreeSet<&str> = minors.iter().copied().collect();
        assert_eq!(unique.len(), minors.len());
    }

    #[test]
    fn higher_chips_clear_the_levels_below() {
        let data = two_years();
        let rows = rows(&data, &minor("2026", "2026-12", "1.14"));
        // Latest, the other year, the lit year, the other month.
        assert_eq!(rows[0][0].target, Filter::Latest);
        assert_eq!(rows[0][1].target, year("2027"));
        assert_eq!(rows[0][2].target, year("2026"));
        assert_eq!(rows[1][1].target, month("2026", "2026-11"));
        assert_eq!(rows[1][0].target, month("2026", "2026-12"));
    }

    #[test]
    fn the_deepest_lit_chip_goes_up_a_level() {
        let data = two_years();
        let rows_v = rows(&data, &minor("2026", "2026-12", "1.14"));
        assert_eq!(rows_v[2][0].target, month("2026", "2026-12"));
        let rows_m = rows(&data, &month("2026", "2026-12"));
        assert_eq!(rows_m[1][0].target, year("2026"));
        let rows_y = rows(&data, &year("2026"));
        assert_eq!(rows_y[0][2].target, Filter::Latest);
        let rows_l = rows(&data, &Filter::Latest);
        assert_eq!(rows_l[0][0].target, Filter::Latest);
    }

    #[test]
    fn the_compiled_in_history_has_its_months() {
        let data = HttpChangelog::current();
        let rows = rows(&data, &year("2026"));
        // Newer months join the front as releases ship; these are the history.
        let months = labels(&rows[1]);
        assert!(
            months.ends_with(&["Oct", "Sep", "Aug", "Jul", "Jun"]),
            "months {months:?}"
        );
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
        let earlier = rows(&data, &Filter::Latest)[0][3].target.clone();
        let chips = rows(&data, &earlier);
        assert_eq!(labels(&chips[0]), ["Latest", "2027", "2026", EARLIER_LABEL]);
        assert_eq!(shown(&data, &earlier), ["1.2.0"]);
        let readable = rows(&two_years(), &Filter::Latest);
        assert!(!labels(&readable[0]).contains(&EARLIER_LABEL));
    }

    #[test]
    fn anchor_ids_are_selector_safe() {
        assert_eq!(anchor_id("1.12.1"), "v1-12-1");
        assert_eq!(anchor_id("1.0.10"), "v1-0-10");
    }

    #[test]
    fn an_anchor_selects_down_to_its_minor() {
        let data = two_years();
        let all = listed(&data);
        assert_eq!(
            anchor_selection("#v1-14-1", &all),
            Some(minor("2026", "2026-12", "1.14"))
        );
        assert_eq!(
            anchor_selection("v1-16-0", &all),
            Some(minor("2027", "2027-01", "1.16"))
        );
        let current = HttpChangelog::current();
        assert_eq!(
            anchor_selection("v1-9-1", &listed(&current)),
            Some(minor("2026", "2026-08", "1.9"))
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
