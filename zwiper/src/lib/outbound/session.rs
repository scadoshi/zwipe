//! Session persistence.
//!
//! On Apple/desktop platforms the session lives in the OS secure store via the
//! `keyring` crate (Keychain on iOS/macOS, Credential Manager on Windows,
//! Secret Service on Linux).
//!
//! `keyring` has **no Android backend** — there it silently falls back to an
//! in-memory mock store that's lost on app restart (which is why Android wasn't
//! persisting sessions). So on Android we persist the session to a JSON file in
//! the app's **private internal storage** (`/data/data/<pkg>/files/`), which is
//! sandboxed per-app and survives restarts. The path is resolved through JNI
//! from the Android `Context` that `ndk-context` exposes.

use zwipe_core::domain::auth::models::session::Session;

/// Trait for persisting sessions across application restarts.
pub trait Persist {
    /// Saves the session to persistent storage.
    fn save(&self) -> anyhow::Result<()>;
    /// Saves the session, logging errors instead of returning them.
    fn infallible_save(&self);
    /// Loads a session from storage, returning `None` if not found or expired.
    fn load() -> anyhow::Result<Option<Session>>;
    /// Loads a session, logging errors and returning `None` on failure.
    fn infallible_load() -> Option<Session>;
    /// Deletes the session from storage.
    fn delete(&self) -> anyhow::Result<()>;
    /// Deletes the session, logging errors instead of returning them.
    fn infallible_delete(&self);
}

impl Persist for Session {
    fn save(&self) -> anyhow::Result<()> {
        platform::save(self)
    }

    fn infallible_save(&self) {
        if let Err(e) = self.save() {
            tracing::error!("failed to save session: {e}");
        }
    }

    fn load() -> anyhow::Result<Option<Self>> {
        platform::load()
    }

    fn infallible_load() -> Option<Session> {
        match Session::load() {
            Ok(session) => session,
            Err(e) => {
                tracing::error!("failed to load session: {e}");
                None
            }
        }
    }

    fn delete(&self) -> anyhow::Result<()> {
        platform::delete()
    }

    fn infallible_delete(&self) {
        if let Err(e) = self.delete() {
            tracing::error!("failed to delete session: {e}");
        }
    }
}

// ============================================================================
// Apple / desktop: OS secure store via keyring.
// ============================================================================
#[cfg(not(target_os = "android"))]
mod platform {
    use super::Session;
    use keyring::default;

    fn service() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-service"
    }

    fn username() -> String {
        env!("CARGO_PKG_NAME").to_string() + "-user"
    }

    pub fn save(session: &Session) -> anyhow::Result<()> {
        let credential =
            default::default_credential_builder().build(None, &service(), &username())?;
        let bytes = serde_json::to_vec(session)?;
        credential.set_secret(&bytes)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Session>> {
        let credential =
            default::default_credential_builder().build(None, &service(), &username())?;
        match credential.get_secret() {
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)?;
                if session.is_expired() {
                    credential.delete_credential()?;
                    return Ok(None);
                }
                Ok(Some(session))
            }
        }
    }

    pub fn delete() -> anyhow::Result<()> {
        let credential =
            default::default_credential_builder().build(None, &service(), &username())?;
        credential.delete_credential()?;
        Ok(())
    }
}

// ============================================================================
// Android: a JSON file in the app's private internal storage.
// ============================================================================
#[cfg(target_os = "android")]
mod platform {
    use super::Session;
    use jni::objects::{JObject, JString};
    use std::path::PathBuf;

    /// The app's private internal files dir (e.g. `/data/data/<pkg>/files`),
    /// resolved via JNI from the Android `Context` `ndk-context` exposes.
    fn files_dir() -> anyhow::Result<PathBuf> {
        let ctx = ndk_context::android_context();
        // SAFETY: ndk-context guarantees a valid JavaVM and Context jobject for
        // the running app.
        let vm = unsafe { jni::JavaVM::from_raw(ctx.vm().cast()) }?;
        let mut env = vm.attach_current_thread()?;
        let context = unsafe { JObject::from_raw(ctx.context().cast()) };

        // File dir = context.getFilesDir();
        let dir = env
            .call_method(&context, "getFilesDir", "()Ljava/io/File;", &[])?
            .l()?;
        // String path = dir.getAbsolutePath();
        let path = env
            .call_method(&dir, "getAbsolutePath", "()Ljava/lang/String;", &[])?
            .l()?;
        let path: String = env.get_string(&JString::from(path))?.into();
        Ok(PathBuf::from(path))
    }

    fn session_path() -> anyhow::Result<PathBuf> {
        Ok(files_dir()?.join("session.json"))
    }

    pub fn save(session: &Session) -> anyhow::Result<()> {
        let bytes = serde_json::to_vec(session)?;
        std::fs::write(session_path()?, bytes)?;
        Ok(())
    }

    pub fn load() -> anyhow::Result<Option<Session>> {
        let path = session_path()?;
        match std::fs::read(&path) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(None),
            Err(e) => Err(e.into()),
            Ok(bytes) => {
                let session: Session = serde_json::from_slice(&bytes)?;
                if session.is_expired() {
                    let _ = std::fs::remove_file(&path);
                    return Ok(None);
                }
                Ok(Some(session))
            }
        }
    }

    pub fn delete() -> anyhow::Result<()> {
        match std::fs::remove_file(session_path()?) {
            Err(e) if e.kind() == std::io::ErrorKind::NotFound => Ok(()),
            other => Ok(other?),
        }
    }
}
