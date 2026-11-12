//! The commander maybeboard: the Show row's color pips, the Sort row's orders, which Quick add results can still be added, the highlighted result's step, and the screen's fixed copy.

use std::cmp::Ordering;
use std::collections::HashSet;
use std::hash::BuildHasher;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;

/// The saved list's group header.
pub const LIST_TITLE: &str = "Commanders";

/// The empty line when nothing is saved.
pub const NO_COMMANDERS: &str = "No commanders";

/// The empty line when entries are saved but none pass the pips and the card filter.
pub const NO_MATCH: &str = "No commanders match";

/// The empty line when the maybeboard failed to load.
pub const LOAD_FAILED: &str = "Couldn't load your commander maybeboard";

/// What the saved commanders sort on.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq, Hash)]
pub enum SortBy {
    /// The order they were saved in, the newest first ascending.
    #[default]
    Newest,
    /// The card's name.
    Name,
    /// The mana value, the cheapest first ascending.
    ManaValue,
    /// The color identity: fewer colors first ascending, then in WUBRG order.
    Color,
}

impl SortBy {
    /// Every key in chip order.
    pub const ALL: [Self; 4] = [Self::Newest, Self::Name, Self::ManaValue, Self::Color];

    /// The chip's label.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::Newest => "Newest saved",
            Self::Name => "Name",
            Self::ManaValue => "Mana value",
            Self::Color => "Color",
        }
    }

    /// The key [`Sort::key`] stores it under.
    fn slug(self) -> &'static str {
        match self {
            Self::Newest => "newest",
            Self::Name => "name",
            Self::ManaValue => "mana",
            Self::Color => "color",
        }
    }
}

/// The saved commanders' order: a key and a direction. Commanders the key ties on fall back to their names, A to Z.
#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub struct Sort {
    /// What to sort on.
    pub by: SortBy,
    /// Which way.
    pub direction: SortDirection,
}

impl Sort {
    /// The sort as one string, such as `mana-descending`, for a client that stores it as text.
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

    /// `cards`, given in saved order with the newest first, in this order.
    #[must_use]
    pub fn sorted(self, mut cards: Vec<Card>) -> Vec<Card> {
        let directed = |ordering: Ordering| match self.direction {
            SortDirection::Ascending => ordering,
            SortDirection::Descending => ordering.reverse(),
        };
        let by_name =
            |a: &Card, b: &Card| CardSortKey::Name.compare(&a.scryfall_data, &b.scryfall_data);
        match self.by {
            SortBy::Newest => {
                if self.direction == SortDirection::Descending {
                    cards.reverse();
                }
            }
            SortBy::Name => cards.sort_by(|a, b| directed(by_name(a, b))),
            SortBy::ManaValue => cards.sort_by(|a, b| {
                directed(CardSortKey::Cmc.compare(&a.scryfall_data, &b.scryfall_data))
                    .then_with(|| by_name(a, b))
            }),
            SortBy::Color => cards.sort_by(|a, b| {
                directed(
                    identity_rank(&a.scryfall_data.color_identity)
                        .cmp(&identity_rank(&b.scryfall_data.color_identity)),
                )
                .then_with(|| by_name(a, b))
            }),
        }
        cards
    }
}

/// Where a color identity falls in the Color order: its size, then its colors' places in WUBRG.
fn identity_rank(identity: &[Color]) -> (usize, Vec<usize>) {
    let mut places: Vec<usize> = identity
        .iter()
        .filter_map(|color| Color::all().into_iter().position(|c| c == *color))
        .collect();
    places.sort_unstable();
    (places.len(), places)
}

/// Which way a highlight step moves through the Quick add results.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Step {
    /// Toward the last result.
    Down,
    /// Toward the first result.
    Up,
}

/// Whether a commander with `identity` stays under the Show row's `picked` pips: it must hold every picked color. No pips picked is All, so every commander passes.
#[must_use]
pub fn passes_pips<S: BuildHasher>(identity: &[Color], picked: &HashSet<Color, S>) -> bool {
    picked.iter().all(|color| identity.contains(color))
}

/// Whether a Quick add result can still be added: it has an oracle id and that commander is not already on the maybeboard.
#[must_use]
pub fn is_addable<S: BuildHasher>(oracle_id: Option<Uuid>, saved: &HashSet<Uuid, S>) -> bool {
    oracle_id.is_some_and(|id| !saved.contains(&id))
}

/// Moves the highlighted Quick add result one `step` through `count` results, stopping at either end. `None` is no highlight: the first step down lands on the first result and the first step up on the last. With no results there is no highlight, and a stale index past the end clamps to the last result.
#[must_use]
pub fn step_highlight(current: Option<usize>, count: usize, step: Step) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match (current, step) {
        (None, Step::Down) => 0,
        (None, Step::Up) => last,
        (Some(at), Step::Down) => at.saturating_add(1).min(last),
        (Some(at), Step::Up) => at.saturating_sub(1).min(last),
    })
}

#[cfg(test)]
mod tests {
    use Color::{Black, Blue, Green, Red, White};

    use super::*;
    use crate::test_support::card;

    /// A commander named `name` with mana value `cmc` and `identity`.
    fn commander(name: &str, cmc: f64, identity: &[Color]) -> Card {
        let mut c = card(name);
        c.scryfall_data.cmc = Some(cmc);
        c.scryfall_data.color_identity = identity.to_vec().into();
        c
    }

    /// The names of `cards`, in order.
    fn names(cards: &[Card]) -> Vec<&str> {
        cards
            .iter()
            .map(|c| c.scryfall_data.name.as_str())
            .collect()
    }

