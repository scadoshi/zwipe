//! The signed-in account: the API client, the session it holds, and the file that keeps the session between runs.

use std::fs;
use std::io::Write;
use std::path::{Path, PathBuf};

use anyhow::{Context, Result};
use chrono::Duration;
use tokio::sync::Mutex;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::{HttpAuthenticateUser, HttpRefreshSession};

/// How long before it expires an access token counts as stale, as in the app: the server's 60-second leeway plus a clock up to a minute slow.
const REFRESH_AHEAD: Duration = Duration::minutes(2);

/// The folder inside the user's config directory that holds the session.
const APP_DIR_NAME: &str = "zwipe-tui";

/// The API client and the current session, shared by every background request.
#[derive(Debug)]
pub(crate) struct Account {
    pub(crate) client: ZwipeClient,
    /// Also the refresh lock: holding it while refreshing makes concurrent requests wait for one refresh.
    session: Mutex<Option<Session>>,
    store: SessionFile,
}

impl Account {
    /// An account over `client`, signed in with the saved session when it has not expired.
    pub(crate) fn new(client: ZwipeClient, store: SessionFile) -> Self {
        let session = store.load().filter(|s| !s.is_expired());
        Self {
            client,
            session: Mutex::new(session),
            store,
        }
    }

    /// The current session, if any, without refreshing it.
    pub(crate) async fn session(&self) -> Option<Session> {
        self.session.lock().await.clone()
    }

    /// Signs in and saves the session.
    ///
    /// # Errors
    ///
    /// Returns the [`ClientError`] the server answered with, such as wrong credentials.
    pub(crate) async fn sign_in(
        &self,
        identifier: &str,
        password: &str,
    ) -> Result<Session, ClientError> {
        let session = self
            .client
            .authenticate_user(HttpAuthenticateUser::new(identifier, password))
            .await?;
        self.store.save(&session).ok();
        *self.session.lock().await = Some(session.clone());
        Ok(session)
    }

    /// A session whose access token is good for at least [`REFRESH_AHEAD`], refreshing it once for all waiting callers.
    ///
    /// # Errors
    ///
    /// Returns [`ClientError::Unauthorized`] when there is no session, or the server rejects the refresh token, which also signs the account out. A refresh that fails for another reason, such as the network, keeps the session for the next try and returns that error.
    pub(crate) async fn fresh(&self) -> Result<Session, ClientError> {
        let mut guard = self.session.lock().await;
        let Some(current) = guard.clone() else {
            return Err(ClientError::Unauthorized("not signed in".to_owned()));
        };
        if !current.access_token.expires_within(REFRESH_AHEAD) {
            return Ok(current);
        }
        if current.is_expired() {
            *guard = None;
            self.store.clear().ok();
            return Err(ClientError::Unauthorized("session expired".to_owned()));
        }

        let request =
            HttpRefreshSession::new(&current.user.id.to_string(), &current.refresh_token.value);
        match self.client.refresh(&request).await {
            Ok(session) => {
                self.store.save(&session).ok();
                *guard = Some(session.clone());
                Ok(session)
            }
            Err(
                e @ (ClientError::Unauthorized(_)
                | ClientError::Forbidden(_)
                | ClientError::NotFound(_)),
            ) => {
                *guard = None;
                self.store.clear().ok();
                Err(ClientError::Unauthorized(e.to_string()))
            }
            Err(e) => Err(e),
        }
    }

    /// Ends the session on the server, best effort, and forgets it here.
    pub(crate) async fn sign_out(&self) {
        let session = self.session.lock().await.take();
        if let Some(session) = session {
            self.client.logout(&session).await.ok();
        }
        self.store.clear().ok();
    }
}

/// The session saved between runs: `session.json`, readable by the owner only on Unix.
#[derive(Debug, Clone)]
pub(crate) struct SessionFile {
    path: PathBuf,
}

impl SessionFile {
    /// `session.json` in `zwipe-tui` under the user's config directory: `~/.config` on Linux, `~/Library/Application Support` on macOS.
    ///
    /// # Errors
    ///
    /// Fails when the system has no config directory.
    pub(crate) fn in_config_dir() -> Result<Self> {
        let dir = dirs::config_dir()
            .context("could not find the config directory")?
            .join(APP_DIR_NAME);
        Ok(Self::at(dir.join("session.json")))
    }

    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> Option<Session> {
        serde_json::from_str(&fs::read_to_string(&self.path).ok()?).ok()
    }

    fn save(&self, session: &Session) -> Result<()> {
        if let Some(dir) = self.path.parent()
            && !dir.exists()
        {
            fs::create_dir_all(dir)?;
            restrict(dir, 0o700)?;
        }
        let mut options = fs::OpenOptions::new();
        options.write(true).create(true).truncate(true);
        #[cfg(unix)]
        {
            use std::os::unix::fs::OpenOptionsExt;
            options.mode(0o600);
        }
        if self.path.exists() {
            restrict(&self.path, 0o600)?;
        }
        options
            .open(&self.path)?
            .write_all(serde_json::to_string(session)?.as_bytes())?;
        Ok(())
    }

    fn clear(&self) -> Result<()> {
        match fs::remove_file(&self.path) {
            Err(e) if e.kind() != std::io::ErrorKind::NotFound => Err(e.into()),
            _ => Ok(()),
        }
    }
}

/// Sets `path`'s permission bits on Unix; elsewhere does nothing.
#[cfg_attr(
    not(unix),
    expect(clippy::unnecessary_wraps, reason = "matches the Unix signature")
)]
fn restrict(path: &Path, mode: u32) -> Result<()> {
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        fs::set_permissions(path, fs::Permissions::from_mode(mode))?;
    }
    #[cfg(not(unix))]
    let _ = (path, mode);
    Ok(())
}

#[cfg(all(test, unix))]
mod tests {
    use std::os::unix::fs::PermissionsExt;

    use super::*;

    fn temp_file(name: &str) -> SessionFile {
        let dir = std::env::temp_dir().join(format!("zwipe-tui-{name}-{}", std::process::id()));
        SessionFile::at(dir.join("session.json"))
    }

    #[test]
    fn clearing_a_missing_session_is_fine() {
        assert!(temp_file("missing").clear().is_ok());
    }

    #[test]
    fn a_missing_or_corrupt_file_loads_as_no_session() {
        let file = temp_file("corrupt");
        assert!(file.load().is_none());
        fs::create_dir_all(file.path.parent().unwrap()).unwrap();
        fs::write(&file.path, "not json").unwrap();
        assert!(file.load().is_none());
        fs::remove_dir_all(file.path.parent().unwrap()).unwrap();
    }

    #[test]
    fn restrict_sets_owner_only_bits() {
        let path = std::env::temp_dir().join(format!("zwipe-tui-mode-{}", std::process::id()));
        fs::write(&path, "x").unwrap();
        restrict(&path, 0o600).unwrap();
        let mode = fs::metadata(&path).unwrap().permissions().mode() & 0o777;
        fs::remove_file(&path).unwrap();
        assert_eq!(mode, 0o600);
    }
}
