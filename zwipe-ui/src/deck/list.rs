//! The deck list's Show filter and Group by, ported from the app's `list.rs`.

use std::cmp::Ordering;
use std::collections::BTreeSet;

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
use zwipe_core::domain::deck::deck_tag::deck_tag_label;
use zwipe_core::domain::deck::{DeckProfile, Format};

/// How the deck list folds into sections.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub enum GroupBy {
    /// One section headed "All".
    #[default]
    None,
    /// By format.
    Format,
    /// By color identity.
    Color,
    /// By deck tag, a deck under each of its tags.
    Tag,
}

impl GroupBy {
    /// Every mode in the app's chip order.
    pub const ALL: [Self; 4] = [Self::None, Self::Format, Self::Color, Self::Tag];

    /// The chip's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Format => "Format",
            Self::Color => "Color",
            Self::Tag => "Tag",
        }
    }
}

/// What the deck list sorts on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortBy {
    /// The deck's name, ignoring case.
    #[default]
    Name,
    /// The main board's card count.
    Size,
    /// The format's name, with decks that have none after the rest either way.
    Format,
    /// When the deck last changed, oldest first ascending. Only offered when the server sends it.
    LastChanged,
    /// The total price in USD, cheapest first ascending, with unpriced decks after the rest either way. Only offered when the server sends it.
    Price,
}

impl SortBy {
    /// Every key in chip order.
    pub const ALL: [Self; 5] = [
        Self::Name,
        Self::Size,
        Self::Format,
        Self::LastChanged,
        Self::Price,
    ];

    /// The chip's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Name => "Name",
            Self::Size => "Size",
            Self::Format => "Format",
            Self::LastChanged => "Last changed",
            Self::Price => "Price",
        }
    }

    /// The keys `profiles` can sort on, in chip order: Name, Size and Format always, Last changed and Price when any deck carries them, which a server older than those fields never sends.
    #[must_use]
    pub fn offered(profiles: &[DeckProfile]) -> Vec<Self> {
        Self::ALL
            .into_iter()
            .filter(|by| match by {
                Self::Name | Self::Size | Self::Format => true,
                Self::LastChanged => profiles.iter().any(|p| p.updated_at.is_some()),
                Self::Price => profiles.iter().any(|p| p.total_price_usd.is_some()),
            })
            .collect()
    }

    /// The key [`Sort::key`] stores it under.
    fn slug(self) -> &'static str {
        match self {
            Self::Name => "name",
            Self::Size => "size",
            Self::Format => "format",
            Self::LastChanged => "changed",
            Self::Price => "price",
        }
    }
}

/// The deck list's order: a key and a direction. Decks the key ties on fall back to their names, A to Z.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    /// What to sort on.
    pub by: SortBy,
    /// Which way.
    pub direction: SortDirection,
}

impl Sort {
    /// The sort as one string, such as `size-descending`, for a client that stores it as text.
    #[must_use]
    pub fn key(self) -> String {
        let direction = match self.direction {
            SortDirection::Ascending => "ascending",
            SortDirection::Descending => "descending",
        };
        format!("{}-{direction}", self.by.slug())
    }

    /// The sort [`Self::key`] gave `key`, or `None` for anything else.
    #[must_use]
    pub fn from_key(key: &str) -> Option<Self> {
        let (by, direction) = key.split_once('-')?;
        let by = SortBy::ALL.into_iter().find(|b| b.slug() == by)?;
        let direction = match direction {
            "ascending" => SortDirection::Ascending,
            "descending" => SortDirection::Descending,
            _ => return None,
        };
        Some(Self { by, direction })
    }

    /// This sort when `offered` holds its key, else Name in the same direction: a remembered key the decks can't sort on.
    #[must_use]
    pub fn within(self, offered: &[SortBy]) -> Self {
        if offered.contains(&self.by) {
            self
        } else {
            Self {
                by: SortBy::Name,
                ..self
            }
        }
    }

