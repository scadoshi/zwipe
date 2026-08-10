//! Platform keyring entry, shared by `session` + `theme_store`.
//!
//! On macOS/desktop this is keyring's `v1` wrapper `Entry`, which installs the
//! platform store itself. On iOS the wrapper is a dead end twice over: its
//! store selection has no iOS arm (`Entry::new` errors at runtime), and the
//! Apple store crate compiles for iOS only with its `protected` feature (the
//! "protected data" store — the one secure store iOS has). So iOS installs
//! that store into `keyring-core` here, once, and uses the core `Entry`
//! directly. The two `Entry` types share their method surface and error type
//! (`keyring::Error` is a re-export of `keyring_core::Error`), so callers
//! don't cfg.
//!
//! Continuity: the protected store keys items as generic passwords by
//! (service, account) via the same Security-framework calls the keyring-3 iOS
//! backend used, so sessions written by 1.7.5 stay readable after upgrade —
//! the iOS sibling of the keyring-3→4 macOS bridge proof (2026-08-06).

#[cfg(not(target_os = "ios"))]
pub use keyring::Entry;
#[cfg(target_os = "ios")]
pub use keyring_core::Entry;

/// Opens the entry for `(service, user)`, installing the iOS store first if
/// this is the process's first use.
pub fn entry(service: &str, user: &str) -> Result<Entry, keyring::Error> {
    #[cfg(target_os = "ios")]
    {
        use std::sync::Once;
        static INSTALL: Once = Once::new();
        INSTALL.call_once(|| {
            match apple_native_keyring_store::protected::Store::new() {
                Ok(store) => keyring_core::set_default_store(store),
                // Entry::new below then fails with NoDefaultStore; callers
                // already treat entry errors as "no persisted value".
                Err(e) => tracing::error!("failed to install iOS keychain store: {e}"),
            }
        });
    }
    Entry::new(service, user)
}
