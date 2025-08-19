use anyhow::anyhow;

use crate::domain::{
    auth::{
        models::{
            jwt::{Jwt, JwtCreationResponse, JwtSecret},
            AuthenticateUserError, AuthenticateUserRequest, AuthenticateUserSuccessResponse,
            RegisterUserError, RegisterUserRequest, UserWithPasswordHash,
        },
        ports::{AuthRepository, AuthService},
    },
    user::models::User,
};

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: AuthRepository,
{
    repo: R,
    jwt_secret: JwtSecret,
}

impl<R> Service<R>
where
    R: AuthRepository,
{
    pub fn new(repo: R, jwt_secret: JwtSecret) -> Self {
        Self { repo, jwt_secret }
    }
}

impl<R> AuthService for Service<R>
where
    R: AuthRepository + Clone,
{
    async fn register_user(
        &self,
        req: &RegisterUserRequest,
        jwt_secret: JwtSecret,
    ) -> Result<AuthenticateUserSuccessResponse, RegisterUserError> {
        let user = self.repo.create_user_with_password_hash(req).await?;

        let JwtCreationResponse {
            jwt: token,
            expires_at: expires_at,
        } = Jwt::generate(user.id, user.email.clone(), jwt_secret)
            .map_err(|e| RegisterUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccessResponse {
            user,
            token,
            expires_at,
        })
    }

    async fn authenticate_user(
        &self,
        req: &AuthenticateUserRequest,
        jwt_secret: JwtSecret,
    ) -> Result<AuthenticateUserSuccessResponse, AuthenticateUserError> {
        let user_with_password_hash: UserWithPasswordHash =
            self.repo.get_user_with_password_hash(req).await?;

        let password_hash = user_with_password_hash.password_hash.clone();
        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&req.password)
            .map_err(|e| AuthenticateUserError::FailedToVerify(anyhow!("{e}")))?;

        if !verified {
            return Err(AuthenticateUserError::InvalidPassword);
        }

        let JwtCreationResponse {
            jwt: token,
            expires_at: expires_at,
        } = Jwt::generate(user.id, user.email.clone(), jwt_secret)
            .map_err(|e| AuthenticateUserError::FailedJwt(anyhow!("{e}")))?;

        Ok(AuthenticateUserSuccessResponse {
            user,
            token,
            expires_at,
        })
    }
}
