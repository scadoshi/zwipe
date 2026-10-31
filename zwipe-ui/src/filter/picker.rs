//! The searchable pickers: where their options come from, how a query finds them, and how a pick or a removal lands in the builder.
//!
//! On the deck cards view and Remove, keywords, oracle words, type-line words, artists and sets come from the deck's own cards, so a picker only offers what the deck has. On Add they come from the public catalogs. Oracle tags always come from the catalog.

use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::oracle_tag::{OracleTag, search_oracle_tags};
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::stop_words::{ORACLE_STOP_WORDS, TYPE_STOP_WORDS};

use super::fields::{MatchGroup, SideGroup};
use super::sections::Modes;
use crate::stores::catalog::{Catalog, Kind};

/// Results shown for keywords, oracle words, type-line words and oracle tags.
const WIDE_CAP: usize = 8;

/// Results shown for artists and sets.
const NARROW_CAP: usize = 5;

/// A searchable picker.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum PickerField {
    /// A group's includes, under its Any or All chip.
    Include(MatchGroup),
    /// A group's excludes.
    Exclude(MatchGroup),
    /// A list under an Include or Exclude chip.
    Side(SideGroup),
}

impl PickerField {
    /// The empty search box's placeholder.
    #[must_use]
    pub fn placeholder(self) -> &'static str {
        match self {
            Self::Include(MatchGroup::Tags) | Self::Exclude(MatchGroup::Tags) => {
                "Search all oracle tags"
            }
            _ => "Type to search",
        }
    }

    fn cap(self) -> usize {
        match self {
            Self::Side(_) => NARROW_CAP,
            _ => WIDE_CAP,
        }
    }

    /// The catalog list behind the picker, or `None` for oracle tags, which search their own way.
    fn kind(self) -> Option<Kind> {
        let group = match self {
            Self::Include(group) | Self::Exclude(group) => group,
            Self::Side(SideGroup::Artist) => return Some(Kind::Artists),
            Self::Side(SideGroup::Set | SideGroup::Rarity) => return Some(Kind::Sets),
        };
        match group {
            MatchGroup::OracleWords => Some(Kind::OracleWords),
            MatchGroup::Keywords => Some(Kind::Keywords),
            MatchGroup::TypeLine => Some(Kind::CardTypes),
            _ => None,
        }
    }

    /// The values picked, shown as removable pills.
    #[must_use]
    pub fn picked(self, b: &CardQueryBuilder) -> Vec<String> {
        match self {
            Self::Include(group) => group.values(b),
            Self::Exclude(group) => group.exclude().values(b),
            Self::Side(group) => group.values(b),
        }
    }

    /// Values the results leave out: for include and exclude pairs, both sides; else the picked ones.
    fn taken(self, b: &CardQueryBuilder) -> Vec<String> {
        match self {
            Self::Include(MatchGroup::Tags) | Self::Side(_) => self.picked(b),
            Self::Include(group) | Self::Exclude(group) => {
                let mut taken = group.values(b);
                taken.extend(group.exclude().values(b));
                taken
            }
        }
    }

    /// Writes `values` as the picker's picks, keeping the mode chip's choice.
    pub fn write(self, b: &mut CardQueryBuilder, modes: &Modes, values: Vec<String>) {
        match self {
            Self::Include(group) => {
                let mode = modes.match_of(b, group);
                group.write(b, mode, values);
            }
            Self::Exclude(group) => group.exclude().write(b, values),
            Self::Side(group) => {
                let side = modes.side_of(b, group);
                group.write(b, side, values);
            }
        }
    }

    /// Adds `value` to the picks.
    pub fn pick(self, b: &mut CardQueryBuilder, modes: &Modes, value: &str) {
        let mut values = self.picked(b);
        if !values.iter().any(|v| v == value) {
            values.push(value.to_owned());
        }
        self.write(b, modes, values);
    }

    /// Drops `value` from the picks.
    pub fn unpick(self, b: &mut CardQueryBuilder, modes: &Modes, value: &str) {
        let values = self.picked(b).into_iter().filter(|v| v != value).collect();
        self.write(b, modes, values);
    }

    /// The results for `query`: nothing for an empty query, else matches from the source minus taken values, capped.
    pub fn results(
        self,
        b: &CardQueryBuilder,
        query: &str,
        sources: &Sources,
        catalog: &Catalog,
    ) -> Vec<String> {
        if query.is_empty() {
            return Vec::new();
        }
        let taken = self.taken(b);
        let Some(kind) = self.kind() else {
            let tags: &[OracleTag] = catalog.oracle_tags.ready().map_or(&[], Vec::as_slice);
            return search_oracle_tags(tags, query)
                .into_iter()
                .map(|t| t.slug)
                .filter(|slug| !taken.contains(slug))
                .take(self.cap())
                .collect();
        };
        let source = sources.list(kind).or_else(|| catalog.words(kind));
        let query = query.to_lowercase();
        source
            .unwrap_or_default()
            .iter()
            .filter(|v| v.to_lowercase().contains(&query) && !taken.contains(v))
            .take(self.cap())
            .cloned()
            .collect()
    }
}

