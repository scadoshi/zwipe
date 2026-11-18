//! Writes whose answer was lost. A request can land on the server while the client sees only a
//! network error, and the user's next try then answers as if it failed: an add says the card is
//! already in the deck, a delete says there is nothing to delete. [`Unconfirmed`] remembers the
//! writes that ended in a network error, so the client can read that repeat answer as the earlier
//! try landing.
//!
//! Each client keeps one [`Unconfirmed`] for the session, above its screens, since a lost add can
//! come back as a Quick add or a Send to on another screen. The client turns its error into an
//! [`Answer`]; nothing here sends or retries.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use uuid::Uuid;

/// What a write does, which decides what its repeat answers once the first try landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Creates something unique, so a repeat answers 409 Conflict (a card already in the deck).
    Create,
    /// Deletes something, so a repeat answers 404 Not Found.
    Delete,
}

impl Kind {
    /// The status a repeat of this write answers once the first try has landed.
    #[must_use]
    pub const fn repeat_status(self) -> u16 {
        match self {
            Self::Create => 409,
            Self::Delete => 404,
        }
    }
}

/// One write, by what it acts on: its kind, the thing it acts in (a deck) and the thing it acts
/// on (a printing, a card, the deck itself).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub struct WriteKey {
    kind: Kind,
    scope: Uuid,
    item: Uuid,
}

impl WriteKey {
    /// Adding `printing` to `deck`.
    #[must_use]
    pub const fn add(deck: Uuid, printing: Uuid) -> Self {
        Self {
            kind: Kind::Create,
            scope: deck,
            item: printing,
        }
    }

    /// Deleting `item` from `scope`: a card from a deck, or a deck from the user's (`item` and
    /// `scope` both the deck).
    #[must_use]
    pub const fn delete(scope: Uuid, item: Uuid) -> Self {
        Self {
            kind: Kind::Delete,
            scope,
            item,
        }
    }
}

/// How a write ended, as the client reads its result.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Answer {
    /// It went through.
    Done,
    /// No answer came back: the request may or may not have landed.
    Network,
    /// The server answered with this error status.
    Status(u16),
    /// Any other failure: a reply that didn't decode, or no session.
    Failed,
}

/// What the client shows for a write.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Verdict {
    /// It went through now.
    Done,
    /// The answer says an earlier try, whose answer was lost, went through: show it as done.
    LandedEarlier,
    /// It failed: show the error as today.
    Failed,
}

/// The writes of this session whose last try ended in a network error.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct Unconfirmed {
    lost: HashSet<WriteKey>,
}

impl Unconfirmed {
    /// Takes how a write ended and says what to show. A network error is remembered; the status a
    /// repeat answers, for a write remembered that way, is that earlier try landing. A success
    /// forgets it; any other failure keeps it, since the earlier try may still have landed.
    pub fn settle(&mut self, key: WriteKey, answer: Answer) -> Verdict {
        match answer {
            Answer::Done => {
                self.lost.remove(&key);
                Verdict::Done
            }
            Answer::Network => {
                self.lost.insert(key);
                Verdict::Failed
            }
            Answer::Status(status)
                if status == key.kind.repeat_status() && self.lost.remove(&key) =>
            {
                Verdict::LandedEarlier
            }
            Answer::Status(_) | Answer::Failed => Verdict::Failed,
        }
    }
}
