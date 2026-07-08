use crate::domain::{
    auth::{
        models::{
            UserWithPasswordHash,
            access_token::{AccessTokenExt, JwtSecret},
            password::{HashedPassword, Password},
        },
        ports::{AuthRepository, AuthService},
        requests::{
            authenticate_user::{AuthenticateUser, AuthenticateUserError},
            change_email::{ChangeEmail, ChangeEmailError},
            change_password::{ChangePassword, ChangePasswordError},
            change_username::{ChangeUsername, ChangeUsernameError},
            create_session::{CreateSession, CreateSessionError},
            delete_expired_sessions::DeleteExpiredSessionsError,
            delete_user::{DeleteUser, DeleteUserError},
            refresh_session::{RefreshSession, RefreshSessionError},
            register_user::{RegisterUser, RegisterUserError},
            request_password_reset::{RequestPasswordReset, RequestPasswordResetError},
            reset_password::{ResetPassword, ResetPasswordError},
            revoke_sessions::{RevokeSessions, RevokeSessionsError},
            verify_email::{VerifyEmail, VerifyEmailError},
        },
    },
    email::{models::SendEmail, ports::EmailSender},
    user::ports::UserRepository,
};
use anyhow::anyhow;
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::Digest;
use std::sync::LazyLock;
use uuid::Uuid;
use zwipe_core::domain::{
    auth::models::{access_token::AccessToken, session::Session},
    user::{User, preferences::UserPreferences},
};

/// A throwaway Argon2 hash used to equalize login timing for non-existent
/// accounts. Generated lazily with the same default Argon2 params as real
/// hashes, so verifying against it costs the same as verifying a real one. It
/// never matches any input — its only job is to burn equivalent CPU so a
/// missing account isn't measurably faster to reject than a real one with a
/// wrong password (which would leak whether the account exists — enumeration).
#[allow(clippy::expect_used)]
static TIMING_EQUALIZER_HASH: LazyLock<HashedPassword> = LazyLock::new(|| {
    Password::new("Zw1pe!Dummy#Hash7")
        .expect("timing-equalizer dummy password must satisfy the password policy")
        .hash()
        .expect("timing-equalizer dummy password must hash")
});

/// Authentication service implementation handling user registration, login, and session management.
///
/// This service orchestrates authentication operations by coordinating between:
/// - **AuthRepository**: Password hashing, refresh token rotation, credential updates
/// - **UserRepository**: User data retrieval
/// - **EmailSender**: Transactional email delivery (verification, password reset)
/// - **JWT Secret**: Access token generation
///
/// # Authorization Pattern
/// Operations that modify user data (change username/email/password, delete user)
/// require password re-authentication for security, even with a valid session.
#[derive(Debug, Clone)]
pub struct Service<AR, UR, ES>
where
    AR: AuthRepository,
    UR: UserRepository,
    ES: EmailSender,
{
    auth_repo: AR,
    user_repo: UR,
    email_sender: ES,
    jwt_secret: JwtSecret,
    /// Public web base URL for building verify/reset links (e.g. `https://zwipe.net`).
    web_base_url: String,
    /// User-facing support email address shown in transactional emails.
    support_email: String,
}

impl<AR, UR, ES> Service<AR, UR, ES>
where
    AR: AuthRepository,
    UR: UserRepository,
    ES: EmailSender,
{
    /// Creates a new authentication service with the provided repositories,
    /// email sender, JWT secret, public web base URL, and support email.
    pub fn new(
        auth_repo: AR,
        user_repo: UR,
        email_sender: ES,
        jwt_secret: JwtSecret,
        web_base_url: String,
        support_email: String,
    ) -> Self {
        Self {
            auth_repo,
            user_repo,
            email_sender,
            jwt_secret,
            web_base_url,
            support_email,
        }
    }

    /// Sends a security notification email after a profile change.
    ///
    /// Fire-and-forget: the change has already been committed, so a delivery
    /// failure is logged but never surfaced to the caller.
    async fn send_change_notification(
        &self,
        user_id: Uuid,
        to_email: &str,
        subject: &str,
        html_body: String,
    ) {
        if let Err(e) = self
            .email_sender
            .send_email(SendEmail {
                to: to_email.to_string(),
                subject: subject.to_string(),
                html_body,
            })
            .await
        {
            tracing::error!(event = "email_send_failure", user_id = %user_id, error = %e);
        }
    }

    /// Re-verifies a user's password for a sensitive operation (change
    /// email/username/password, delete account) **without** touching the login
    /// lockout and **without** minting a session.
    ///
    /// Account lockout is a *login* control. Coupling it to these endpoints
    /// (which are already tightly rate-limited per user) would let a stolen
    /// access token lock the owner out of login, and let a user lock themselves
    /// out by mistyping their current password. Brute-force protection here is
    /// the per-route rate limit, not the lockout. Unlike `authenticate_user`,
    /// this neither increments `failed_login_attempts` nor issues tokens.
    async fn verify_password(
        &self,
        request: &AuthenticateUser,
    ) -> Result<User, AuthenticateUserError> {
        let user_with_password_hash = self.auth_repo.get_user_with_password_hash(request).await?;
        let password_hash = user_with_password_hash.password_hash.clone();
        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&request.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(e.into()))?;

        if !verified {
            return Err(AuthenticateUserError::InvalidPassword);
        }

        Ok(user)
    }
}

