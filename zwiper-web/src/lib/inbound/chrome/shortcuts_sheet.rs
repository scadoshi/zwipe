//! The web's screens offer their keys to components' shortcuts sheet: each [`ShortcutsScreen`]'s
//! groups and note, a swipe screen's swipes named as its dock names them.

use components::keys::{self, ShortcutsList};
pub use components::keys::{
    ShortcutsSheet, use_global_keys, use_shortcuts_action, use_shortcuts_root,
};

use crate::inbound::chrome::shortcuts::{self, ShortcutsScreen};

/// Offers `screen`'s keys while it is mounted, so Shift+H opens the shortcuts sheet with them.
/// Elsewhere it lists only the keys every page answers.
pub fn use_offer_shortcuts(screen: ShortcutsScreen) {
    keys::use_offer_shortcuts(ShortcutsList {
        groups: shortcuts::sheet_groups(&screen),
        note: shortcuts::sheet_note(&screen),
        claims_steps: shortcuts::claims_steps(&screen),
    });
}