    /// How `a` and `b` order under this sort.
    #[must_use]
    pub fn compare(self, a: &DeckProfile, b: &DeckProfile) -> Ordering {
        let directed = |ordering: Ordering| match self.direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        };
        let by_name = || a.name.to_lowercase().cmp(&b.name.to_lowercase());
        match self.by {
            SortBy::Name => directed(by_name()),
            SortBy::Size => directed(a.card_count.cmp(&b.card_count)),
            SortBy::Format => present_first(
                a.format.as_ref().map(Format::display_name),
                b.format.as_ref().map(Format::display_name),
                |x, y| directed(x.cmp(y)),
            ),
            SortBy::LastChanged => {
                present_first(a.updated_at, b.updated_at, |x, y| directed(x.cmp(y)))
            }
            SortBy::Price => present_first(a.total_price_usd, b.total_price_usd, |x, y| {
                directed(x.total_cmp(y))
            }),
        }
        .then_with(by_name)
    }
}

/// Orders two optional keys: both present compare with `compare`, and a missing one goes after a present one whichever way the sort runs.
fn present_first<T>(a: Option<T>, b: Option<T>, compare: impl Fn(&T, &T) -> Ordering) -> Ordering {
    match (a, b) {
        (Some(x), Some(y)) => compare(&x, &y),
        (Some(_), None) => Ordering::Less,
        (None, Some(_)) => Ordering::Greater,
        (None, None) => Ordering::Equal,
    }
}

/// What the Show row lets through: decks holding every picked color, and, when any tag is picked, at least one of them.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct Show {
    /// The picked colors.
    pub colors: BTreeSet<Color>,
    /// Tag keys: deck tag slugs and other tags' display names.
    pub tags: BTreeSet<String>,
}

impl Show {
    /// Whether nothing is picked, which the app shows as "All".
    #[must_use]
    pub fn is_all(&self) -> bool {
        self.colors.is_empty() && self.tags.is_empty()
    }

    /// Unpicks everything, back to All.
    pub fn clear(&mut self) {
        self.colors.clear();
        self.tags.clear();
    }

    /// Picks or unpicks a color.
    pub fn toggle_color(&mut self, color: Color) {
        if !self.colors.remove(&color) {
            self.colors.insert(color);
        }
    }

    /// Picks or unpicks a tag by its key.
    pub fn toggle_tag(&mut self, key: &str) {
        if !self.tags.remove(key) {
            self.tags.insert(key.to_owned());
        }
    }

    /// Whether `profile` passes: every picked color is in its identity, and it carries a picked tag when any is picked.
    #[must_use]
    pub fn matches(&self, profile: &DeckProfile) -> bool {
        let identity = identity_colors(profile);
        if !self.colors.iter().all(|c| identity.contains(c)) {
            return false;
        }
        self.tags.is_empty()
            || tag_labels(profile)
                .iter()
                .any(|(key, _)| self.tags.contains(key))
    }
}

/// The deck's identity colors in WUBRG order, unknown letters dropped.
#[must_use]
pub fn identity_colors(profile: &DeckProfile) -> Vec<Color> {
    let present: BTreeSet<Color> = profile
        .color_identity
        .iter()
        .filter_map(|s| Color::try_from(s.as_str()).ok())
        .collect();
    Color::all()
        .into_iter()
        .filter(|c| present.contains(c))
        .collect()
}

/// Every tag on the deck as a key and a label: deck tags keyed by slug, other tags by display name.
#[must_use]
pub fn tag_labels(profile: &DeckProfile) -> Vec<(String, String)> {
    profile
        .tags
        .iter()
        .map(|t| (t.clone(), deck_tag_label(t)))
        .chain(
            profile
                .other_tags
                .iter()
                .map(|t| (t.display_name().to_owned(), t.display_name().to_owned())),
        )
        .collect()
}

/// Every distinct tag across `profiles` as a key and a label, sorted by label.
pub fn all_tags(profiles: &[DeckProfile]) -> Vec<(String, String)> {
    let mut tags: Vec<(String, String)> = Vec::new();
    for (key, label) in profiles.iter().flat_map(tag_labels) {
        if !tags.iter().any(|(k, _)| *k == key) {
            tags.push((key, label));
        }
    }
    tags.sort_by(|a, b| a.1.cmp(&b.1));
    tags
}

