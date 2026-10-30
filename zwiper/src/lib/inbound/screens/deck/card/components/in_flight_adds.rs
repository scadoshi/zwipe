//! Swipe adds whose create call hasn't answered yet.
//!
//! An undo can land before the add's POST returns. There is nothing to
//! delete yet, so the undo is parked here and the add's own task deletes the
//! card once its POST lands.

use std::collections::HashMap;

use uuid::Uuid;

/// Per-card count of adds still waiting on the server, and how many of them
/// the user has already undone.
#[derive(Debug, Default)]
pub(crate) struct InFlightAdds {
    cards: HashMap<Uuid, Pending>,
}

#[derive(Debug, Default)]
struct Pending {
    posts: u32,
    undone: u32,
}

/// What an add's task does once its POST answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum AddOutcome {
    /// The add stands: record it as usual.
    Keep,
    /// The user undid it while it was in flight: delete the card.
    Undo,
}

impl InFlightAdds {
    /// Marks an add of `card_id` as sent.
    pub(crate) fn begin(&mut self, card_id: Uuid) {
        let pending = self.cards.entry(card_id).or_default();
        pending.posts = pending.posts.saturating_add(1);
    }

    /// Parks an undo of `card_id` if one of its adds is still in flight.
    /// `true` means the add's task now owns the delete and the caller sends
    /// nothing.
    pub(crate) fn undo(&mut self, card_id: Uuid) -> bool {
        let Some(pending) = self.cards.get_mut(&card_id) else {
            return false;
        };
        if pending.undone >= pending.posts {
            return false;
        }
        pending.undone = pending.undone.saturating_add(1);
        true
    }

    /// Settles the oldest in-flight add of `card_id` once its POST answers.
    pub(crate) fn finish(&mut self, card_id: Uuid) -> AddOutcome {
        let Some(pending) = self.cards.get_mut(&card_id) else {
            return AddOutcome::Keep;
        };
        pending.posts = pending.posts.saturating_sub(1);
        let outcome = if pending.undone > 0 {
            pending.undone = pending.undone.saturating_sub(1);
            AddOutcome::Undo
        } else {
            AddOutcome::Keep
        };
        if pending.posts == 0 {
            self.cards.remove(&card_id);
        }
        outcome
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn undo_during_the_post_deletes_once_it_lands() {
        let card = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        adds.begin(card);
        assert!(adds.undo(card), "an in-flight add takes the undo");
        assert_eq!(
            adds.finish(card),
            AddOutcome::Undo,
            "the landed add is deleted"
        );
        assert!(!adds.undo(card), "nothing left in flight");
    }

    #[test]
    fn an_add_nobody_undid_stands() {
        let card = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        adds.begin(card);
        assert_eq!(adds.finish(card), AddOutcome::Keep);
    }

    #[test]
    fn undo_after_the_post_answered_is_not_parked() {
        let card = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        assert!(!adds.undo(card), "no add was ever sent");
        adds.begin(card);
        assert_eq!(adds.finish(card), AddOutcome::Keep);
        assert!(
            !adds.undo(card),
            "the add already landed: the normal delete path runs"
        );
    }

    #[test]
    fn one_undo_per_in_flight_add() {
        let card = Uuid::new_v4();
        let other = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        adds.begin(card);
        adds.begin(other);
        assert!(adds.undo(card));
        assert!(
            !adds.undo(card),
            "the second undo has no add left to cancel"
        );
        assert_eq!(
            adds.finish(other),
            AddOutcome::Keep,
            "another card is untouched"
        );
        assert_eq!(adds.finish(card), AddOutcome::Undo);
    }
}
