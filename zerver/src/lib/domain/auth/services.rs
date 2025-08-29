use crate::domain::{
    auth::{
        models::{
            jwt::{Jwt, JwtCreationResponse, JwtSecret},
            AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserSuccessResponse,
            ChangePasswordError, ChangePasswordRequest, RegisterUserError, RegisterUserRequest,
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
        request: &RegisterUserRequest,
    ) -> Result<AuthenticateUserSuccessResponse, RegisterUserError> {
        let user = self.repo.create_user_with_password_hash(request).await?;

        let JwtCreationResponse {
            jwt: token,
            expires_at,
        } = Jwt::generate(user.id, user.email.clone(), &self.jwt_secret)
            .map_err(|e| RegisterUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccessResponse {
            user,
            token,
            expires_at,
        })
    }

    async fn authenticate_user(
        &self,
        request: &AuthenticateUserRequest,
    ) -> Result<AuthenticateUserSuccessResponse, AuthenticateUserError> {
        let user_with_password_hash: UserWithPasswordHash =
            self.repo.get_user_with_password_hash(request).await?;

        let password_hash = user_with_password_hash
            .password_hash
            .clone()
            .ok_or(AuthenticateUserError::InvalidPassword)?;

        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&request.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(anyhow!("{e}")))?;

        if !verified {
            return Err(AuthenticateUserError::InvalidPassword);
        }

        let JwtCreationResponse {
            jwt: token,
            expires_at,
        } = Jwt::generate(user.id, user.email.clone(), &self.jwt_secret)
            .map_err(|e| AuthenticateUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccessResponse {
            user,
            token,
            expires_at,
        })
    }

    async fn change_password(
        &self,
        request: &ChangePasswordRequest,
    ) -> Result<(), ChangePasswordError> {
        self.repo.change_password(request).await
    }
}
