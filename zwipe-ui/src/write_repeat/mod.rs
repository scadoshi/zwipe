//! Writes whose answer was lost. A request can land on the server while the client sees only a
//! network error, and the user's next try then answers as if it failed: an add says the card is
//! already in the deck, a delete says there is nothing to delete. [`Unconfirmed`] remembers the
//! writes that ended in a network error, so the client can read that repeat answer as the earlier
//! try landing. The answers are the server's: 409 for an add, 422 "deck card not found" for a
//! card's removal, 404 for a deck's deletion.
//!
//! Each client keeps one [`Unconfirmed`] for the session, above its screens, since a lost add can
//! come back as a Quick add or a Send to on another screen. The client turns its error into an
//! [`Answer`]; nothing here sends or retries.

#[cfg(test)]
mod tests;

use std::collections::HashSet;

use uuid::Uuid;
use zwipe_core::domain::deck::{Board, Deck, DeckCard, Quantity};
use zwipe_core::http::contracts::deck_card::HttpCreateDeckCard;

use crate::write_id::WriteId;

/// What a write does, which decides what its repeat answers once the first try landed.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum Kind {
    /// Adds a card to a deck, so a repeat answers 409 Conflict (already in the deck).
    AddCard,
    /// Removes a card from a deck, so a repeat answers 422 "deck card not found".
    RemoveCard,
    /// Deletes a deck, so a repeat answers 404 Not Found.
    DeleteDeck,
}

impl Kind {
    /// The status a repeat of this write answers once the first try has landed.
    #[must_use]
    pub const fn repeat_status(self) -> u16 {
        match self {
            Self::AddCard => 409,
            Self::RemoveCard => 422,
            Self::DeleteDeck => 404,
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
            kind: Kind::AddCard,
            scope: deck,
            item: printing,
        }
    }

    /// Removing the card `printing` from `deck`.
    #[must_use]
    pub const fn remove_card(deck: Uuid, printing: Uuid) -> Self {
        Self {
            kind: Kind::RemoveCard,
            scope: deck,
            item: printing,
        }
    }

