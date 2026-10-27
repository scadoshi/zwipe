//! The oracle-tag profile of a group of cards: which tags the cards lean on
//! more than Magic cards do overall, and by how much.
//!
//! A deck's profile feeds the commander suggestions and the Playstyle tag
//! table, and a commander's precomputed profile is scored against it, so every
//! builder of a profile follows these steps in this order:
//!
//! 1. Count each oracle tag's distinct cards in the group (`deck_cards`) and in
//!    the whole tagged corpus (`corpus_cards`), plus the group's and the
//!    corpus's cards carrying at least one oracle tag of any kind
//!    (`deck_tagged`, `corpus_tagged`).
//! 2. Drop tags starting with [`PROFILE_EXCLUDED_TAG_PREFIX`] and tags under no
//!    card role.
//! 3. Weigh each remaining tag with [`lift_weight`], dropping the ones it
//!    returns `None` for.
//! 4. Keep the [`PROFILE_TAG_LIMIT`] heaviest, slug ascending between equal
//!    weights.
//! 5. Divide each kept weight by their sum, so the profile sums to 1.
//!
//! [`profile_weights`] runs all five steps.

use std::collections::BTreeMap;

/// Most tags one profile keeps, the heaviest by [`lift_weight`].
pub const PROFILE_TAG_LIMIT: u16 = 40;

/// Tags starting with this group cards by print run rather than by what they
/// do, so no profile carries them.
pub const PROFILE_EXCLUDED_TAG_PREFIX: &str = "cycle-";

/// One tag's counts for [`profile_weights`].
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TagCounts {
    /// The oracle tag slug.
    pub slug: String,
    /// Distinct cards in the group carrying the tag.
    pub deck_cards: u32,
    /// Distinct cards in the whole corpus carrying the tag.
    pub corpus_cards: u32,
}

/// How over-represented a tag is in a group of cards against all Magic
/// cards: `lift = (deck_cards / deck_tagged) / (corpus_cards / corpus_tagged)`,
/// the tag's share of the group's tagged cards over its share of the corpus's.
/// The weight is `deck_cards × ln(lift)`, so a tag counts for more the more
/// cards carry it and the more the group leans on it.
///
/// `None` when the lift is not above 1 (the group leans on the tag no more
/// than the corpus does) or any count is 0. The lift test compares the
/// integer cross products, so a tag exactly at the corpus rate is always out.
#[must_use]
pub fn lift_weight(
    deck_cards: u32,
    deck_tagged: u32,
    corpus_cards: u32,
    corpus_tagged: u32,
) -> Option<f64> {
    if deck_cards == 0 || deck_tagged == 0 || corpus_cards == 0 || corpus_tagged == 0 {
        return None;
    }
    let over = u64::from(deck_cards).checked_mul(u64::from(corpus_tagged))?;
    let under = u64::from(corpus_cards).checked_mul(u64::from(deck_tagged))?;
    if over <= under {
        return None;
    }
    let n = f64::from(deck_cards);
    let lift = (n / f64::from(deck_tagged)) / (f64::from(corpus_cards) / f64::from(corpus_tagged));
    Some(n * lift.ln())
}

/// A group's tag profile from its tag counts, following the module's five
/// steps: tag slug to weight, the weights summing to 1. `has_role` says
/// whether a slug falls under a card role. Empty when no tag survives.
#[must_use]
pub fn profile_weights(
    tags: &[TagCounts],
    deck_tagged: u32,
    corpus_tagged: u32,
    has_role: impl Fn(&str) -> bool,
) -> BTreeMap<String, f64> {
    let mut weighted: Vec<(&str, f64)> = tags
        .iter()
        .filter(|t| !t.slug.starts_with(PROFILE_EXCLUDED_TAG_PREFIX) && has_role(&t.slug))
        .filter_map(|t| {
            lift_weight(t.deck_cards, deck_tagged, t.corpus_cards, corpus_tagged)
                .map(|w| (t.slug.as_str(), w))
        })
        .collect();
    weighted.sort_by(|a, b| b.1.total_cmp(&a.1).then_with(|| a.0.cmp(b.0)));
    weighted.truncate(usize::from(PROFILE_TAG_LIMIT));
    let sum: f64 = weighted.iter().map(|(_, w)| w).sum();
    weighted
        .into_iter()
        .map(|(slug, w)| (slug.to_string(), w / sum))
        .collect()
}

#[cfg(test)]
mod tests {
    use super::*;

