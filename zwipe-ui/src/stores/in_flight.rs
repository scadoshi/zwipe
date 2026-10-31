//! Swipe adds whose create call has not answered yet.
//!
//! An undo can land before the add's POST answers, when there is nothing to delete yet. The undo is parked here, and the add's answer deletes the card once it lands.

use std::collections::HashMap;

use uuid::Uuid;

/// Per card, the adds still waiting on the server and how many of them were undone.
#[derive(Debug, Default)]
pub struct InFlightAdds {
    cards: HashMap<Uuid, Pending>,
}

#[derive(Debug, Default)]
struct Pending {
    posts: u32,
    undone: u32,
}

/// What an add does once its POST answers.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum AddOutcome {
    /// The add stands.
    Keep,
    /// The user undid it while it was in flight: delete the card.
    Undo,
}

impl InFlightAdds {
    /// Marks an add of `card_id` as sent.
    pub fn begin(&mut self, card_id: Uuid) {
        let pending = self.cards.entry(card_id).or_default();
        pending.posts = pending.posts.saturating_add(1);
    }

    /// Parks an undo of `card_id` when one of its adds is still in flight. True means the add's answer now owns the delete and the caller sends nothing.
    pub fn undo(&mut self, card_id: Uuid) -> bool {
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
    pub fn finish(&mut self, card_id: Uuid) -> AddOutcome {
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
    fn an_undo_during_the_post_deletes_once_it_lands() {
        let card = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        adds.begin(card);
        assert!(adds.undo(card));
        assert_eq!(adds.finish(card), AddOutcome::Undo);
        assert!(!adds.undo(card), "nothing left in flight");
    }

    #[test]
    fn an_add_nobody_undid_stands() {
        let card = Uuid::new_v4();
        let mut adds = InFlightAdds::default();
        assert!(!adds.undo(card), "no add was sent");
        adds.begin(card);
        assert_eq!(adds.finish(card), AddOutcome::Keep);
        assert!(!adds.undo(card), "the add already landed");
    }

    #[test]
    fn one_undo_per_in_flight_add() {
        let (card, other) = (Uuid::new_v4(), Uuid::new_v4());
        let mut adds = InFlightAdds::default();
        adds.begin(card);
        adds.begin(other);
        assert!(adds.undo(card));
        assert!(!adds.undo(card));
        assert_eq!(adds.finish(other), AddOutcome::Keep);
        assert_eq!(adds.finish(card), AddOutcome::Undo);
    }
}