    /// Saved order, newest first.
    fn saved() -> Vec<Card> {
        vec![
            commander("Krenko", 4.0, &[Red]),
            commander("Atraxa", 4.0, &[White, Blue, Black, Green]),
            commander("Kenrith", 5.0, &[White, Blue, Black, Red, Green]),
            commander("Brago", 4.0, &[White, Blue]),
            commander("Karn", 4.0, &[]),
        ]
    }

    #[test]
    fn the_default_is_the_saved_order() {
        assert_eq!(Sort::default().by, SortBy::Newest);
        assert_eq!(
            names(&Sort::default().sorted(saved())),
            ["Krenko", "Atraxa", "Kenrith", "Brago", "Karn"]
        );
        let oldest = Sort {
            by: SortBy::Newest,
            direction: SortDirection::Descending,
        };
        assert_eq!(
            names(&oldest.sorted(saved())),
            ["Karn", "Brago", "Kenrith", "Atraxa", "Krenko"]
        );
    }

    #[test]
    fn each_key_orders_and_ties_fall_back_to_the_name() {
        let by = |by| Sort {
            by,
            direction: SortDirection::Ascending,
        };
        assert_eq!(
            names(&by(SortBy::Name).sorted(saved())),
            ["Atraxa", "Brago", "Karn", "Kenrith", "Krenko"]
        );
        assert_eq!(
            names(&by(SortBy::ManaValue).sorted(saved())),
            ["Atraxa", "Brago", "Karn", "Krenko", "Kenrith"]
        );
        assert_eq!(
            names(&by(SortBy::Color).sorted(saved())),
            ["Karn", "Krenko", "Brago", "Atraxa", "Kenrith"]
        );
        let down = Sort {
            by: SortBy::ManaValue,
            direction: SortDirection::Descending,
        };
        assert_eq!(
            names(&down.sorted(saved())),
            ["Kenrith", "Atraxa", "Brago", "Karn", "Krenko"]
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
        assert_eq!(Sort::from_key("name"), None);
        assert_eq!(Sort::from_key("size-ascending"), None);
    }

    #[test]
    fn pips_keep_identities_holding_every_picked_color() {
        let picked: HashSet<Color> = [White, Blue].into_iter().collect();
        assert!(passes_pips(&[White, Blue, Black], &picked));
        assert!(
            passes_pips(&[Blue, White], &picked),
            "order does not matter"
        );
        assert!(!passes_pips(&[White, Green], &picked));
        assert!(
            !passes_pips(&[], &picked),
            "colorless holds no picked color"
        );
    }

    #[test]
    fn no_pips_is_all() {
        let none = HashSet::<Color>::new();
        assert!(passes_pips(&[], &none));
        assert!(passes_pips(&[White, Blue, Black, Red, Green], &none));
    }

    #[test]
    fn one_pip_keeps_every_identity_with_that_color() {
        let picked: HashSet<Color> = [Red].into_iter().collect();
        assert!(passes_pips(&[Red], &picked));
        assert!(passes_pips(&[Black, Red, Green], &picked));
        assert!(!passes_pips(&[Black, Green], &picked));
    }

    #[test]
    fn saved_and_oracle_less_results_are_not_addable() {
        let saved_id = Uuid::new_v4();
        let saved: HashSet<Uuid> = [saved_id].into_iter().collect();
        assert!(!is_addable(Some(saved_id), &saved));
        assert!(!is_addable(None, &saved));
        assert!(is_addable(Some(Uuid::new_v4()), &saved));
    }

    #[test]
    fn with_nothing_saved_every_result_with_an_oracle_is_addable() {
        let saved = HashSet::<Uuid>::new();
        assert!(is_addable(Some(Uuid::new_v4()), &saved));
        assert!(!is_addable(None, &saved));
    }

    #[test]
    fn the_highlight_stops_at_both_ends() {
        assert_eq!(step_highlight(None, 3, Step::Down), Some(0));
        assert_eq!(step_highlight(None, 3, Step::Up), Some(2));
        assert_eq!(step_highlight(Some(2), 3, Step::Down), Some(2));
        assert_eq!(step_highlight(Some(0), 3, Step::Up), Some(0));
        assert_eq!(step_highlight(Some(1), 0, Step::Down), None);
        assert_eq!(
            step_highlight(Some(5), 3, Step::Up),
            Some(2),
            "a stale index clamps"
        );
    }

    #[test]
    fn the_highlight_moves_one_result_at_a_time() {
        assert_eq!(step_highlight(Some(0), 3, Step::Down), Some(1));
        assert_eq!(step_highlight(Some(1), 3, Step::Down), Some(2));
        assert_eq!(step_highlight(Some(2), 3, Step::Up), Some(1));
        assert_eq!(
            step_highlight(Some(5), 3, Step::Down),
            Some(2),
            "a stale index clamps going down too"
        );
    }

    #[test]
    fn no_results_clears_the_highlight_either_way() {
        assert_eq!(step_highlight(None, 0, Step::Down), None);
        assert_eq!(step_highlight(None, 0, Step::Up), None);
        assert_eq!(step_highlight(Some(0), 0, Step::Up), None);
    }

    #[test]
    fn one_result_holds_the_highlight() {
        for step in [Step::Down, Step::Up] {
            assert_eq!(step_highlight(None, 1, step), Some(0), "{step:?}");
            assert_eq!(step_highlight(Some(0), 1, step), Some(0), "{step:?}");
        }
    }

    #[test]
    fn the_fixed_copy_is_the_apps() {
        assert_eq!(LIST_TITLE, "Commanders");
        assert_eq!(NO_COMMANDERS, "No commanders");
        assert_eq!(NO_MATCH, "No commanders match");
        assert_eq!(LOAD_FAILED, "Couldn't load your commander maybeboard");
    }
}