    fn counts(slug: &str, deck_cards: u32, corpus_cards: u32) -> TagCounts {
        TagCounts {
            slug: slug.to_string(),
            deck_cards,
            corpus_cards,
        }
    }

    fn close(got: f64, want: f64) -> bool {
        (got - want).abs() < 1e-12
    }

    #[test]
    fn lift_weight_is_cards_times_the_log_of_the_lift() {
        // 10 of 100 deck cards against 1,000 of 100,000 corpus cards: lift 10.
        let w = lift_weight(10, 100, 1_000, 100_000).unwrap();
        assert!(close(w, 10.0 * 10_f64.ln()), "{w}");
    }

    #[test]
    fn a_tag_at_or_under_the_corpus_rate_has_no_weight() {
        assert_eq!(
            lift_weight(10, 100, 10_000, 100_000),
            None,
            "lift exactly 1"
        );
        assert_eq!(lift_weight(5, 100, 10_000, 100_000), None, "lift 0.5");
    }

    #[test]
    fn zero_counts_have_no_weight() {
        assert_eq!(lift_weight(0, 100, 10, 100_000), None);
        assert_eq!(lift_weight(1, 0, 10, 100_000), None);
        assert_eq!(lift_weight(1, 100, 0, 100_000), None, "no division by zero");
        assert_eq!(lift_weight(1, 100, 10, 0), None);
    }

    #[test]
    fn a_tag_on_every_deck_card_still_weighs_when_the_corpus_has_it_less() {
        let w = lift_weight(30, 30, 50_000, 100_000).unwrap();
        assert!(close(w, 30.0 * 2_f64.ln()), "{w}");
        assert_eq!(
            lift_weight(30, 30, 100_000, 100_000),
            None,
            "a tag on every corpus card never weighs"
        );
    }

    #[test]
    fn a_generic_tag_drops_out_and_a_distinctive_one_rises() {
        // A 100-card deck in a 100,000-card corpus. activated-ability sits on
        // 31% of both; token-maker on 12% of the deck and 2% of the corpus.
        let tags = [
            counts("activated-ability", 31, 31_000),
            counts("token-maker", 12, 2_000),
        ];
        let weights = profile_weights(&tags, 100, 100_000, |_| true);
        assert_eq!(
            weights.keys().collect::<Vec<_>>(),
            vec!["token-maker"],
            "the deck leans on activated abilities no more than Magic does"
        );
        assert!(close(weights.get("token-maker").copied().unwrap(), 1.0));
    }

    #[test]
    fn many_cards_at_a_small_lift_lose_to_fewer_at_a_large_one() {
        // generic: 40 cards at lift 1.1; distinctive: 8 cards at lift 20.
        let tags = [counts("generic", 40, 36_364), counts("distinctive", 8, 400)];
        let weights = profile_weights(&tags, 100, 100_000, |_| true);
        assert!(
            weights.get("distinctive").copied().unwrap() > weights.get("generic").copied().unwrap(),
            "8 × ln 20 beats 40 × ln 1.1: {weights:?}"
        );
    }

    #[test]
    fn tags_under_no_role_and_cycle_tags_are_left_out() {
        let tags = [
            counts("alliteration", 20, 1_000),
            counts("cycle-dominaria-uncommon", 5, 50),
            counts("sacrifice-outlet", 5, 1_000),
        ];
        let weights = profile_weights(&tags, 100, 100_000, |slug| slug != "alliteration");
        assert_eq!(weights.keys().collect::<Vec<_>>(), vec!["sacrifice-outlet"]);
    }

    #[test]
    fn the_heaviest_tags_are_kept_ties_on_slug_and_sum_to_one() {
        let tags: Vec<TagCounts> = (0..50)
            .map(|i| counts(&format!("tag-{i:02}"), 5, 100))
            .collect();
        let weights = profile_weights(&tags, 100, 100_000, |_| true);
        assert_eq!(weights.len(), usize::from(PROFILE_TAG_LIMIT));
        assert_eq!(weights.keys().next().map(String::as_str), Some("tag-00"));
        assert_eq!(weights.keys().last().map(String::as_str), Some("tag-39"));
        let sum: f64 = weights.values().sum();
        assert!(close(sum, 1.0), "{sum}");
        assert!(weights.values().all(|w| close(*w, 1.0 / 40.0)));
    }

    #[test]
    fn nothing_surviving_is_an_empty_profile() {
        let tags = [counts("everywhere", 10, 100_000)];
        assert!(profile_weights(&tags, 10, 100_000, |_| true).is_empty());
        assert!(profile_weights(&[], 10, 100_000, |_| true).is_empty());
    }
}
