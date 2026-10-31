//! The slow-changing public lists the filter pickers, the oracle tag
//! dictionary and the Config screen read, as the app's catalog cache keeps
//! them: keywords, keyword reminders, oracle words, card types, artists,
//! sets, card roles, oracle tags, Universes Beyond franchises and deck tags.
//!
//! A list is fetched once and stays fresh for [`CATALOG_TTL`]. After that the
//! next read fetches it again while the stale list stays readable, and a
//! failed refresh keeps the stale list. One that failed with nothing to fall
//! back on is fetched again on the next read. One fetch per list is in
//! flight at a time. The client does the fetching and passes in the time.

use std::collections::HashMap;

use chrono::{DateTime, Duration, Utc};
use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::oracle_tag::OracleTag;
use zwipe_core::domain::card::scryfall_data::universe::UbFranchiseView;
use zwipe_core::domain::deck::DeckTagView;

/// How long a fetched list stays fresh: one day, the nightly sync's cadence
/// and the API's edge cache.
pub const CATALOG_TTL: Duration = Duration::hours(24);

/// One list's state.
#[derive(Debug, Clone, PartialEq)]
pub struct Slot<T> {
    /// The last good list and when it was fetched.
    loaded: Option<(T, DateTime<Utc>)>,
    /// Whether the last fetch failed with no list to fall back on.
    failed: bool,
    /// Whether a fetch is in flight.
    fetching: bool,
}

impl<T> Default for Slot<T> {
    fn default() -> Self {
        Self {
            loaded: None,
            failed: false,
            fetching: false,
        }
    }
}

impl<T> Slot<T> {
    /// The list once loaded, fresh or stale.
    #[must_use]
    pub fn ready(&self) -> Option<&T> {
        self.loaded.as_ref().map(|(data, _)| data)
    }

    /// Whether the last fetch failed with no list to fall back on.
    #[must_use]
    pub fn is_failed(&self) -> bool {
        self.failed
    }

    /// Whether the list is past [`CATALOG_TTL`] at `now`. A deadline past
    /// the last representable instant never comes.
    fn is_stale(&self, now: DateTime<Utc>) -> bool {
        self.loaded.as_ref().is_some_and(|(_, fetched_at)| {
            fetched_at
                .checked_add_signed(CATALOG_TTL)
                .is_some_and(|deadline| now >= deadline)
        })
    }

    /// Marks a fetch in flight when the list needs one at `now` (never
    /// loaded, failed or stale) and none is running, and says whether it did.
    pub fn claim(&mut self, now: DateTime<Utc>) -> bool {
        let wanted = !self.fetching && (self.loaded.is_none() || self.is_stale(now));
        if wanted {
            self.fetching = true;
        }
        wanted
    }

    /// Stores a fetch's answer, fetched at `now`: the list, or `None` when
    /// the fetch failed, which keeps any list already loaded.
    pub fn settle(&mut self, answer: Option<T>, now: DateTime<Utc>) {
        self.fetching = false;
        match answer {
            Some(data) => {
                self.loaded = Some((data, now));
                self.failed = false;
            }
            None => self.failed = self.loaded.is_none(),
        }
    }
}

/// Which list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Keyword abilities.
    Keywords,
    /// Keyword reminder text by keyword.
    KeywordReminders,
    /// Normalized oracle-text words.
    OracleWords,
    /// Card types.
    CardTypes,
    /// Artist names.
    Artists,
    /// Set names.
    Sets,
    /// The card role catalog.
    CardRoles,
    /// The oracle tag catalog.
    OracleTags,
    /// Universes Beyond franchises, the exceptions picker's list.
    UbFranchises,
    /// The deck tag catalog, the one list that needs a session.
    DeckTags,
}

impl Kind {
    /// The lists the card filter's pickers search.
    pub const FILTER: [Self; 7] = [
        Self::Keywords,
        Self::OracleWords,
        Self::CardTypes,
        Self::Artists,
        Self::Sets,
        Self::CardRoles,
        Self::OracleTags,
    ];

    /// The public lists, fetched at launch with no session.
    pub const PUBLIC: [Self; 9] = [
        Self::Artists,
        Self::Sets,
        Self::Keywords,
        Self::KeywordReminders,
        Self::OracleWords,
        Self::CardTypes,
        Self::CardRoles,
        Self::OracleTags,
        Self::UbFranchises,
    ];
}

/// The cached lists.
#[derive(Debug, Default)]
pub struct Catalog {
    /// Keyword abilities.
    pub keywords: Slot<Vec<String>>,
    /// Keyword reminder text by keyword.
    pub keyword_reminders: Slot<HashMap<String, String>>,
    /// Normalized oracle-text words.
    pub oracle_words: Slot<Vec<String>>,
    /// Card types.
    pub card_types: Slot<Vec<String>>,
    /// Artist names.
    pub artists: Slot<Vec<String>>,
    /// Set names.
    pub sets: Slot<Vec<String>>,
    /// The card role catalog.
    pub card_roles: Slot<Vec<CardRoleView>>,
    /// The oracle tag catalog.
    pub oracle_tags: Slot<Vec<OracleTag>>,
    /// Universes Beyond franchises.
    pub ub_franchises: Slot<Vec<UbFranchiseView>>,
    /// The deck tag catalog.
    pub deck_tags: Slot<Vec<DeckTagView>>,
}

