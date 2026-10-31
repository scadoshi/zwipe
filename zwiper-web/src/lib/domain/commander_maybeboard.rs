//! The commander maybeboard's in-memory narrowing: the Show row's color
//! pips and the Quick add results that are not saved yet.

// Belongs in zwipe-ui: the app keeps these inline in its maybeboard screen,
// and zwipe-ui has no maybeboard module yet.

use std::collections::HashSet;
use std::hash::BuildHasher;

use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::colors::Color;

/// Whether a commander with `identity` stays under the Show row's `picked`
/// pips: it must hold every picked color. No pips picked is `All`.
pub fn passes_pips<S: BuildHasher>(identity: &[Color], picked: &HashSet<Color, S>) -> bool {
    picked.iter().all(|color| identity.contains(color))
}

/// Whether a Quick add result can still be added: it has an oracle id and
/// that commander is not already on the maybeboard.
pub fn is_addable<S: BuildHasher>(oracle_id: Option<Uuid>, saved: &HashSet<Uuid, S>) -> bool {
    oracle_id.is_some_and(|id| !saved.contains(&id))
}

/// Moves the highlighted Quick add result one step down (`forward`) or up
/// through `count` results, stopping at either end. `None` is no highlight;
/// the first step down lands on the first result.
pub fn step_highlight(current: Option<usize>, count: usize, forward: bool) -> Option<usize> {
    let last = count.checked_sub(1)?;
    Some(match (current, forward) {
        (None, true) => 0,
        (None, false) => last,
        (Some(at), true) => at.saturating_add(1).min(last),
        (Some(at), false) => at.saturating_sub(1).min(last),
    })
}

#[cfg(test)]
mod tests {
    use Color::{Black, Blue, Green, White};

    use super::*;

    #[test]
    fn pips_keep_identities_holding_every_picked_color() {
        let picked: HashSet<Color> = [White, Blue].into_iter().collect();
        assert!(passes_pips(&[White, Blue, Black], &picked));
        assert!(!passes_pips(&[White, Green], &picked));
        assert!(passes_pips(&[], &HashSet::<Color>::new()), "no pips is All");
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
    fn the_highlight_stops_at_both_ends() {
        assert_eq!(step_highlight(None, 3, true), Some(0));
        assert_eq!(step_highlight(None, 3, false), Some(2));
        assert_eq!(step_highlight(Some(2), 3, true), Some(2));
        assert_eq!(step_highlight(Some(0), 3, false), Some(0));
        assert_eq!(step_highlight(Some(1), 0, true), None);
        assert_eq!(
            step_highlight(Some(5), 3, false),
            Some(2),
            "a stale index clamps"
        );
    }
}