    /// Deleting `deck`.
    #[must_use]
    pub const fn delete_deck(deck: Uuid) -> Self {
        Self {
            kind: Kind::DeleteDeck,
            scope: deck,
            item: deck,
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

/// How a write ended for the screen that made it, a lost earlier try counted: a client's
/// authed runner answers with it.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Written<T> {
    /// It went through, with its answer.
    Done(T),
    /// An earlier try, whose answer was lost, went through: its repeat answered as a repeat does.
    LandedEarlier,
    /// It failed; the error toast is up.
    Failed,
}

impl<T> Written<T> {
    /// The answer, `landed()` for a write an earlier try already made, `None` for a failure: for
    /// a caller that goes on the same way either way it went through.
    #[must_use]
    pub fn or_landed(self, landed: impl FnOnce() -> T) -> Option<T> {
        match self {
            Self::Done(value) => Some(value),
            Self::LandedEarlier => Some(landed()),
            Self::Failed => None,
        }
    }
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

/// The row `deck`, as the server reads it back, holds for an add sent as `sent`: the sent
/// printing on the sent board, so an add whose answer was lost shows as landed with the server's
/// own row. `None` when the deck doesn't hold it there, or the request doesn't read back.
#[must_use]
pub fn landed_in(deck: &Deck, sent: &HttpCreateDeckCard) -> Option<DeckCard> {
    let printing: Uuid = sent.scryfall_data_id.parse().ok()?;
    let board = sent_board(sent)?;
    deck.entries
        .iter()
        .map(|entry| &entry.deck_card)
        .find(|row| row.scryfall_data_id == printing && row.board == board)
        .cloned()
}

/// The board an add's request names, the main deck when it names none; `None` for a name that
/// isn't a board.
fn sent_board(sent: &HttpCreateDeckCard) -> Option<Board> {
    match sent.board.as_deref() {
        Some(board) => Board::try_from(board).ok(),
        None => Some(Board::Deck),
    }
}

/// The deck card an add's request made in `deck`, for an add that [`Verdict::LandedEarlier`]: its
/// repeat's 409 carries no body, so the client builds what the server made from what it sent, a
/// fresh row with no star or flag. `None` when the request doesn't read back as one.
#[must_use]
pub fn landed_deck_card(deck: Uuid, sent: &HttpCreateDeckCard) -> Option<DeckCard> {
    let board = sent_board(sent)?;
    Some(DeckCard {
        deck_id: deck,
        scryfall_data_id: sent.scryfall_data_id.parse().ok()?,
        oracle_id: sent.oracle_id.parse().ok()?,
        quantity: Quantity::new(sent.quantity).ok()?,
        board,
        mvp_at: sent.mvp_at,
        flagged: false,
    })
}

/// How a form's submit ended (Register, Change password, Reset password, a link's confirm), as
/// the client reads it.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormAnswer {
    /// It went through.
    Done,
    /// No answer came back: it may or may not have gone through.
    Network,
    /// The refusal a repeat gets once the first try went through: a username taken, a current
    /// password that no longer matches, a link already used.
    RepeatRefused,
    /// Any other failure.
    Other,
}

/// What a form says in place of its client's own line ([`FormRepeat::settle`]).
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum FormNote {
    /// The answer was lost: say the submit may have gone through.
    Lost,
    /// A repeat was refused after a lost answer: the first try likely went through.
    Landed,
}

/// A form whose submit can land while its answer is lost, and whose repeat is then refused. The
/// server can't say whether the first try landed, so the form says what to do next instead of a
/// bare error. `K` is what the submit sends: a refusal reads as the lost try landing only for the
/// same `K`, so a retry with a mistyped field is refused as itself. Each send's identity comes
/// from a [`WriteId`], as a create's does.
#[derive(Debug, Clone, PartialEq)]
pub struct FormRepeat<K> {
    ids: WriteId<K>,
    /// The send whose answer was lost.
    lost: Option<Uuid>,
    /// How many sends took a fresh identity, which numbers the next one.
    sends: u128,
}

impl<K> Default for FormRepeat<K> {
    fn default() -> Self {
        Self::new()
    }
}

impl<K> FormRepeat<K> {
    /// A form with nothing sent yet.
    #[must_use]
    pub const fn new() -> Self {
        Self {
            ids: WriteId::new(),
            lost: None,
            sends: 0,
        }
    }
}

// The comparison is whole: a held form and its identity, the lost send and the count.
impl<K: Eq> Eq for FormRepeat<K> {}

impl<K: PartialEq> FormRepeat<K> {
    /// Takes how the submit of `sent` ended and answers with the note to show in place of the
    /// client's own line, `None` to show the client's own (a success, or a failure that isn't
    /// this form's repeat).
    pub fn settle(&mut self, sent: K, answer: FormAnswer) -> Option<FormNote> {
        let sends = &mut self.sends;
        let id = self.ids.id_for(sent, || {
            *sends = sends.wrapping_add(1);
            Uuid::from_u128(*sends)
        });
        match answer {
            FormAnswer::Done => {
                self.ids.landed();
                self.lost = None;
                None
            }
            FormAnswer::Network => {
                self.lost = Some(id);
                Some(FormNote::Lost)
            }
            FormAnswer::RepeatRefused if self.lost == Some(id) => {
                self.lost = None;
                Some(FormNote::Landed)
            }
            FormAnswer::RepeatRefused | FormAnswer::Other => None,
        }
    }
}

/// Register, its answer lost.
pub const REGISTER_LOST: &str = "We couldn't confirm your sign-up. Check your inbox for a verification email before trying again.";

/// Register, its repeat answered that the username is taken after a lost answer.
pub const REGISTER_LANDED: &str =
    "This account may already exist. Check your email for a verification link, or log in.";

/// Change password, its answer lost or its repeat refused as a wrong current password.
pub const PASSWORD_UNCONFIRMED: &str = "We couldn't confirm the change. It may have gone through, so try your new password next time you log in.";

/// Reset password, its answer lost or its repeat refused as a used link.
pub const RESET_UNCONFIRMED: &str = "We couldn't confirm the reset. Try logging in with your new password, and if that fails, ask for a new link.";

/// The title over a link page's line when its answer was lost (Verify email, Confirm email
/// change).
pub const LINK_UNCONFIRMED_TITLE: &str = "Couldn't confirm";

/// Verify email, its answer lost, under [`LINK_UNCONFIRMED_TITLE`].
pub const VERIFY_LOST: &str =
    "Reload the page: if it says the link was already used, your email is verified.";

/// Confirm email change, its answer lost, under [`LINK_UNCONFIRMED_TITLE`].
pub const CONFIRM_EMAIL_LOST: &str =
    "Reload the page: if it says the link was already used, your new email is confirmed.";
