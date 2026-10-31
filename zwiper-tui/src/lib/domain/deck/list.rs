//! The deck list's Show filter and Group by, ported from the app's `list.rs`.

use std::collections::BTreeSet;

use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::deck::{DeckProfile, deck_tag};

/// How the deck list folds into sections.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum GroupBy {
    /// One section headed "All".
    #[default]
    None,
    Format,
    Color,
    Tag,
}

impl GroupBy {
    /// Every mode in the app's chip order.
    pub(crate) const ALL: [Self; 4] = [Self::None, Self::Format, Self::Color, Self::Tag];

    pub(crate) fn label(self) -> &'static str {
        match self {
            Self::None => "None",
            Self::Format => "Format",
            Self::Color => "Color",
            Self::Tag => "Tag",
        }
    }
}

/// What the Show row lets through: decks holding every picked color, and, when any tag is picked, at least one of them.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub(crate) struct Show {
    pub(crate) colors: BTreeSet<Color>,
    /// Tag keys: deck tag slugs and other tags' display names.
    pub(crate) tags: BTreeSet<String>,
}

impl Show {
    /// Whether nothing is picked, which the app shows as "All".
    pub(crate) fn is_all(&self) -> bool {
        self.colors.is_empty() && self.tags.is_empty()
    }

    pub(crate) fn clear(&mut self) {
        self.colors.clear();
        self.tags.clear();
    }

    pub(crate) fn toggle_color(&mut self, color: Color) {
        if !self.colors.remove(&color) {
            self.colors.insert(color);
        }
    }

    pub(crate) fn toggle_tag(&mut self, key: &str) {
        if !self.tags.remove(key) {
            self.tags.insert(key.to_owned());
        }
    }

    /// Whether `profile` passes: every picked color is in its identity, and it carries a picked tag when any is picked.
    pub(crate) fn matches(&self, profile: &DeckProfile) -> bool {
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
pub(crate) fn identity_colors(profile: &DeckProfile) -> Vec<Color> {
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
pub(crate) fn tag_labels(profile: &DeckProfile) -> Vec<(String, String)> {
    profile
        .tags
        .iter()
        .map(|t| (t.clone(), deck_tag::deck_tag_label(t)))
        .chain(
            profile
                .other_tags
                .iter()
                .map(|t| (t.display_name().to_owned(), t.display_name().to_owned())),
        )
        .collect()
}

/// Every distinct tag across `profiles` as a key and a label, sorted by label.
pub(crate) fn all_tags(profiles: &[DeckProfile]) -> Vec<(String, String)> {
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
pub(crate) struct DeckGroup {
    /// Empty for a color group with colors, which shows pips instead.
    pub(crate) header: String,
    pub(crate) pips: Option<Vec<Color>>,
    pub(crate) decks: Vec<DeckProfile>,
}

impl DeckGroup {
    /// What the group folds under: its header, or for a pip group its colors' letters, such as `WU`.
    pub(crate) fn key(&self) -> String {
        if !self.header.is_empty() {
            return self.header.clone();
        }
        self.pips
            .as_ref()
            .map(|pips| pips.iter().map(Color::to_short_name).collect::<String>())
            .unwrap_or_default()
    }
}

/// The decks that pass `show`, sorted by name ignoring case.
pub(crate) fn shown(profiles: &[DeckProfile], show: &Show) -> Vec<DeckProfile> {
    let mut decks: Vec<DeckProfile> = profiles
        .iter()
        .filter(|p| show.matches(p))
        .cloned()
        .collect();
    decks.sort_by_cached_key(|d| d.name.to_lowercase());
    decks
}

/// Folds `profiles` into sections the way the app does for `by`.
pub(crate) fn group(profiles: &[DeckProfile], by: GroupBy) -> Vec<DeckGroup> {
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
        let all: Vec<String> = shown(&decks, &show)
            .iter()
            .map(|d| d.name.to_string())
            .collect();
        assert_eq!(all, ["A", "b", "c"]);
        show.toggle_color(Color::Red);
        assert_eq!(shown(&decks, &show).len(), 1);
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