/// One section of the list: a header, or pips for a color group, and its decks.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckGroup {
    /// Empty for a color group with colors, which shows pips instead.
    pub header: String,
    /// Pips in place of the header, for a color group.
    pub pips: Option<Vec<Color>>,
    /// The group's decks.
    pub decks: Vec<DeckProfile>,
}

impl DeckGroup {
    /// What the group folds under: its header, or for a pip group its colors' letters, such as `WU`.
    #[must_use]
    pub fn key(&self) -> String {
        if !self.header.is_empty() {
            return self.header.clone();
        }
        self.pips
            .as_ref()
            .map_or_default(|pips| pips.iter().map(Color::to_short_name).collect::<String>())
    }
}

/// The decks that pass `show`, in `sort`'s order.
#[must_use]
pub fn shown(profiles: &[DeckProfile], show: &Show, sort: Sort) -> Vec<DeckProfile> {
    let mut decks: Vec<DeckProfile> = profiles
        .iter()
        .filter(|p| show.matches(p))
        .cloned()
        .collect();
    decks.sort_by(|a, b| sort.compare(a, b));
    decks
}

/// Folds `profiles` into sections the way the app does for `by`.
#[must_use]
pub fn group(profiles: &[DeckProfile], by: GroupBy) -> Vec<DeckGroup> {
    match by {
        GroupBy::None => vec![DeckGroup {
            header: "All".to_owned(),
            pips: None,
            decks: profiles.to_vec(),
        }],
        GroupBy::Format => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            for profile in profiles {
                let header = profile
                    .format
                    .map_or_else(|| "No format".to_owned(), |f| f.display_name().to_owned());
                push_into(&mut groups, header, None, profile);
            }
            groups.sort_by_key(|g| (g.header == "No format", g.header.clone()));
            groups
        }
        GroupBy::Color => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            for profile in profiles {
                let pips = identity_colors(profile);
                match groups
                    .iter_mut()
                    .find(|g| g.pips.as_deref() == Some(pips.as_slice()))
                {
                    Some(group) => group.decks.push(profile.clone()),
                    None => groups.push(DeckGroup {
                        header: if pips.is_empty() {
                            "Colorless".to_owned()
                        } else {
                            String::new()
                        },
                        pips: Some(pips),
                        decks: vec![profile.clone()],
                    }),
                }
            }
            groups.sort_by_key(|g| {
                let pips = g.pips.clone().unwrap_or_default();
                let order: Vec<usize> = pips
                    .iter()
                    .map(|c| Color::all().iter().position(|a| a == c).unwrap_or(0))
                    .collect();
                (pips.is_empty(), pips.len(), order)
            });
            groups
        }
        GroupBy::Tag => {
            let mut groups: Vec<DeckGroup> = Vec::new();
            let mut untagged: Vec<DeckProfile> = Vec::new();
            for profile in profiles {
                let labels = tag_labels(profile);
                if labels.is_empty() {
                    untagged.push(profile.clone());
                }
                for (_, label) in labels {
                    push_into(&mut groups, label, None, profile);
                }
            }
            groups.sort_by_key(|g| g.header.clone());
            if !untagged.is_empty() {
                groups.push(DeckGroup {
                    header: "Untagged".to_owned(),
                    pips: None,
                    decks: untagged,
                });
            }
            groups
        }
    }
}

/// Adds `profile` to the group headed `header`, starting it when there is none.
fn push_into(
    groups: &mut Vec<DeckGroup>,
    header: String,
    pips: Option<Vec<Color>>,
    profile: &DeckProfile,
) {
    match groups.iter_mut().find(|g| g.header == header) {
        Some(group) => group.decks.push(profile.clone()),
        None => groups.push(DeckGroup {
            header,
            pips,
            decks: vec![profile.clone()],
        }),
    }
}

#[cfg(test)]
mod tests {
    use chrono::DateTime;
    use zwipe_core::domain::deck::{DeckOtherTag, Format};

    use super::*;
    use crate::test_support::deck_profile;

    fn colored(name: &str, identity: &[&str]) -> DeckProfile {
        let mut profile = deck_profile(name);
        profile.color_identity = identity.iter().map(|&c| c.to_owned()).collect();
        profile
    }

    fn names(group: &DeckGroup) -> Vec<String> {
        group.decks.iter().map(|d| d.name.to_string()).collect()
    }

