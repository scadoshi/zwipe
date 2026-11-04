//! The commander maybeboard's in-memory narrowing: the Show row's color
//! pips and the Quick add results that are not saved yet.

// Belongs in zwipe-ui, whose `commander::maybeboard` already holds the same
// two functions; the screen switches over with its model.

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
}
