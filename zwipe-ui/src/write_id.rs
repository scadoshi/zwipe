//! The id a create sends for the thing it makes, kept across Saves of the same form: a Save
//! whose reply was lost may have landed, so the next Save of the same form goes under the same
//! id and the server answers the thing it made instead of making a second one or turning the
//! name down. A changed form is a new thing, with a new id.

use uuid::Uuid;

/// The id for the form last sent, held until its write goes through. `K` is what the Save
/// sends, compared whole: the same `K` keeps the id, any other gets a fresh one.
#[derive(Debug, Clone, PartialEq)]
pub struct WriteId<K> {
    held: Option<(K, Uuid)>,
}

impl<K> Default for WriteId<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> WriteId<K> {
    /// Holds no id: the first Save gets a fresh one.
    #[must_use]
    pub const fn new() -> Self {
        Self { held: None }
    }

    /// Forgets the id once its write went through, so the next Save is a new thing.
    pub fn landed(&mut self) {
        self.held = None;
    }
}

impl<K: PartialEq> WriteId<K> {
    /// The id to send `sent` under: the held one when `sent` is what the last Save sent, else
    /// `fresh()`, held for `sent` from now on.
    pub fn id_for(&mut self, sent: K, fresh: impl FnOnce() -> Uuid) -> Uuid {
        if let Some((last, id)) = &self.held
            && *last == sent
        {
            return *id;
        }
        let id = fresh();
        self.held = Some((sent, id));
        id
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_same_form_keeps_its_id_and_a_changed_one_gets_a_fresh_one() {
        let mut ids = WriteId::new();
        let first = ids.id_for("Burn", || Uuid::from_u128(1));
        assert_eq!(first, Uuid::from_u128(1));
        assert_eq!(ids.id_for("Burn", || Uuid::from_u128(2)), first);
        assert_eq!(
            ids.id_for("Burn 2", || Uuid::from_u128(3)),
            Uuid::from_u128(3)
        );
        assert_eq!(
            ids.id_for("Burn", || Uuid::from_u128(4)),
            Uuid::from_u128(4),
            "only the last form sent keeps its id"
        );
    }

    #[test]
    fn a_write_that_went_through_lets_the_next_save_be_a_new_thing() {
        let mut ids = WriteId::new();
        let _ = ids.id_for("Burn", || Uuid::from_u128(1));
        ids.landed();
        assert_eq!(
            ids.id_for("Burn", || Uuid::from_u128(2)),
            Uuid::from_u128(2)
        );
    }
}
