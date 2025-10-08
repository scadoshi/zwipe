use std::future::Future;

use crate::domain::{
    auth::models::{
        access_token::JwtSecret,
        session::{
            CreateSession, CreateSessionError, RefreshSession, RefreshSessionError, RevokeSessions,
            RevokeSessionsError, Session,
        },
        AuthenticateUser, AuthenticateUserError, ChangeEmail, ChangeEmailError, ChangePassword,
        ChangePasswordError, ChangeUsername, ChangeUsernameError, DeleteUser, DeleteUserError,
        RegisterUser, RegisterUserError, UserWithPasswordHash,
    },
    user::models::User,
};

/// enables auth related database operations
pub trait AuthRepository: Clone + Send + Sync + 'static {
    fn create_user_with_password_hash(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<User, RegisterUserError>> + Send;

    fn create_session(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, CreateSessionError>> + Send;

    fn refresh_session(
        &self,
        request: &RefreshSession,
    ) -> impl Future<Output = Result<Session, RefreshSessionError>> + Send;

    fn revoke_sessions(
        &self,
        request: &RevokeSessions,
    ) -> impl Future<Output = Result<(), RevokeSessionsError>> + Send;

    fn get_user_with_password_hash(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<UserWithPasswordHash, AuthenticateUserError>> + Send;

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

    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}

/// orchestrates auth related operations
pub trait AuthService: Clone + Send + Sync + 'static {
    fn jwt_secret(&self) -> &JwtSecret;

    fn register_user(
        &self,
        request: &RegisterUser,
    ) -> impl Future<Output = Result<Session, RegisterUserError>> + Send;

    fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> impl Future<Output = Result<Session, AuthenticateUserError>> + Send;

    fn create_session(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, CreateSessionError>> + Send;

    fn refresh_session(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, RefreshSessionError>> + Send;

    fn revoke_sessions(
        &self,
        request: &CreateSession,
    ) -> impl Future<Output = Result<Session, RevokeSessionsError>> + Send;

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

    fn delete_user(
        &self,
        request: &DeleteUser,
    ) -> impl Future<Output = Result<(), DeleteUserError>> + Send;
}