/// Minimal HTML escape for user-controlled values injected into email templates.
///
/// Usernames have no character whitelist (only length/whitespace/profanity rules),
/// so `<`, `>`, and `&` are all possible.
fn escape_html(value: &str) -> String {
    value
        .replace('&', "&amp;")
        .replace('<', "&lt;")
        .replace('>', "&gt;")
        .replace('"', "&quot;")
}

/// Generates a (raw, hash) hex-token pair.
///
/// 32 random bytes → hex-encode → raw token sent to client.
/// SHA-256 hash of raw → stored in database.
fn generate_hex_token() -> (String, String) {
    let mut bytes = [0u8; 32];
    rand::rng().fill_bytes(&mut bytes);
    let raw = hex::encode(bytes);
    let hash = hex::encode(sha2::Sha256::digest(raw.as_bytes()));
    (raw, hash)
}

impl<AR, UR, ES> AuthService for Service<AR, UR, ES>
where
    AR: AuthRepository + Clone,
    UR: UserRepository + Clone,
    ES: EmailSender,
{
    // ========
    //  config
    // ========
    fn jwt_secret(&self) -> &JwtSecret {
        &self.jwt_secret
    }

    // ========
    //  create
    // ========
    async fn register_user(&self, request: &RegisterUser) -> Result<Session, RegisterUserError> {
        let (user, refresh_token) = self
            .auth_repo
            .create_user_and_refresh_token(request)
            .await?;

        let preferences = UserPreferences::default();

        let access_token = AccessToken::generate(&user, &self.jwt_secret)
            .map_err(|e| RegisterUserError::FailedAccessToken(anyhow!("{e}")))?;

        // Fire-and-forget: don't fail registration if email sending fails.
        if let Err(e) = self
            .send_verification_email(user.id, user.email.as_ref())
            .await
        {
            tracing::error!(event = "verification_email_failed", user_id = %user.id, error = %e);
        }

        Ok(Session {
            user,
            access_token,
            refresh_token,
            preferences,
        })
    }

    async fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> Result<Session, AuthenticateUserError> {
        let user_with_password_hash: UserWithPasswordHash = match self
            .auth_repo
            .get_user_with_password_hash(request)
            .await
        {
            Ok(user) => user,
            Err(AuthenticateUserError::UserNotFound) => {
                // Equalize timing with the wrong-password path: run an Argon2
                // verify against a dummy hash so a non-existent account isn't
                // measurably faster to reject (prevents username/email
                // enumeration via response timing). Result is discarded — it
                // never matches.
                let _ = TIMING_EQUALIZER_HASH.verify(&request.password);
                tracing::warn!(event = "login_failure", reason = "user_not_found", identifier = %request.identifier);
                return Err(AuthenticateUserError::UserNotFound);
            }
            Err(e) => return Err(e),
        };

        // Check lockout before Argon2 — avoids expensive hashing for locked accounts.
        if let Some(until) = user_with_password_hash.lockout_until
            && until > Utc::now()
        {
            tracing::warn!(event = "login_failure", reason = "account_locked", identifier = %request.identifier);
            return Err(AuthenticateUserError::AccountLocked);
        }

        let password_hash = user_with_password_hash.password_hash.clone();

        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&request.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(e.into()))?;

        if !verified {
            self.auth_repo.increment_failed_attempts(user.id).await?;
            tracing::warn!(event = "login_failure", reason = "invalid_password", identifier = %request.identifier);
            return Err(AuthenticateUserError::InvalidPassword);
        }

        self.auth_repo.reset_failed_attempts(user.id).await?;
        tracing::info!(event = "login_success", identifier = %request.identifier);

        let preferences = self
            .user_repo
            .get_preferences(user.id)
            .await
            .unwrap_or_default();

        let access_token = AccessToken::generate(&user, &self.jwt_secret)
            .map_err(|e| AuthenticateUserError::FailedAccessToken(anyhow!("{e}")))?;

        let refresh_token = self.auth_repo.create_refresh_token(user.id).await?;

        Ok(Session {
            user,
            access_token,
            refresh_token,
            preferences,
        })
    }

    async fn create_session(&self, request: &CreateSession) -> Result<Session, CreateSessionError> {
        let user = self.user_repo.get_user(request.user_id).await?;
        let preferences = self
            .user_repo
            .get_preferences(request.user_id)
            .await
            .unwrap_or_default();

        let refresh_token = self.auth_repo.create_refresh_token(request.user_id).await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session {
            user,
            access_token,
            refresh_token,
            preferences,
        };

        Ok(session)
    }

    async fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> Result<Session, RefreshSessionError> {
        let user = self.user_repo.get_user(request.user_id).await?;
        let preferences = self
            .user_repo
            .get_preferences(request.user_id)
            .await
            .unwrap_or_default();

        let refresh_token = self.auth_repo.use_refresh_token(request).await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session {
            user,
            access_token,
            refresh_token,
            preferences,
        };

        Ok(session)
    }

    async fn revoke_sessions(&self, request: &RevokeSessions) -> Result<(), RevokeSessionsError> {
        self.auth_repo
            .delete_users_refresh_tokens(request.user_id)
            .await?;
        Ok(())
    }

    // ========
    //  update
    // ========
    async fn change_username(&self, request: &ChangeUsername) -> Result<User, ChangeUsernameError> {
        let old_username = self.verify_password(&request.into()).await?.username;
        let user = self.auth_repo.change_username(request).await?;

        let html = include_str!("email_templates/username_changed.html")
            .replace("{old_username}", &escape_html(&old_username))
            .replace("{new_username}", &escape_html(&user.username));
        self.send_change_notification(
            user.id,
            user.email.as_ref(),
            "Your Zwipe username was changed",
            html,
        )
        .await;

        Ok(user)
    }

    async fn change_email(&self, request: &ChangeEmail) -> Result<User, ChangeEmailError> {
        let old_email = self.verify_password(&request.into()).await?.email;
        let user = self.auth_repo.change_email(request).await?;

        // Notify the OLD address: the new one is what an attacker would control.
        let html = include_str!("email_templates/email_changed.html")
            .replace("{new_email}", &escape_html(user.email.as_ref()))
            .replace("{support_email}", &escape_html(&self.support_email));
        self.send_change_notification(
            user.id,
            old_email.as_ref(),
            "Your Zwipe email was changed",
            html,
        )
        .await;

        Ok(user)
    }

    async fn change_password_and_revoke_sessions(
        &self,
        request: &ChangePassword,
    ) -> Result<(), ChangePasswordError> {
        let user = self.verify_password(&request.into()).await?;
        self.auth_repo
            .change_password_and_revoke_sessions(request)
            .await?;

        let html = include_str!("email_templates/password_changed.html").to_string();
        self.send_change_notification(
            user.id,
            user.email.as_ref(),
            "Your Zwipe password was changed",
            html,
        )
        .await;

        Ok(())
    }

    // ========
    //  delete
    // ========
    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        self.verify_password(&request.into()).await?;
        self.auth_repo.delete_user(request).await
    }

    async fn delete_expired_sessions(&self) -> Result<(), DeleteExpiredSessionsError> {
        self.auth_repo.delete_expired_refresh_tokens().await
    }

    // ========================
    //  email verification
    // ========================

    async fn send_verification_email(
        &self,
        user_id: Uuid,
        to_email: &str,
    ) -> Result<(), anyhow::Error> {
        self.auth_repo
            .delete_email_verification_tokens(user_id)
            .await?;

        let (raw, hash) = generate_hex_token();
        let expires_at = Utc::now() + Duration::hours(24);

        self.auth_repo
            .store_email_verification_token(user_id, hash, expires_at)
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let link = format!("{}/verify/{raw}", self.web_base_url);
        // Dev convenience: surface the link + raw token in the server log so a
        // developer can verify an email without a working email provider. Debug
        // builds only — a release build never logs a live verification token.
        #[cfg(debug_assertions)]
        tracing::warn!(
            event = "dev_email_verification",
            %user_id,
            verify_link = %link,
            raw_token = %raw,
            "DEV ONLY: email verification link (or POST the raw_token to /api/auth/verify-email)"
        );
        let html = include_str!("email_templates/verify_email.html").replace("{link}", &link);

        self.email_sender
            .send_email(SendEmail {
                to: to_email.to_string(),
                subject: "Verify your Zwipe email".to_string(),
                html_body: html,
            })
            .await
            .map_err(|e| anyhow!("{e}"))?;

        Ok(())
    }

    async fn verify_email(&self, request: &VerifyEmail) -> Result<(), VerifyEmailError> {
        let hash = hex::encode(sha2::Sha256::digest(request.token.as_bytes()));
        let user_id = self.auth_repo.use_email_verification_token(&hash).await?;
        self.auth_repo.mark_email_verified(user_id).await?;
        tracing::info!(event = "email_verified", user_id = %user_id);
        Ok(())
    }

    // ========================
    //  password reset
    // ========================

    async fn request_password_reset(
        &self,
        request: &RequestPasswordReset,
    ) -> Result<(), RequestPasswordResetError> {
        let user_id = match self.auth_repo.get_user_id_by_email(&request.email).await? {
            Some(id) => id,
            // Silently return Ok — never reveal whether an email is registered
            None => {
                tracing::debug!(event = "password_reset_unknown_email");
                return Ok(());
            }
        };

        if self
            .auth_repo
            .is_password_reset_on_cooldown(user_id)
            .await?
        {
            tracing::debug!(event = "password_reset_cooldown", user_id = %user_id);
            return Ok(());
        }

        self.auth_repo.delete_password_reset_tokens(user_id).await?;

        let (raw, hash) = generate_hex_token();
        let expires_at = Utc::now() + Duration::minutes(15);

        self.auth_repo
            .store_password_reset_token(user_id, hash, expires_at)
            .await?;

        let link = format!("{}/reset/{raw}", self.web_base_url);
        // Dev convenience: same as verification — surface the reset link + token
        // so password reset is testable without an email provider. Debug only.
        #[cfg(debug_assertions)]
        tracing::warn!(
            event = "dev_password_reset",
            %user_id,
            reset_link = %link,
            raw_token = %raw,
            "DEV ONLY: password reset link (or POST the raw_token to /api/auth/reset-password)"
        );
        let html = include_str!("email_templates/reset_password.html").replace("{link}", &link);

        if let Err(e) = self
            .email_sender
            .send_email(SendEmail {
                to: request.email.clone(),
                subject: "Reset your Zwipe password".to_string(),
                html_body: html,
            })
            .await
        {
            tracing::error!(event = "email_send_failure", error = %e);
        }

        tracing::info!(event = "password_reset_requested", user_id = %user_id);
        Ok(())
    }

    async fn reset_password(
        &self,
        request: &ResetPassword,
    ) -> Result<uuid::Uuid, ResetPasswordError> {
        let hash = hex::encode(sha2::Sha256::digest(request.token.as_bytes()));
        let user_id = self.auth_repo.use_password_reset_token(&hash).await?;
        self.auth_repo
            .reset_password_and_revoke_sessions(user_id, request.new_password_hash.clone())
            .await?;
        tracing::info!(event = "password_reset_success", user_id = %user_id);
        Ok(user_id)
    }
}

#[cfg(test)]
mod tests {
    use super::TIMING_EQUALIZER_HASH;

    #[test]
    fn timing_equalizer_hash_is_valid_and_never_matches() {
        // Forces lazy init (so a policy-violating dummy fails here, not at first
        // login) and confirms it verifies like a real hash: Ok(false), not Err.
        assert!(matches!(
            TIMING_EQUALIZER_HASH.verify("definitely-not-the-dummy"),
            Ok(false)
        ));
    }
}
