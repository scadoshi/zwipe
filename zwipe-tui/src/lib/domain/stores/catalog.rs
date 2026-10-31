//! The public lists the filter pickers search when no deck supplies them: keywords, oracle words, card types, artists, sets, card roles and oracle tags.
//!
//! Each is fetched once. One that failed is fetched again the next time the filter sheet opens.

use zwipe_core::domain::card::card_role::CardRoleView;
use zwipe_core::domain::card::oracle_tag::OracleTag;

/// One list's state.
#[derive(Debug, Clone, PartialEq, Default)]
pub(crate) enum Slot<T> {
    #[default]
    Missing,
    Loading,
    Ready(T),
    Failed,
}

impl<T> Slot<T> {
    /// The list, once loaded.
    pub(crate) fn ready(&self) -> Option<&T> {
        match self {
            Self::Ready(value) => Some(value),
            _ => None,
        }
    }

    pub(crate) fn is_failed(&self) -> bool {
        matches!(self, Self::Failed)
    }

    /// Whether the list should be fetched: never asked for, or failed.
    fn wanted(&self) -> bool {
        matches!(self, Self::Missing | Self::Failed)
    }

    /// Marks the list as loading when it is wanted, and says whether it was.
    fn claim(&mut self) -> bool {
        let wanted = self.wanted();
        if wanted {
            *self = Self::Loading;
        }
        wanted
    }

    /// Stores a fetch's answer.
    pub(crate) fn settle(&mut self, answer: Option<T>) {
        *self = answer.map_or(Self::Failed, Self::Ready);
    }
}

/// Which list.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub(crate) enum Kind {
    Keywords,
    OracleWords,
    CardTypes,
    Artists,
    Sets,
    CardRoles,
    OracleTags,
}

impl Kind {
    /// Every list.
    pub(crate) const ALL: [Self; 7] = [
        Self::Keywords,
        Self::OracleWords,
        Self::CardTypes,
        Self::Artists,
        Self::Sets,
        Self::CardRoles,
        Self::OracleTags,
    ];
}

/// The cached lists.
#[derive(Debug, Default)]
pub(crate) struct Catalog {
    pub(crate) keywords: Slot<Vec<String>>,
    pub(crate) oracle_words: Slot<Vec<String>>,
    pub(crate) card_types: Slot<Vec<String>>,
    pub(crate) artists: Slot<Vec<String>>,
    pub(crate) sets: Slot<Vec<String>>,
    pub(crate) card_roles: Slot<Vec<CardRoleView>>,
    pub(crate) oracle_tags: Slot<Vec<OracleTag>>,
}

impl Catalog {
    /// Whether any list failed to load.
    pub(crate) fn any_failed(&self) -> bool {
        self.keywords.is_failed()
            || self.oracle_words.is_failed()
            || self.card_types.is_failed()
            || self.artists.is_failed()
            || self.sets.is_failed()
            || self.card_roles.is_failed()
            || self.oracle_tags.is_failed()
    }

    /// Marks each of `kinds` that is missing or failed as loading, and returns those to fetch.
    pub(crate) fn claim(&mut self, kinds: &[Kind]) -> Vec<Kind> {
        kinds
            .iter()
            .copied()
            .filter(|&kind| match kind {
                Kind::Keywords => self.keywords.claim(),
                Kind::OracleWords => self.oracle_words.claim(),
                Kind::CardTypes => self.card_types.claim(),
                Kind::Artists => self.artists.claim(),
                Kind::Sets => self.sets.claim(),
                Kind::CardRoles => self.card_roles.claim(),
                Kind::OracleTags => self.oracle_tags.claim(),
            })
            .collect()
    }

    /// A plain word list by kind; `None` for roles and tags, or before it loads.
    pub(crate) fn words(&self, kind: Kind) -> Option<&[String]> {
        match kind {
            Kind::Keywords => self.keywords.ready(),
            Kind::OracleWords => self.oracle_words.ready(),
            Kind::CardTypes => self.card_types.ready(),
            Kind::Artists => self.artists.ready(),
            Kind::Sets => self.sets.ready(),
            Kind::CardRoles | Kind::OracleTags => None,
        }
        .map(Vec::as_slice)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn claim_fetches_missing_and_failed_lists_once() {
        let mut catalog = Catalog::default();
        assert_eq!(
            catalog.claim(&[Kind::Keywords, Kind::Sets]),
            [Kind::Keywords, Kind::Sets]
        );
        assert!(
            catalog.claim(&[Kind::Keywords]).is_empty(),
            "already loading"
        );
        catalog.keywords.settle(None);
        catalog.sets.settle(Some(vec!["Alpha".to_owned()]));
        assert!(catalog.any_failed());
        assert_eq!(catalog.claim(&Kind::ALL).len(), 6, "all but sets");
        assert_eq!(catalog.words(Kind::Sets), Some(&["Alpha".to_owned()][..]));
        assert_eq!(catalog.words(Kind::CardRoles), None);
    }
}
