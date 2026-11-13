//! The `data-action` values on the card buttons a keyboard shortcut presses. A key finds its
//! button by this attribute, so a button's words can change without turning the key off.

/// The card details' Image button.
pub const IMAGE: &str = "image";

/// A card's flip control: the details' Flip button and the card image's own.
pub const FLIP: &str = "flip";

/// The card details' Printings button.
pub const PRINTINGS: &str = "printings";

/// A screen's Undo button.
pub const UNDO: &str = "undo";

/// A meld half's Meld toggle: the details' and the card image's own.
pub const MELD: &str = "meld";

/// A deck card's Star toggle.
pub const STAR: &str = "star";

/// The menu's palette button, which opens the theme sheet: orangui's mark, which its page keys'
/// `t` presses too.
pub const THEMES: &str = orangui::THEMES_ACTION;

/// What the flip key presses, the first the card shows: its flip control, else, a card with one
/// face, its Meld toggle.
pub const FLIP_OR_MELD: [&str; 2] = [FLIP, MELD];
