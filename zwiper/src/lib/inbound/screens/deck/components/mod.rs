//! Extracted components for deck screens.

/// Clone deck dialog with text input for the new deck name.
pub(crate) mod clone_deck_dialog;
/// Deck chart visualizations for the view screen.
pub(crate) mod deck_charts;
/// Shared deck name, format selector, and commander search for create/edit screens.
pub(crate) mod deck_fields;
/// Deck profile info and warnings section for the view screen.
pub(crate) mod deck_profile;
/// Deck stats summary section for the view screen.
pub(crate) mod deck_stats;
/// Deck warnings section with remove buttons for card-specific warnings.
pub(crate) mod deck_warnings;
/// Buy sheet and more actions bottom sheets for the view screen.
pub(crate) mod more_buttons;
/// Skeleton placeholders shown while deck data loads.
pub(crate) mod skeletons;
/// Full-screen "Zwipe select" command-zone swipe picker.
pub(crate) mod swipe_select;
/// Full-screen tag picker with definitions.
pub(crate) mod tag_select;
