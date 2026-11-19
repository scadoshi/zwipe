//! The List | Stacks choice, kept across launches: the deck cards screen's, and Collection's
//! apart from it.
//!
//! Same backends as [`count_scope_store`]: the keyring on Apple and desktop,
//! a file in Android private storage. Never deleted on logout; it is a
//! display choice, not account data.
//!
//! [`count_scope_store`]: crate::outbound::count_scope_store

use zwipe_ui::deck_cards::stacks::CardsView;
use zwipe_ui::screens::collection_list::stacks::DEFAULT_VIEW as COLLECTION_DEFAULT;

/// The screen a view is kept for.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Slot {
    /// The deck cards screen, on the List until picked.
    Deck,
    /// Collection, on [`COLLECTION_DEFAULT`] until picked.
    Collection,
}

impl Slot {
    /// The view before the user picks one, and for one stored unreadably.
    const fn default_view(self) -> CardsView {
        match self {
            Self::Deck => CardsView::List,
            Self::Collection => COLLECTION_DEFAULT,
        }
    }

    /// Where the slot is stored: the keyring user on Apple and desktop, the file on Android.
    const fn name(self) -> &'static str {
        match self {
            Self::Deck => "cards_view",
            Self::Collection => "collection_cards_view",
        }
    }
}

/// Saves the deck cards screen's `view`, logging errors instead of returning them.
pub fn infallible_save(view: CardsView) {
    save(Slot::Deck, view);
}

/// The deck cards screen's saved view, or the List when none is saved or it cannot be read.
pub fn restored() -> CardsView {
    restore(Slot::Deck)
}

/// Saves Collection's `view`, logging errors instead of returning them.
pub fn infallible_save_collection(view: CardsView) {
    save(Slot::Collection, view);
}

/// Collection's saved view, or [`COLLECTION_DEFAULT`] when none is saved or it cannot be read.
pub fn restored_collection() -> CardsView {
    restore(Slot::Collection)
}

fn save(slot: Slot, view: CardsView) {
    if let Err(e) = platform::save(slot.name(), view.key().as_bytes()) {
        tracing::error!("failed to save the cards view: {e}");
    }
}

fn restore(slot: Slot) -> CardsView {
    match platform::load(slot.name()) {
        Ok(bytes) => bytes.map_or(slot.default_view(), |b| from_stored(slot, &b)),
        Err(e) => {
            tracing::error!("failed to load the cards view: {e}");
            slot.default_view()
        }
    }
}

/// Reads the stored form; anything unreadable is the slot's default.
fn from_stored(slot: Slot, bytes: &[u8]) -> CardsView {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(CardsView::from_key)
        .unwrap_or(slot.default_view())
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use crate::outbound::keyring_entry::{self, Entry};

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");

    /// The keyring user for `slot`: the deck screen's keeps its original name.
    fn entry(slot: &str) -> Result<Entry, keyring::Error> {
        let user = format!("{}-{}", env!("CARGO_PKG_NAME"), slot.replace('_', "-"));
        keyring_entry::entry(SERVICE, &user)
    }

    pub(crate) fn save(slot: &str, bytes: &[u8]) -> anyhow::Result<()> {
        entry(slot)?.set_secret(bytes)?;
        Ok(())
    }

    pub(crate) fn load(slot: &str) -> anyhow::Result<Option<Vec<u8>>> {
        match entry(slot)?.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(bytes)),
        }
    }
}

// Android: a file in the app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use std::path::PathBuf;

    use crate::outbound::android_fs;

    fn view_path(slot: &str) -> anyhow::Result<PathBuf> {
        Ok(android_fs::files_dir()?.join(slot))
    }

    pub(crate) fn save(slot: &str, bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::write(view_path(slot)?, bytes)?;
        Ok(())
    }

    pub(crate) fn load(slot: &str) -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(view_path(slot)?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(bytes)),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_view_reads_back_as_it_was_stored() {
        for slot in [Slot::Deck, Slot::Collection] {
            for view in [CardsView::List, CardsView::Stacks] {
                assert_eq!(from_stored(slot, view.key().as_bytes()), view);
            }
        }
    }

    #[test]
    fn an_unreadable_stored_view_is_the_slots_default() {
        assert_eq!(from_stored(Slot::Deck, b"???"), CardsView::List);
        assert_eq!(from_stored(Slot::Deck, &[0xff, 0xfe]), CardsView::List);
        assert_eq!(from_stored(Slot::Collection, b"???"), CardsView::Stacks);
    }

    #[test]
    fn the_deck_screen_keeps_its_stored_name_and_collection_has_its_own() {
        assert_eq!(Slot::Deck.name(), "cards_view");
        assert_ne!(Slot::Collection.name(), Slot::Deck.name());
    }
}
