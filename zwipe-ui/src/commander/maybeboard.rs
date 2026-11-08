//! The commander maybeboard: the Show row's color pips, which Quick add results can still be added, the highlighted result's step, and the screen's fixed copy.

use std::collections::HashSet;
use std::hash::BuildHasher;

use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;

/// The saved list's group header.
pub const LIST_TITLE: &str = "Commanders";

/// The empty line when nothing is saved.
pub const NO_COMMANDERS: &str = "No commanders";

/// The empty line when entries are saved but none pass the pips and the card filter.
pub const NO_MATCH: &str = "No commanders match";

/// The empty line when the maybeboard failed to load.
pub const LOAD_FAILED: &str = "Couldn't load your commander maybeboard";

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
