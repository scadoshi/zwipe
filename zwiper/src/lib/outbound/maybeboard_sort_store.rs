//! The commander maybeboard's sort, kept across launches.
//!
//! Same backends as [`count_scope_store`]: the keyring on Apple and desktop,
//! a file in Android private storage. Never deleted on logout; it is a
//! display choice, not account data.
//!
//! [`count_scope_store`]: crate::outbound::count_scope_store

use zwipe_ui::commander::maybeboard::Sort;

/// Saves `sort`, logging errors instead of returning them.
pub fn infallible_save(sort: Sort) {
    if let Err(e) = platform::save(sort.key().as_bytes()) {
        tracing::error!("failed to save the maybeboard sort: {e}");
    }
}

/// The saved sort, or the default (Newest saved) when none is saved or it
/// cannot be read.
pub fn restored() -> Sort {
    match platform::load() {
        Ok(bytes) => bytes.map_or_else(Sort::default, |b| from_stored(&b)),
        Err(e) => {
            tracing::error!("failed to load the maybeboard sort: {e}");
            Sort::default()
        }
    }
}

/// Reads the stored form; anything unreadable is the default.
fn from_stored(bytes: &[u8]) -> Sort {
    std::str::from_utf8(bytes)
        .ok()
        .and_then(Sort::from_key)
        .unwrap_or_default()
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use crate::outbound::keyring_entry::{self, Entry};

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");
    const USERNAME: &str = concat!(env!("CARGO_PKG_NAME"), "-maybeboard-sort");

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

    fn sort_path() -> anyhow::Result<PathBuf> {
        Ok(android_fs::files_dir()?.join("maybeboard_sort"))
    }

    pub(crate) fn save(bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::write(sort_path()?, bytes)?;
        Ok(())
    }

    pub(crate) fn load() -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(sort_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(bytes)),
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::search_card::card_filter::sort_direction::SortDirection;
    use zwipe_ui::commander::maybeboard::SortBy;

    use super::*;

    #[test]
    fn a_sort_reads_back_as_it_was_stored() {
        let sort = Sort {
            by: SortBy::Color,
            direction: SortDirection::Descending,
        };
        assert_eq!(from_stored(sort.key().as_bytes()), sort);
    }

    #[test]
    fn an_unreadable_stored_sort_is_the_default() {
        assert_eq!(from_stored(b"???"), Sort::default());
        assert_eq!(from_stored(&[0xff, 0xfe]), Sort::default());
    }
}
