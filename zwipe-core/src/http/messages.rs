//! The server's exact text for errors a user caused and can fix. zerver sends
//! these unchanged; clients match them to put plainer words in front of the
//! user. Changing one here changes what the server sends, so a client still
//! matching the old text falls back to showing it raw.

/// An Archidekt import whose link holds no deck id.
pub const ARCHIDEKT_BAD_LINK: &str = "could not parse an archidekt deck id from the url";

/// An Archidekt deck that is missing or private.
pub const ARCHIDEKT_NOT_FOUND: &str =
    "deck not found on archidekt, make sure it's public and the url is correct";

/// An Archidekt deck past the import's size cap.
pub const ARCHIDEKT_TOO_LARGE: &str = "deck from archidekt is too large to import";

/// An add or import that would take a deck past its card limit.
pub const CARD_LIMIT: &str =
    "card limit reached (mainboard, maybeboard, and sideboard all count toward it)";

/// [`CARD_LIMIT`] for an account whose email isn't verified, which has a
/// lower limit.
pub const UNVERIFIED_CARD_LIMIT: &str =
    "card limit reached across all boards, verify your email to unlock more";

/// A create or rename to a deck name the user already has.
pub const DECK_NAME_TAKEN: &str = "deck with name and user combination already exists";

/// A clone under a deck name the user already has.
pub const CLONE_NAME_TAKEN: &str = "a deck with that name already exists";

/// A registration with a username someone else has.
pub const USERNAME_TAKEN: &str = "username already taken";

/// A username change to one someone else has.
pub const USERNAME_IN_USE: &str = "username already in use";

/// An email change confirmed for an address another account has.
pub const EMAIL_IN_USE: &str = "that address is already in use by another account";

/// An add of a card the deck already holds on that board.
pub const CARD_ALREADY_IN_DECK: &str = "card and deck combination already exist";

/// A password past the server's verify cap.
pub const PASSWORD_TOO_LONG: &str = "password too long";
