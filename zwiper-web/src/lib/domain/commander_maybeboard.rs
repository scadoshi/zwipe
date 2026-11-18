//! The commander maybeboard's in-memory narrowing: the Quick add results
//! that are not saved yet.

// Belongs in zwipe-ui, whose `commander::maybeboard` already holds the same
// function; the screen switches over with its model.

use std::collections::HashSet;
use std::hash::BuildHasher;

use uuid::Uuid;

/// Whether a Quick add result can still be added: it has an oracle id and
/// that commander is not already on the maybeboard.
pub fn is_addable<S: BuildHasher>(oracle_id: Option<Uuid>, saved: &HashSet<Uuid, S>) -> bool {
    oracle_id.is_some_and(|id| !saved.contains(&id))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn saved_and_oracle_less_results_are_not_addable() {
        let saved_id = Uuid::new_v4();
        let saved: HashSet<Uuid> = [saved_id].into_iter().collect();
        assert!(!is_addable(Some(saved_id), &saved));
        assert!(!is_addable(None, &saved));
        assert!(is_addable(Some(Uuid::new_v4()), &saved));
    }
}
