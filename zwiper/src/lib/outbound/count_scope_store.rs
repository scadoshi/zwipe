//! Which counts the home hero shows, everyone's or the user's own, kept
//! across launches.
//!
//! Same backends as [`counts_store`]: the keyring on Apple and desktop, a
//! file in Android private storage. Never deleted on logout; it is a
//! display choice, not account data.
//!
//! [`counts_store`]: crate::outbound::counts_store

/// Whose counts the home hero shows.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub enum CountScope {
    /// The sums across every user.
    #[default]
    AllUsers,
    /// The signed-in user's own counts.
    JustMe,
}

impl CountScope {
    /// The other scope.
    #[must_use]
    pub fn toggled(self) -> Self {
        match self {
            Self::AllUsers => Self::JustMe,
            Self::JustMe => Self::AllUsers,
        }
    }

    /// The label on the hero's scope button.
    #[must_use]
    pub fn label(self) -> &'static str {
        match self {
            Self::AllUsers => "All players",
            Self::JustMe => "Only me",
        }
    }

    /// The stored form.
    fn as_stored(self) -> &'static [u8] {
        match self {
            Self::AllUsers => b"all",
            Self::JustMe => b"mine",
        }
    }

    /// Reads the stored form; anything else is everyone's counts.
    fn from_stored(bytes: &[u8]) -> Self {
        if bytes == Self::JustMe.as_stored() {
            Self::JustMe
        } else {
            Self::AllUsers
        }
    }

    /// Saves the scope, logging errors instead of returning them.
    pub fn infallible_save(self) {
        if let Err(e) = platform::save(self.as_stored()) {
            tracing::error!("failed to save the count scope: {e}");
        }
    }

    /// The saved scope, or everyone's counts when none is saved or it cannot
    /// be read.
    pub fn restored() -> Self {
        match platform::load() {
            Ok(bytes) => bytes.map_or(Self::AllUsers, |b| Self::from_stored(&b)),
            Err(e) => {
                tracing::error!("failed to load the count scope: {e}");
                Self::AllUsers
            }
        }
    }
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use crate::outbound::keyring_entry::{self, Entry};

    fn service() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-service"
    }

    fn username() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-count-scope"
    }

    fn entry() -> Result<Entry, keyring::Error> {
        keyring_entry::entry(&service(), &username())
    }

    pub fn save(bytes: &[u8]) -> anyhow::Result<()> {
        entry()?.set_secret(bytes)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Vec<u8>>> {
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

    use crate::outbound::android_fs::files_dir;

    fn scope_path() -> anyhow::Result<PathBuf> {
        Ok(files_dir()?.join("count_scope"))
    }

    pub fn save(bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::write(scope_path()?, bytes)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(scope_path()?) {
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
    fn a_scope_reads_back_as_it_was_stored() {
        for scope in [CountScope::AllUsers, CountScope::JustMe] {
            assert_eq!(CountScope::from_stored(scope.as_stored()), scope);
        }
    }

    #[test]
    fn an_unknown_stored_scope_is_everyone() {
        assert_eq!(CountScope::from_stored(b"???"), CountScope::AllUsers);
    }

    #[test]
    fn toggling_twice_returns_the_scope() {
        assert_eq!(CountScope::JustMe.toggled().toggled(), CountScope::JustMe);
    }
}
