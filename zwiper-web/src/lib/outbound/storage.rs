//! A string key-value store: the browser's `localStorage` in the web build.

use std::cell::RefCell;
use std::collections::HashMap;

/// Why the store could not take a write.
#[derive(Debug, Clone, PartialEq, Eq, thiserror::Error)]
pub enum StorageError {
    /// The browser offers no `localStorage` (a sandboxed frame, or storage
    /// turned off).
    #[error("storage is unavailable")]
    Unavailable,
    /// The browser refused the write, such as over quota.
    #[error("storage refused the write: {0}")]
    Refused(String),
}

/// Where the session and theme live between visits.
pub trait Storage {
    /// The value under `key`, or `None` when there is none or the store
    /// cannot be read.
    fn read(&self, key: &str) -> Option<String>;

    /// Puts `value` under `key`.
    ///
    /// # Errors
    ///
    /// Fails when the store is missing or refuses the write.
    fn write(&self, key: &str, value: &str) -> Result<(), StorageError>;

    /// Drops whatever is under `key`. Removing a missing key succeeds.
    ///
    /// # Errors
    ///
    /// Fails when the store is missing or refuses the removal.
    fn remove(&self, key: &str) -> Result<(), StorageError>;
}

/// The browser's `localStorage`, which keeps its values for this origin
/// until the user clears site data.
///
/// Off wasm (the workspace lint and the tests build natively) there is no
/// browser: reads find nothing and writes fail with
/// [`StorageError::Unavailable`].
#[derive(Debug, Clone, Copy, Default)]
pub struct BrowserStorage;

#[cfg(target_arch = "wasm32")]
impl BrowserStorage {
    fn local() -> Result<web_sys::Storage, StorageError> {
        web_sys::window()
            .and_then(|window| window.local_storage().ok().flatten())
            .ok_or(StorageError::Unavailable)
    }
}

#[cfg(target_arch = "wasm32")]
impl Storage for BrowserStorage {
    fn read(&self, key: &str) -> Option<String> {
        Self::local().ok()?.get_item(key).ok().flatten()
    }

    fn write(&self, key: &str, value: &str) -> Result<(), StorageError> {
        Self::local()?
            .set_item(key, value)
            .map_err(|e| StorageError::Refused(format!("{e:?}")))
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        Self::local()?
            .remove_item(key)
            .map_err(|e| StorageError::Refused(format!("{e:?}")))
    }
}

#[cfg(not(target_arch = "wasm32"))]
impl Storage for BrowserStorage {
    fn read(&self, _key: &str) -> Option<String> {
        None
    }

    fn write(&self, _key: &str, _value: &str) -> Result<(), StorageError> {
        Err(StorageError::Unavailable)
    }

    fn remove(&self, _key: &str) -> Result<(), StorageError> {
        Err(StorageError::Unavailable)
    }
}

/// A store held in memory, for tests and anything else that must not touch
/// the browser.
#[derive(Debug, Default)]
pub struct MemoryStorage {
    values: RefCell<HashMap<String, String>>,
}

impl Storage for MemoryStorage {
    fn read(&self, key: &str) -> Option<String> {
        self.values.borrow().get(key).cloned()
    }

    fn write(&self, key: &str, value: &str) -> Result<(), StorageError> {
        self.values
            .borrow_mut()
            .insert(key.to_string(), value.to_string());
        Ok(())
    }

    fn remove(&self, key: &str) -> Result<(), StorageError> {
        self.values.borrow_mut().remove(key);
        Ok(())
    }
}
