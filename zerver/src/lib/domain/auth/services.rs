use crate::domain::{
    auth::{
        models::{
            UserWithPasswordHash,
            access_token::{AccessTokenExt, JwtSecret},
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
    email::{
        models::SendEmail,
        ports::EmailSender,
    },
    user::ports::UserRepository,
};
use anyhow::anyhow;
use zwipe_core::domain::{
    auth::models::{access_token::AccessToken, session::Session},
    user::{preferences::UserPreferences, User},
};
use chrono::{Duration, Utc};
use rand::Rng;
use sha2::Digest;
use uuid::Uuid;

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
}

impl<AR, UR, ES> Service<AR, UR, ES>
where
    AR: AuthRepository,
    UR: UserRepository,
    ES: EmailSender,
{
    /// Creates a new authentication service with the provided repositories, email sender, and JWT secret.
    pub fn new(auth_repo: AR, user_repo: UR, email_sender: ES, jwt_secret: JwtSecret) -> Self {
        Self {
            auth_repo,
            user_repo,
            email_sender,
            jwt_secret,
        }
    }
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
        if let Err(e) = self.send_verification_email(user.id, user.email.as_ref()).await {
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
        let user_with_password_hash: UserWithPasswordHash =
            self.auth_repo.get_user_with_password_hash(request).await?;

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

        let preferences = self.user_repo.get_preferences(user.id).await.unwrap_or_default();

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
        let preferences = self.user_repo.get_preferences(request.user_id).await.unwrap_or_default();

        let refresh_token = self.auth_repo.create_refresh_token(request.user_id).await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session { user, access_token, refresh_token, preferences };

        Ok(session)
    }

    async fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> Result<Session, RefreshSessionError> {
        let user = self.user_repo.get_user(request.user_id).await?;
        let preferences = self.user_repo.get_preferences(request.user_id).await.unwrap_or_default();

        let refresh_token = self.auth_repo.use_refresh_token(request).await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session { user, access_token, refresh_token, preferences };

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
        self.authenticate_user(&request.into()).await?;
        self.auth_repo.change_username(request).await
    }

    async fn change_email(&self, request: &ChangeEmail) -> Result<User, ChangeEmailError> {
        self.authenticate_user(&request.into()).await?;
        self.auth_repo.change_email(request).await
    }

    async fn change_password_and_revoke_sessions(&self, request: &ChangePassword) -> Result<(), ChangePasswordError> {
        self.authenticate_user(&request.into()).await?;
        self.auth_repo.change_password_and_revoke_sessions(request).await
    }

    // ========
    //  delete
    // ========
    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        self.authenticate_user(&request.into()).await?;
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
        self.auth_repo.delete_email_verification_tokens(user_id).await?;

        let (raw, hash) = generate_hex_token();
        let expires_at = Utc::now() + Duration::hours(24);

        self.auth_repo
            .store_email_verification_token(user_id, hash, expires_at)
            .await
            .map_err(|e| anyhow!("{e}"))?;

        let link = format!("https://zwipe.net/verify/{raw}");
        let html = include_str!("email_templates/verify_email.html")
            .replace("{link}", &link);

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

        if self.auth_repo.is_password_reset_on_cooldown(user_id).await? {
            tracing::debug!(event = "password_reset_cooldown", user_id = %user_id);
            return Ok(());
        }

        self.auth_repo.delete_password_reset_tokens(user_id).await?;

        let (raw, hash) = generate_hex_token();
        let expires_at = Utc::now() + Duration::minutes(15);

        self.auth_repo
            .store_password_reset_token(user_id, hash, expires_at)
            .await?;

        let link = format!("https://zwipe.net/reset/{raw}");
        let html = include_str!("email_templates/reset_password.html")
            .replace("{link}", &link);

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

    async fn reset_password(&self, request: &ResetPassword) -> Result<uuid::Uuid, ResetPasswordError> {
        let hash = hex::encode(sha2::Sha256::digest(request.token.as_bytes()));
        let user_id = self.auth_repo.use_password_reset_token(&hash).await?;
        self.auth_repo
            .reset_password_and_revoke_sessions(user_id, request.new_password_hash.clone())
            .await?;
        tracing::info!(event = "password_reset_success", user_id = %user_id);
        Ok(user_id)
    }
}