/// Where the word pickers draw from: the deck's own cards, or the catalogs.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub enum Sources {
    /// The public catalogs, on Add.
    #[default]
    Catalog,
    /// The deck's own cards.
    Deck(DeckLists),
}

impl Sources {
    /// Lists drawn from `cards`.
    #[must_use]
    pub fn deck(cards: &[Card]) -> Self {
        Self::Deck(DeckLists {
            keywords: extract_keywords(cards),
            oracle_words: extract_oracle_words(cards),
            type_words: extract_type_words(cards),
            artists: extract_artists(cards),
            sets: extract_sets(cards),
        })
    }

    /// The deck's list for `kind`; `None` when drawing from the catalogs.
    fn list(&self, kind: Kind) -> Option<&[String]> {
        let Self::Deck(lists) = self else {
            return None;
        };
        let list = match kind {
            Kind::Keywords => &lists.keywords,
            Kind::OracleWords => &lists.oracle_words,
            Kind::CardTypes => &lists.type_words,
            Kind::Artists => &lists.artists,
            Kind::Sets => &lists.sets,
            Kind::KeywordReminders
            | Kind::CardRoles
            | Kind::OracleTags
            | Kind::UbFranchises
            | Kind::DeckTags => return None,
        };
        Some(list)
    }

    /// The catalogs this host needs fetched: all of them on Add, only roles and tags for a deck.
    #[must_use]
    pub fn needs(&self) -> &'static [Kind] {
        match self {
            Self::Catalog => &Kind::FILTER,
            Self::Deck(_) => &[Kind::CardRoles, Kind::OracleTags],
        }
    }
}

/// Picker lists drawn from one deck's cards.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct DeckLists {
    keywords: Vec<String>,
    oracle_words: Vec<String>,
    type_words: Vec<String>,
    artists: Vec<String>,
    sets: Vec<String>,
}

/// Distinct type-line words: split on whitespace, stray punctuation trimmed, lowercased, stop words dropped.
pub fn extract_type_words(cards: &[Card]) -> Vec<String> {
    let mut words: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.type_line.as_deref())
        .flat_map(str::split_whitespace)
        .map(|w| w.trim_matches(|c: char| matches!(c, ':' | '-' | '?' | ',' | ' ')))
        .map(str::to_lowercase)
        .filter(|w| !w.is_empty() && !TYPE_STOP_WORDS.contains(&w.as_str()))
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Distinct oracle-text words: ASCII letters only, lowercased, stop words dropped.
pub fn extract_oracle_words(cards: &[Card]) -> Vec<String> {
    let mut words: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.oracle_text.as_deref())
        .flat_map(str::split_whitespace)
        .map(|w| {
            w.chars()
                .filter(char::is_ascii_alphabetic)
                .collect::<String>()
                .to_lowercase()
        })
        .filter(|w| !w.is_empty() && !ORACLE_STOP_WORDS.contains(&w.as_str()))
        .collect();
    words.sort();
    words.dedup();
    words
}

/// Distinct keywords, trimmed and lowercased.
#[must_use]
pub fn extract_keywords(cards: &[Card]) -> Vec<String> {
    let mut keywords: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.keywords.as_ref())
        .flatten()
        .map(|k| k.trim().to_lowercase())
        .filter(|k| !k.is_empty())
        .collect();
    keywords.sort();
    keywords.dedup();
    keywords
}

/// Distinct artist names.
#[must_use]
pub fn extract_artists(cards: &[Card]) -> Vec<String> {
    let mut artists: Vec<String> = cards
        .iter()
        .filter_map(|c| c.scryfall_data.artist.clone())
        .filter(|a| !a.is_empty())
        .collect();
    artists.sort();
    artists.dedup();
    artists
}