impl Catalog {
    /// Whether any public list failed to load. Deck tags stay out: they load
    /// only once there is a session, so an unloaded one is no failure.
    #[must_use]
    pub fn any_public_failed(&self) -> bool {
        self.keywords.is_failed()
            || self.keyword_reminders.is_failed()
            || self.oracle_words.is_failed()
            || self.card_types.is_failed()
            || self.artists.is_failed()
            || self.sets.is_failed()
            || self.card_roles.is_failed()
            || self.oracle_tags.is_failed()
            || self.ub_franchises.is_failed()
    }

    /// Marks each of `kinds` that needs a fetch at `now` as in flight, and
    /// returns those for the client to fetch.
    pub fn claim(&mut self, kinds: &[Kind], now: DateTime<Utc>) -> Vec<Kind> {
        kinds
            .iter()
            .copied()
            .filter(|&kind| match kind {
                Kind::Keywords => self.keywords.claim(now),
                Kind::KeywordReminders => self.keyword_reminders.claim(now),
                Kind::OracleWords => self.oracle_words.claim(now),
                Kind::CardTypes => self.card_types.claim(now),
                Kind::Artists => self.artists.claim(now),
                Kind::Sets => self.sets.claim(now),
                Kind::CardRoles => self.card_roles.claim(now),
                Kind::OracleTags => self.oracle_tags.claim(now),
                Kind::UbFranchises => self.ub_franchises.claim(now),
                Kind::DeckTags => self.deck_tags.claim(now),
            })
            .collect()
    }

    /// A plain word list by kind; `None` for the other lists, or before it
    /// loads.
    #[must_use]
    pub fn words(&self, kind: Kind) -> Option<&[String]> {
        match kind {
            Kind::Keywords => self.keywords.ready(),
            Kind::OracleWords => self.oracle_words.ready(),
            Kind::CardTypes => self.card_types.ready(),
            Kind::Artists => self.artists.ready(),
            Kind::Sets => self.sets.ready(),
            Kind::KeywordReminders
            | Kind::CardRoles
            | Kind::OracleTags
            | Kind::UbFranchises
            | Kind::DeckTags => None,
        }
        .map(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use chrono::TimeZone;

    use super::*;

    fn at(hour: u32) -> DateTime<Utc> {
        Utc.with_ymd_and_hms(2026, 10, 30, hour, 0, 0).unwrap()
    }

    #[test]
    fn claim_fetches_missing_and_failed_lists_once() {
        let mut catalog = Catalog::default();
        assert_eq!(
            catalog.claim(&[Kind::Keywords, Kind::Sets], at(1)),
            [Kind::Keywords, Kind::Sets]
        );
        assert!(
            catalog.claim(&[Kind::Keywords], at(1)).is_empty(),
            "already in flight"
        );
        catalog.keywords.settle(None, at(1));
        catalog.sets.settle(Some(vec!["Alpha".to_owned()]), at(1));
        assert!(catalog.any_public_failed());
        assert_eq!(catalog.claim(&Kind::FILTER, at(2)).len(), 6, "all but sets");
        assert_eq!(catalog.words(Kind::Sets), Some(&["Alpha".to_owned()][..]));
        assert_eq!(catalog.words(Kind::CardRoles), None);
    }

    #[test]
    fn a_list_refreshes_after_a_day_and_a_failed_refresh_keeps_it() {
        let mut slot: Slot<Vec<String>> = Slot::default();
        assert!(slot.claim(at(0)));
        slot.settle(Some(vec!["Rebecca Guay".to_owned()]), at(0));
        assert!(!slot.claim(at(23)), "fresh for a day");
        let next_day = at(0).checked_add_signed(CATALOG_TTL).unwrap();
        assert!(slot.claim(next_day), "stale after a day");
        assert!(!slot.claim(next_day), "one fetch at a time");
        assert_eq!(slot.ready().map(Vec::len), Some(1), "stale stays readable");
        slot.settle(None, next_day);
        assert!(!slot.is_failed(), "a failed refresh keeps the last list");
        assert_eq!(slot.ready().map(Vec::len), Some(1));
    }

    #[test]
    fn a_fetch_at_the_end_of_time_is_never_stale() {
        let mut slot: Slot<Vec<String>> = Slot::default();
        slot.claim(at(0));
        slot.settle(Some(Vec::new()), DateTime::<Utc>::MAX_UTC);
        assert!(!slot.claim(DateTime::<Utc>::MAX_UTC));
    }

    #[test]
    fn deck_tags_never_count_as_a_public_failure() {
        let mut catalog = Catalog::default();
        catalog.claim(&[Kind::DeckTags], at(1));
        catalog.deck_tags.settle(None, at(1));
        assert!(!catalog.any_public_failed());
        assert_eq!(Kind::PUBLIC.len(), 9);
    }
}
