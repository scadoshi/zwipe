//! The deck and card caps that depend on whether an account's email is verified, plus the commander maybeboard's cap.
//!
//! The server enforces every one of them. Clients read them to warn before a request the server would refuse.

/// Maximum total card quantity in a single deck, **counting all boards**
/// (mainboard + maybeboard + sideboard), for verified accounts. Counting every
/// board keeps an unbounded maybeboard/sideboard from evading the cap.
pub const MAX_CARDS_PER_DECK: i64 = 500;

/// Maximum decks for accounts with an unverified email address.
pub const UNVERIFIED_MAX_DECKS_PER_USER: i64 = 1;

/// Same per-deck card cap (all boards counted) for accounts with an unverified
/// email address.
pub const UNVERIFIED_MAX_CARDS_PER_DECK: i64 = 100;

/// Maximum entries on a user's commander maybeboard. Over-cap adds are
/// rejected (422), not evicted: every entry is a deliberate save.
pub const MAX_COMMANDER_MAYBEBOARD: i64 = 50;