    #[test]
    fn colors_must_all_be_in_the_identity() {
        let mut show = Show::default();
        assert!(show.is_all());
        show.toggle_color(Color::Green);
        show.toggle_color(Color::Blue);
        assert!(show.matches(&colored("Simic", &["G", "U"])));
        assert!(!show.matches(&colored("Mono green", &["G"])));
        show.toggle_color(Color::Blue);
        assert!(show.matches(&colored("Mono green", &["G"])));
        show.clear();
        assert!(show.is_all());
    }

    #[test]
    fn any_picked_tag_lets_a_deck_through() {
        let mut tokens = deck_profile("Tokens");
        tokens.tags = vec!["tokens".to_owned()];
        let mut jank = deck_profile("Jank");
        jank.other_tags = vec![DeckOtherTag::Jank];
        let mut show = Show::default();
        show.toggle_tag("tokens");
        show.toggle_tag("Jank");
        assert!(show.matches(&tokens));
        assert!(show.matches(&jank));
        assert!(!show.matches(&deck_profile("Plain")));
        show.toggle_tag("Jank");
        assert!(!show.matches(&jank));
    }

    #[test]
    fn the_tag_row_lists_each_tag_once_by_label() {
        let mut a = deck_profile("A");
        a.tags = vec!["tokens".to_owned()];
        a.other_tags = vec![DeckOtherTag::Budget];
        let mut b = deck_profile("B");
        b.tags = vec!["tokens".to_owned(), "aristocrats".to_owned()];
        let labels: Vec<String> = all_tags(&[a, b]).into_iter().map(|(_, l)| l).collect();
        assert_eq!(labels, ["Aristocrats", "Budget", "Tokens"]);
    }

    #[test]
    fn shown_decks_sort_by_name_ignoring_case() {
        let decks = [colored("b", &[]), colored("A", &[]), colored("c", &["R"])];
        let mut show = Show::default();
        let all: Vec<String> = shown(&decks, &show, Sort::default())
            .iter()
            .map(|d| d.name.to_string())
            .collect();
        assert_eq!(all, ["A", "b", "c"]);
        show.toggle_color(Color::Red);
        assert_eq!(shown(&decks, &show, Sort::default()).len(), 1);
    }

    fn sorted(decks: &[DeckProfile], by: SortBy, direction: SortDirection) -> Vec<String> {
        shown(decks, &Show::default(), Sort { by, direction })
            .iter()
            .map(|d| d.name.to_string())
            .collect()
    }

    #[test]
    fn decks_sort_by_name_size_or_format_either_way() {
        use SortDirection::{Ascending, Descending};
        let mut big = deck_profile("big");
        big.card_count = 100;
        big.format = Some(Format::Commander);
        let mut small = deck_profile("Small");
        small.card_count = 40;
        small.format = Some(Format::Modern);
        let mut also_small = deck_profile("also small");
        also_small.card_count = 40;
        let decks = [big, small, also_small];

        assert_eq!(
            sorted(&decks, SortBy::Name, Ascending),
            ["also small", "big", "Small"]
        );
        assert_eq!(
            sorted(&decks, SortBy::Name, Descending),
            ["Small", "big", "also small"]
        );
        assert_eq!(
            sorted(&decks, SortBy::Size, Ascending),
            ["also small", "Small", "big"],
            "a tie falls back to the name"
        );
        assert_eq!(
            sorted(&decks, SortBy::Size, Descending),
            ["big", "also small", "Small"]
        );
        assert_eq!(
            sorted(&decks, SortBy::Format, Ascending),
            ["big", "Small", "also small"],
            "no format goes last"
        );
        assert_eq!(
            sorted(&decks, SortBy::Format, Descending),
            ["Small", "big", "also small"],
            "and stays last descending"
        );
    }

