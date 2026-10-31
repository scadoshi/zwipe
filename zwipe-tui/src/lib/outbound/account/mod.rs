//! The signed-in account: the API client, the session it holds, and the file that keeps the session between runs.

use std::fs;
use std::path::PathBuf;

use anyhow::Result;
use chrono::Duration;
use tokio::sync::Mutex;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::{User, UserPreferences};
use zwipe_core::http::contracts::auth::{HttpAuthenticateUser, HttpRefreshSession};

use super::counts_file::CountsFile;
use super::files;

/// How long logging out waits on the server before forgetting the session anyway, from the app's `LOGOUT_TIMEOUT`.
const LOGOUT_TIMEOUT: std::time::Duration = std::time::Duration::from_secs(5);

/// How long before it expires an access token counts as stale, as in the app: the server's 60-second leeway plus a clock up to a minute slow.
const REFRESH_AHEAD: Duration = Duration::minutes(2);

/// The API client and the current session, shared by every background request.
#[derive(Debug)]
pub(crate) struct Account {
    pub(crate) client: ZwipeClient,
    /// One random id for this launch, which anonymous funnel events carry instead of an account.
    pub(crate) launch_id: Uuid,
    /// Also the refresh lock: holding it while refreshing makes concurrent requests wait for one refresh.
    session: Mutex<Option<Session>>,
    store: SessionFile,
    /// Everybody's counts as last fetched, beside the session file.
    pub(crate) counts: CountsFile,
}

impl Account {
    /// An account over `client`, signed in with the saved session when it has not expired.
    pub(crate) fn new(client: ZwipeClient, store: SessionFile) -> Self {
        let session = store.load().filter(|s| !s.is_expired());
        Self {
            client,
            launch_id: Uuid::new_v4(),
            session: Mutex::new(session),
            counts: CountsFile::at(store.path.with_file_name("counts.json")),
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

    /// Replaces the session's copy of the user, such as after a username change, and saves it.
    pub(crate) async fn set_user(&self, user: User) {
        let mut guard = self.session.lock().await;
        if let Some(session) = guard.as_mut() {
            session.user = user;
            self.store.save(session).ok();
        }
    }

    /// Replaces the session's copy of the preferences, so the next launch opens in them, and saves it.
    pub(crate) async fn set_preferences(&self, preferences: UserPreferences) {
        let mut guard = self.session.lock().await;
        if let Some(session) = guard.as_mut() {
            session.preferences = preferences;
            self.store.save(session).ok();
        }
    }

    /// Forgets the session here without telling the server, for an account the server has already deleted.
    pub(crate) async fn forget(&self) {
        self.session.lock().await.take();
        self.store.clear().ok();
    }

    /// Ends the session on the server, waiting at most [`LOGOUT_TIMEOUT`], then forgets it here whether the server answered or not.
    pub(crate) async fn sign_out(&self) {
        let logout = async {
            if let Ok(session) = self.fresh().await {
                self.client.logout(&session).await.ok();
            }
        };
        tokio::time::timeout(LOGOUT_TIMEOUT, logout).await.ok();
        self.session.lock().await.take();
        self.store.clear().ok();
    }
}

/// The session saved between runs: `session.json`, readable by the owner only.
#[derive(Debug, Clone)]
pub(crate) struct SessionFile {
    path: PathBuf,
}

impl SessionFile {
    /// `session.json` in the app's config folder.
    ///
    /// # Errors
    ///
    /// Fails when the system has no config directory.
    pub(crate) fn in_config_dir() -> Result<Self> {
        Ok(Self::at(files::app_dir()?.join("session.json")))
    }

    pub(crate) fn at(path: PathBuf) -> Self {
        Self { path }
    }

    fn load(&self) -> Option<Session> {
        serde_json::from_str(&fs::read_to_string(&self.path).ok()?).ok()
    }

    fn save(&self, session: &Session) -> Result<()> {
        files::write_private(&self.path, &serde_json::to_string(session)?)
    }

    fn clear(&self) -> Result<()> {
        files::remove(&self.path)
    }
}

#[cfg(all(test, unix))]
mod tests {
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
}
