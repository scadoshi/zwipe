use crate::domain::{
    auth::{
        models::{
            access_token::{AccessToken, JwtSecret},
            authenticate_user::{AuthenticateUser, AuthenticateUserError},
            change_email::{ChangeEmail, ChangeEmailError},
            change_password::{ChangePassword, ChangePasswordError},
            change_username::{ChangeUsername, ChangeUsernameError},
            delete_user::{DeleteUser, DeleteUserError},
            register_user::{RegisterUser, RegisterUserError},
            session::{
                create_session::{CreateSession, CreateSessionError},
                delete_expired_sessions::DeleteExpiredSessionsError,
                refresh_session::{RefreshSession, RefreshSessionError},
                revoke_sessions::{RevokeSessions, RevokeSessionsError},
                Session,
            },
            UserWithPasswordHash,
        },
        ports::{AuthRepository, AuthService},
    },
    user::{models::User, ports::UserRepository},
};
use anyhow::anyhow;

/// structure which implements `AuthService`
#[derive(Debug, Clone)]
pub struct Service<AR, UR>
where
    AR: AuthRepository,
    UR: UserRepository,
{
    auth_repo: AR,
    user_repo: UR,
    jwt_secret: JwtSecret,
}

impl<AR, UR> Service<AR, UR>
where
    AR: AuthRepository,
    UR: UserRepository,
{
    pub fn new(auth_repo: AR, user_repo: UR, jwt_secret: JwtSecret) -> Self {
        Self {
            auth_repo,
            user_repo,
            jwt_secret,
        }
    }
}

impl<AR, UR> AuthService for Service<AR, UR>
where
    AR: AuthRepository + Clone,
    UR: UserRepository + Clone,
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

        let access_token = AccessToken::generate(&user, &self.jwt_secret)
            .map_err(|e| RegisterUserError::FailedAccessToken(anyhow!("{e}")))?;

        Ok(Session {
            user,
            access_token,
            refresh_token,
        })
    }

    async fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> Result<Session, AuthenticateUserError> {
        let user_with_password_hash: UserWithPasswordHash =
            self.auth_repo.get_user_with_password_hash(request).await?;

        let password_hash = user_with_password_hash.password_hash.clone();

        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&request.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(e.into()))?;

        if !verified {
            return Err(AuthenticateUserError::InvalidPassword);
        }

        let access_token = AccessToken::generate(&user, &self.jwt_secret)
            .map_err(|e| AuthenticateUserError::FailedAccessToken(anyhow!("{e}")))?;

        let refresh_token = self.auth_repo.create_refresh_token(&user.id.into()).await?;

        Ok(Session {
            user,
            access_token,
            refresh_token,
        })
    }

    async fn create_session(&self, request: &CreateSession) -> Result<Session, CreateSessionError> {
        let user = self.user_repo.get_user(&request.user_id).await?;

        let refresh_token = self
            .auth_repo
            .create_refresh_token(&request.user_id)
            .await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session::new(user, access_token, refresh_token);

        Ok(session)
    }

    async fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> Result<Session, RefreshSessionError> {
        let user = self.user_repo.get_user(&request.user_id).await?;

        let refresh_token = self.auth_repo.use_refresh_token(&request).await?;

        let access_token = AccessToken::generate(&user, self.jwt_secret())?;

        let session = Session::new(user, access_token, refresh_token);

        Ok(session)
    }

    async fn revoke_sessions(&self, request: &RevokeSessions) -> Result<(), RevokeSessionsError> {
        self.auth_repo
            .delete_users_refresh_tokens(&request.user_id)
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

    async fn change_password(&self, request: &ChangePassword) -> Result<(), ChangePasswordError> {
        self.authenticate_user(&request.into()).await?;
        self.auth_repo.change_password(request).await
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
}
