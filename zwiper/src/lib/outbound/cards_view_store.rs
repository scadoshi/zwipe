//! The deck cards screen's List | Stacks choice, kept across launches.
//!
//! Same backends as [`count_scope_store`]: the keyring on Apple and desktop,
//! a file in Android private storage. Never deleted on logout; it is a
//! display choice, not account data.
//!
//! [`count_scope_store`]: crate::outbound::count_scope_store

use zwipe_ui::deck_cards::stacks::CardsView;

/// Saves `view`, logging errors instead of returning them.
pub fn infallible_save(view: CardsView) {
    if let Err(e) = platform::save(view.key().as_bytes()) {
        tracing::error!("failed to save the cards view: {e}");
    }
}

/// The saved view, or the List when none is saved or it cannot be read.
pub fn restored() -> CardsView {
    match platform::load() {
        Ok(bytes) => bytes.map_or(CardsView::List, |b| from_stored(&b)),
        Err(e) => {
            tracing::error!("failed to load the cards view: {e}");
            CardsView::List
        }
    }
}

/// Reads the stored form; anything unreadable is the List.
fn from_stored(bytes: &[u8]) -> CardsView {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(CardsView::from_key)
        .unwrap_or(CardsView::List)
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use crate::outbound::keyring_entry::{self, Entry};

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");
    const USERNAME: &str = concat!(env!("CARGO_PKG_NAME"), "-cards-view");

    fn entry() -> Result<Entry, keyring::Error> {
        keyring_entry::entry(SERVICE, USERNAME)
    }

    pub(crate) fn save(bytes: &[u8]) -> anyhow::Result<()> {
        entry()?.set_secret(bytes)?;
        Ok(())
    }

    pub(crate) fn load() -> anyhow::Result<Option<Vec<u8>>> {
        match entry()?.get_secret() {
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

    fn view_path() -> anyhow::Result<PathBuf> {
        Ok(android_fs::files_dir()?.join("cards_view"))
    }

    pub(crate) fn save(bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::write(view_path()?, bytes)?;
        Ok(())
    }

    pub(crate) fn load() -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(view_path()?) {
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
        for view in [CardsView::List, CardsView::Stacks] {
            assert_eq!(from_stored(view.key().as_bytes()), view);
        }
    }

    #[test]
    fn an_unreadable_stored_view_is_the_list() {
        assert_eq!(from_stored(b"???"), CardsView::List);
        assert_eq!(from_stored(&[0xff, 0xfe]), CardsView::List);
    }
}
