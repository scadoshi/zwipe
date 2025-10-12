use std::future::Future;

use uuid::Uuid;

use crate::domain::{
    auth::models::{
        access_token::JwtSecret,
        authenticate_user::{AuthenticateUser, AuthenticateUserError},
        change_email::{ChangeEmail, ChangeEmailError},
        change_password::{ChangePassword, ChangePasswordError},
        change_username::{ChangeUsername, ChangeUsernameError},
        delete_user::{DeleteUser, DeleteUserError},
        refresh_token::RefreshToken,
        register_user::{RegisterUser, RegisterUserError},
        session::{
            CreateSession, CreateSessionError, DeleteExpiredSessionsError, RefreshSession,
            RefreshSessionError, RevokeSessions, RevokeSessionsError, Session,
        },
        UserWithPasswordHash,
    },
    user::models::User,
};

/// enables auth related database operations
pub trait AuthRepository: Clone + Send + Sync + 'static {
    // ========
    //  create
    // ========
    fn create_user_and_refresh_token(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<(User, RefreshToken), RegisterUserError>> + Send;

    fn create_refresh_token(
        &self,
        request: &Uuid,
    ) -> impl Future<Output = Result<RefreshToken, CreateSessionError>> + Send;

    fn use_refresh_token(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<RefreshToken, RefreshSessionError>> + Send;

    // =====
    //  get
    // =====
    fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

    // ========
    //  update
    // ========
    fn change_password(
        &self,
        request: &ChangePassword,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;

    fn change_username(
        &self,
        request: &ChangeUsername,
    ) -> impl Future<Output = Result<User, ChangeUsernameError>> + Send;

    fn change_email(
        &self,
        request: &ChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;

    // ========
    //  delete
    // ========
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    fn delete_expired_refresh_tokens(
        &self,
    ) -> impl Future<Output = Result<(), DeleteExpiredSessionsError>> + Send;

    fn delete_users_refresh_tokens(
        &self,
        user_id: &Uuid,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;
}

/// orchestrates auth related operations
pub trait AuthService: Clone + Send + Sync + 'static {
    // ========
    //  config
    // ========
    fn jwt_secret(&self) -> &JwtSecret;

    // ========
    //  create
    // ========
    fn register_user(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<Session, RegisterUserError>> + Send;

    fn create_session(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, CreateSessionError>> + Send;

    fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<Session, RefreshSessionError>> + Send;

    fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<Session, AuthenticateUserError>> + Send;

    // ========
    //  update
    // ========
    fn change_password(
        &self,
        request: &ChangePassword,
    ) -> impl Future<Output = Result<(), ChangePasswordError>> + Send;

    fn change_username(
        &self,
        request: &ChangeUsername,
    ) -> impl Future<Output = Result<User, ChangeUsernameError>> + Send;

    fn change_email(
        &self,
        request: &ChangeEmail,
    ) -> impl Future<Output = Result<User, ChangeEmailError>> + Send;

    // ========
    //  delete
    // ========
    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;

    fn delete_expired_sessions(
        &self,
    ) -> impl Future<Output = Result<(), DeleteExpiredSessionsError>> + Send;

    fn revoke_sessions(
        &self,
        request: &RevokeSessions,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;
}
