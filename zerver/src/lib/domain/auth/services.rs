use crate::domain::{
    auth::{
        models::{
            jwt::{Jwt, JwtCreationResponse, JwtSecret},
            AuthenticateUser, AuthenticateUserError, AuthenticateUserSuccess, ChangeEmail,
            ChangeEmailError, ChangePassword, ChangePasswordError, ChangeUsername,
            ChangeUsernameError, DeleteUser, DeleteUserError, RegisterUser, RegisterUserError,
            UserWithPasswordHash,
        },
        ports::{AuthRepository, AuthService},
    },
    user::models::User,
};
use anyhow::anyhow;

/// structure which implements `AuthService`
#[derive(Debug, Clone)]
pub struct Service<R: AuthRepository> {
    repo: R,
    jwt_secret: JwtSecret,
}

impl<R: AuthRepository> Service<R> {
    pub fn new(repo: R, jwt_secret: JwtSecret) -> Self {
        Self { repo, jwt_secret }
    }
}

impl<R: AuthRepository + Clone> AuthService for Service<R> {
    fn jwt_secret(&self) -> &JwtSecret {
        &self.jwt_secret
    }

    async fn register_user(
        &self,
        request: &RegisterUser,
    ) -> Result<AuthenticateUserSuccess, RegisterUserError> {
        let user = self.repo.create_user_with_password_hash(request).await?;

        let JwtCreationResponse {
            jwt: token,
            expires_at,
        } = Jwt::generate(user.id, user.email.clone(), &self.jwt_secret)
            .map_err(|e| RegisterUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccess {
            user,
            token,
            expires_at,
        })
    }

    async fn authenticate_user(
        &self,
        request: &AuthenticateUser,
    ) -> Result<AuthenticateUserSuccess, AuthenticateUserError> {
        let user_with_password_hash: UserWithPasswordHash =
            self.repo.get_user_with_password_hash(request).await?;

        let password_hash = user_with_password_hash.password_hash.clone();

        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&request.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(e.into()))?;

        if !verified {
            return Err(AuthenticateUserError::InvalidPassword);
        }

        let JwtCreationResponse {
            jwt: token,
            expires_at,
        } = Jwt::generate(user.id, user.email.clone(), &self.jwt_secret)
            .map_err(|e| AuthenticateUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccess {
            user,
            token,
            expires_at,
        })
    }

    async fn change_password(&self, request: &ChangePassword) -> Result<(), ChangePasswordError> {
        let _ = self.authenticate_user(&request.into()).await?;
        self.repo.change_password(request).await
    }

    async fn change_username(&self, request: &ChangeUsername) -> Result<User, ChangeUsernameError> {
        self.repo.change_username(request).await
    }

    async fn change_email(&self, request: &ChangeEmail) -> Result<User, ChangeEmailError> {
        self.repo.change_email(request).await
    }

    async fn delete_user(&self, request: &DeleteUser) -> Result<(), DeleteUserError> {
        self.repo.delete_user(request).await
    }
}