    #[test]
    fn last_changed_and_price_show_only_when_the_server_sends_them() {
        use SortDirection::{Ascending, Descending};
        let mut old = deck_profile("Old");
        let mut new = deck_profile("New");
        let mut unpriced = deck_profile("Unpriced");
        let base = [old.clone(), new.clone(), unpriced.clone()];
        assert_eq!(
            SortBy::offered(&base),
            [SortBy::Name, SortBy::Size, SortBy::Format]
        );
        let remembered = Sort {
            by: SortBy::Price,
            direction: SortDirection::Descending,
        };
        assert_eq!(
            remembered.within(&SortBy::offered(&base)),
            Sort {
                by: SortBy::Name,
                direction: SortDirection::Descending
            },
            "an old server falls back to Name"
        );

        old.updated_at = DateTime::from_timestamp(1_000, 0);
        new.updated_at = DateTime::from_timestamp(2_000, 0);
        old.total_price_usd = Some(12.5);
        new.total_price_usd = Some(3.0);
        unpriced.updated_at = DateTime::from_timestamp(1_500, 0);
        let decks = [old, new, unpriced];
        assert_eq!(SortBy::offered(&decks), SortBy::ALL);
        assert_eq!(remembered.within(&SortBy::offered(&decks)), remembered);
        assert_eq!(
            sorted(&decks, SortBy::LastChanged, Ascending),
            ["Old", "Unpriced", "New"]
        );
        assert_eq!(
            sorted(&decks, SortBy::LastChanged, Descending),
            ["New", "Unpriced", "Old"]
        );
        assert_eq!(
            sorted(&decks, SortBy::Price, Ascending),
            ["New", "Old", "Unpriced"]
        );
        assert_eq!(
            sorted(&decks, SortBy::Price, Descending),
            ["Old", "New", "Unpriced"],
            "an unpriced deck stays last"
        );
    }

    #[test]
    fn a_sort_round_trips_through_its_key() {
        for by in SortBy::ALL {
            for direction in [SortDirection::Ascending, SortDirection::Descending] {
                let sort = Sort { by, direction };
                assert_eq!(Sort::from_key(&sort.key()), Some(sort), "{}", sort.key());
            }
        }
        assert_eq!(Sort::default().key(), "name-ascending");
        assert_eq!(Sort::from_key("rarity-ascending"), None);
        assert_eq!(Sort::from_key("name"), None);
        assert_eq!(Sort::from_key("name-sideways"), None);
    }

    #[test]
    fn no_grouping_is_one_group_called_all() {
        let groups = group(&[deck_profile("A")], GroupBy::None);
        assert_eq!(groups.len(), 1);
        assert_eq!(groups[0].header, "All");
    }

    #[test]
    fn format_groups_sort_by_name_with_no_format_last() {
        let mut modern = deck_profile("M");
        modern.format = Some(Format::Modern);
        let mut commander = deck_profile("C");
        commander.format = Some(Format::Commander);
        let groups = group(
            &[deck_profile("None"), modern, commander.clone(), commander],
            GroupBy::Format,
        );
        let headers: Vec<&str> = groups.iter().map(|g| g.header.as_str()).collect();
        assert_eq!(headers, ["Commander", "Modern", "No format"]);
        assert_eq!(groups[0].decks.len(), 2);
    }

    #[test]
    fn color_groups_put_fewer_colors_first_and_colorless_last() {
        let groups = group(
            &[
                colored("Colorless", &[]),
                colored("Simic", &["U", "G"]),
                colored("Red", &["R"]),
                colored("White", &["W"]),
                colored("Azorius", &["W", "U"]),
            ],
            GroupBy::Color,
        );
        let keys: Vec<String> = groups.iter().map(DeckGroup::key).collect();
        assert_eq!(keys, ["W", "R", "WU", "UG", "Colorless"]);
        assert_eq!(groups[2].header, "", "pip groups have no header text");
    }

    #[test]
    fn tag_groups_repeat_decks_and_end_with_untagged() {
        let mut both = deck_profile("Both");
        both.tags = vec!["tokens".to_owned()];
        both.other_tags = vec![DeckOtherTag::Budget];
        let groups = group(&[deck_profile("Plain"), both], GroupBy::Tag);
        let headers: Vec<&str> = groups.iter().map(|g| g.header.as_str()).collect();
        assert_eq!(headers, ["Budget", "Tokens", "Untagged"]);
        assert_eq!(names(&groups[0]), ["Both"]);
        assert_eq!(names(&groups[2]), ["Plain"]);
    }
}
