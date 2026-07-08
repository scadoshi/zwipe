//! Extracted components for card screens.

/// Undo/redo action history for card swipe operations.
pub(crate) mod action_history;
/// Per-deck parking for the add screen's search stack.
pub(crate) mod add_stack_cache;
/// Card info display and skeleton components.
pub(crate) mod card_info;
/// Expandable card row component.
pub(crate) mod card_row;
/// Contained swipe-stack state (cards, cursor, history, enter animation).
pub(crate) mod card_stack;
/// Card image with built-in flip control for double-faced cards.
pub(crate) mod flippable_card_image;
/// Fullscreen image preview overlay.
pub(crate) mod image_preview;
/// Keyword chips with inline reminder reveal.
pub(crate) mod keyword_chips;
/// Oracle text rendered with Mana-font symbol glyphs.
pub(crate) mod oracle_text;
/// Bottom sheet for selecting card printings.
pub(crate) mod printing_sheet;
