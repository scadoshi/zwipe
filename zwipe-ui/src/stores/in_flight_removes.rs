//! Swipe removes whose delete has not answered yet.
//!
//! An undo can land while the remove's DELETE is still out, when putting the card back
//! would race the delete. The restore is parked here, and the remove's answer sends it
//! once the delete lands. The store lives for the whole run, so a parked restore still
//! goes out after the screen that parked it is gone.

use std::collections::{HashMap, HashSet};

use uuid::Uuid;

/// Per card, the removes still waiting on the server and the restore an undo parked on
/// one. `R` is what a client needs to put the card back.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct InFlightRemoves<R> {
    deleting: HashSet<Uuid>,
    parked: HashMap<Uuid, R>,
}

impl<R> Default for InFlightRemoves<R> {
    fn default() -> Self {
        Self {
            deleting: HashSet::new(),
            parked: HashMap::new(),
        }
    }
}

impl<R> InFlightRemoves<R> {
    /// Marks a remove of `card_id` as sent.
    pub fn begin(&mut self, card_id: Uuid) {
        self.deleting.insert(card_id);
    }

    /// Parks `restore` on `card_id`'s remove while its delete is out and nothing is
    /// parked on it yet, and returns `None`: the remove's answer now owns the restore.
    /// Otherwise hands `restore` back for the caller to send now.
    pub fn park(&mut self, card_id: Uuid, restore: R) -> Option<R> {
        if self.deleting.contains(&card_id) && !self.parked.contains_key(&card_id) {
            self.parked.insert(card_id, restore);
            return None;
        }
        Some(restore)
    }

    /// Ends `card_id`'s remove and returns the restore parked on it, if any.
    pub fn finish(&mut self, card_id: Uuid) -> Option<R> {
        self.deleting.remove(&card_id);
        self.parked.remove(&card_id)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_undo_with_no_delete_out_restores_now() {
        let mut removes = InFlightRemoves::default();
        let card = Uuid::new_v4();
        assert_eq!(removes.park(card, "back"), Some("back"));
    }

    #[test]
    fn an_undo_while_the_delete_is_out_waits_for_it() {
        let mut removes = InFlightRemoves::default();
        let card = Uuid::new_v4();
        removes.begin(card);
        assert_eq!(removes.park(card, "back"), None, "parked on the delete");
        assert_eq!(removes.finish(card), Some("back"), "the answer sends it");
        assert_eq!(removes.finish(card), None, "exactly once");
        assert_eq!(
            removes.park(card, "again"),
            Some("again"),
            "no delete out now"
        );
    }

    #[test]
    fn a_second_undo_on_the_same_delete_is_not_parked() {
        let mut removes = InFlightRemoves::default();
        let card = Uuid::new_v4();
        removes.begin(card);
        assert_eq!(removes.park(card, "first"), None);
        assert_eq!(removes.park(card, "second"), Some("second"));
    }

    #[test]
    fn a_delete_with_nothing_parked_answers_none() {
        let mut removes: InFlightRemoves<&str> = InFlightRemoves::default();
        let card = Uuid::new_v4();
        removes.begin(card);
        assert_eq!(removes.finish(card), None);
    }
}