/// Distinct set names.
#[must_use]
pub fn extract_sets(cards: &[Card]) -> Vec<String> {
    let mut sets: Vec<String> = cards
        .iter()
        .map(|c| c.scryfall_data.set_name.clone())
        .collect();
    sets.sort();
    sets.dedup();
    sets
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::filter::fields::Match;
    use crate::test_support::card;

    fn strings(values: &[&str]) -> Vec<String> {
        values.iter().map(|&v| v.to_owned()).collect()
    }

    fn deck_cards() -> Vec<Card> {
        let mut bolt = card("Bolt");
        bolt.scryfall_data.type_line = Some("Instant — Lesson".to_owned());
        bolt.scryfall_data.oracle_text = Some("Deal 3 damage to any target.".to_owned());
        bolt.scryfall_data.keywords = Some(strings(&[" Flying ", "Haste"]));
        bolt.scryfall_data.artist = Some("Christopher Rush".to_owned());
        let mut elf = card("Elf");
        elf.scryfall_data.type_line = Some("Creature - Elf Druid".to_owned());
        elf.scryfall_data.artist = Some(String::new());
        vec![bolt, elf]
    }

    #[test]
    fn deck_lists_extract_like_the_app() {
        let cards = deck_cards();
        assert_eq!(extract_keywords(&cards), strings(&["flying", "haste"]));
        assert!(extract_type_words(&cards).contains(&"druid".to_owned()));
        assert!(!extract_type_words(&cards).contains(&"-".to_owned()));
        assert!(extract_oracle_words(&cards).contains(&"damage".to_owned()));
        assert!(!extract_oracle_words(&cards).contains(&"3".to_owned()));
        assert_eq!(extract_artists(&cards), strings(&["Christopher Rush"]));
        assert_eq!(extract_sets(&cards).len(), 1, "both share a set");
    }

    #[test]
    fn results_search_the_deck_minus_taken_values_and_cap() {
        let sources = Sources::deck(&deck_cards());
        let catalog = Catalog::default();
        let mut b = CardQueryBuilder::default();
        let keywords = PickerField::Include(MatchGroup::Keywords);
        assert_eq!(keywords.results(&b, "", &sources, &catalog).len(), 0);
        assert_eq!(
            keywords.results(&b, "A", &sources, &catalog),
            strings(&["haste"])
        );
        let modes = Modes::default();
        keywords.pick(&mut b, &modes, "haste");
        assert_eq!(keywords.results(&b, "a", &sources, &catalog).len(), 0);
        assert!(
            PickerField::Exclude(MatchGroup::Keywords)
                .results(&b, "a", &sources, &catalog)
                .is_empty(),
            "an include is not offered as an exclude"
        );
        keywords.unpick(&mut b, &modes, "haste");
        assert!(b.is_empty());
    }

    #[test]
    fn results_fall_back_to_the_catalog_and_cap_at_five_for_artists() {
        let mut catalog = Catalog::default();
        catalog.artists.settle(
            Some((0..9).map(|i| format!("Artist {i}")).collect()),
            chrono::Utc::now(),
        );
        let b = CardQueryBuilder::default();
        let artist = PickerField::Side(SideGroup::Artist);
        assert_eq!(
            artist
                .results(&b, "artist", &Sources::Catalog, &catalog)
                .len(),
            5
        );
        assert_eq!(Sources::Catalog.needs().len(), 7);
        assert_eq!(Sources::deck(&[]).needs().len(), 2);
    }

    #[test]
    fn tags_search_by_rank_and_picks_follow_the_mode() {
        let mut catalog = Catalog::default();
        let tag = |slug: &str, description: &str| OracleTag {
            slug: slug.to_owned(),
            label: slug.to_owned(),
            description: Some(description.to_owned()),
            parent_slugs: Vec::new(),
            curated: false,
        };
        catalog.oracle_tags.settle(
            Some(vec![
                tag("removal", "kills things"),
                tag("ramp", "more mana"),
            ]),
            chrono::Utc::now(),
        );
        let mut b = CardQueryBuilder::default();
        let include = PickerField::Include(MatchGroup::Tags);
        assert_eq!(
            include.results(&b, "mana", &Sources::Catalog, &catalog),
            strings(&["ramp"])
        );
        let mut modes = Modes::default();
        modes.flip(
            &mut b,
            crate::filter::sections::ModeChip::Match(MatchGroup::Tags),
        );
        include.pick(&mut b, &modes, "ramp");
        assert_eq!(b.oracle_tags_contains_all(), Some(&strings(&["ramp"])[..]));
        assert_eq!(modes.match_of(&b, MatchGroup::Tags), Match::All);
        assert_eq!(include.placeholder(), "Search all oracle tags");
    }
}
