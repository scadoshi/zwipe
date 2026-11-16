//! The settings changes sent with no reply yet, kept across launches so one
//! the app died before hearing back about goes up on the next launch (see
//! [`zwipe_ui::pending_settings`]).
//!
//! Same backends as [`theme_store`]: the keyring on Apple and desktop, a
//! file in Android private storage. Stored as the user's id beside the
//! sparse update the record sends.
//!
//! [`theme_store`]: crate::outbound::theme_store

use serde_json::{Value, json};
use uuid::Uuid;
use zwipe_core::domain::user::settings::PipStyle;
use zwipe_core::http::contracts::user::HttpUpdateSettings;
use zwipe_ui::pending_settings::PendingSettings;

/// The stored form.
fn to_stored(pending: &PendingSettings) -> anyhow::Result<Vec<u8>> {
    let stored = json!({ "user_id": pending.user_id(), "update": pending.update() });
    Ok(serde_json::to_vec(&stored)?)
}

/// Reads the stored form; `None` when it holds nothing to send.
fn from_stored(bytes: &[u8]) -> anyhow::Result<Option<PendingSettings>> {
    let mut stored: Value = serde_json::from_slice(bytes)?;
    let mut field = |name: &str| stored.get_mut(name).map_or_default(Value::take);
    let user_id: Uuid = serde_json::from_value(field("user_id"))?;
    let mut update = field("update");
    // A change stored by an earlier build may name its pip color the earlier
    // way. PipStyle's own serde still reads that name; the request takes only
    // the current one, so the color is written back under it first.
    if let Some(pip_style) = update.get_mut("pip_style")
        && let Ok(style) = serde_json::from_value::<PipStyle>(pip_style.clone())
    {
        *pip_style = Value::from(style.as_str());
    }
    let update: HttpUpdateSettings = serde_json::from_value(update)?;
    Ok(PendingSettings::from_update(user_id, update))
}

/// The pending changes, or `None` when there are none or they cannot be read.
pub fn load() -> Option<PendingSettings> {
    let read = platform::load().and_then(|bytes| bytes.map_or(Ok(None), |b| from_stored(&b)));
    match read {
        Ok(pending) => pending,
        Err(e) => {
            tracing::error!("failed to load the pending settings: {e}");
            None
        }
    }
}

/// Stores `pending`, or clears the store for `None`, logging a failure.
pub fn store(pending: Option<&PendingSettings>) {
    let written = match pending {
        Some(pending) => to_stored(pending).and_then(|bytes| platform::save(&bytes)),
        None => platform::delete(),
    };
    if let Err(e) = written {
        tracing::error!("failed to store the pending settings: {e}");
    }
}

// Apple / desktop: OS store via keyring (same service as the session).
#[cfg(not(target_os = "android"))]
mod platform {
    use crate::outbound::keyring_entry::{self, Entry};

    const SERVICE: &str = concat!(env!("CARGO_PKG_NAME"), "-service");
    const USERNAME: &str = concat!(env!("CARGO_PKG_NAME"), "-pending-settings");

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

    pub(crate) fn delete() -> anyhow::Result<()> {
        match entry()?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(e) => Err(e.into()),
        }
    }
}

// Android: a file in the app's private internal storage.
#[cfg(target_os = "android")]
mod platform {
    use std::path::PathBuf;

    use crate::outbound::android_fs;

    fn path() -> anyhow::Result<PathBuf> {
        Ok(android_fs::files_dir()?.join("pending_settings.json"))
    }

    pub(crate) fn save(bytes: &[u8]) -> anyhow::Result<()> {
        std::fs::write(path()?, bytes)?;
        Ok(())
    }

    pub(crate) fn load() -> anyhow::Result<Option<Vec<u8>>> {
        match std::fs::read(path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => Ok(Some(bytes)),
        }
    }

    pub(crate) fn delete() -> anyhow::Result<()> {
        match std::fs::remove_file(path()?) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_ui::settings::SettingChange;

    use super::*;

    #[test]
    fn a_record_reads_back_as_it_was_stored() {
        let user = Uuid::new_v4();
        let pending = PendingSettings::record(
            None,
            user,
            SettingChange::DarkMode {
                theme: "nord".to_owned(),
                dark_mode: true,
            },
        );
        let back = from_stored(&to_stored(&pending).unwrap()).unwrap();
        assert_eq!(back, Some(pending));
    }
    #[test]
    fn a_pip_color_stored_under_its_earlier_name_still_reads() {
        let user = Uuid::new_v4();
        for (stored, current) in [("natural", PipStyle::Classic), ("theme", PipStyle::Themed)] {
            let bytes = serde_json::to_vec(&json!({
                "user_id": user,
                "update": { "pip_style": stored },
            }))
            .unwrap();
            let back = from_stored(&bytes).unwrap().expect("a change to send");
            assert_eq!(back.update().pip_style, Some(current), "stored {stored}");
        }
    }
}
