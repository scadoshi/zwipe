use crate::domain::{
    auth::models::jwt::{Jwt, JwtCreationResponse, JwtSecret},
    user::{
        models::{
            User, UserAuthenticationError, UserAuthenticationRequest,
            UserAuthenticationSuccessResponse, UserCreationRequest, UserRegistrationError,
            UserWithPasswordHash,
        },
        ports::{UserRepository, UserService},
    },
};
use anyhow::anyhow;

#[derive(Debug, Clone)]
pub struct Service<R>
where
    R: UserRepository,
{
    repo: R,
    jwt_secret: JwtSecret,
}

impl<R> Service<R>
where
    R: UserRepository,
{
    pub fn new(repo: R, jwt_secret: JwtSecret) -> Self {
        Self { repo, jwt_secret }
    }
}

impl<R> UserService for Service<R>
where
    R: UserRepository,
{
    async fn register_user(
        &self,
        req: &UserCreationRequest,
        jwt_secret: JwtSecret,
    ) -> Result<UserAuthenticationSuccessResponse, UserRegistrationError> {
        let user = self.repo.create_user(req).await?;

        let JwtCreationResponse {
            jwt: token,
            expires_at: expires_at,
        } = Jwt::generate(user.id, user.email.clone(), jwt_secret)
            .map_err(|e| UserRegistrationError::FailedJwt(anyhow!("{e}")))?;

        Ok(UserAuthenticationSuccessResponse {
            user,
            token,
            expires_at,
        })
    }

    async fn authenticate_user(
        &self,
        req: &UserAuthenticationRequest,
        jwt_secret: JwtSecret,
    ) -> Result<UserAuthenticationSuccessResponse, UserAuthenticationError> {
        let user_with_password_hash: UserWithPasswordHash =
            self.repo.get_user_with_password_hash(req).await?;

        let password_hash = user_with_password_hash.password_hash.clone();
        let user: User = user_with_password_hash.into();

        let verified = password_hash
            .verify(&req.password)
            .map_err(|e| UserAuthenticationError::FailedToVerify(anyhow!("{e}")))?;

        if !verified {
            return Err(UserAuthenticationError::InvalidPassword);
        }

        let JwtCreationResponse {
            jwt: token,
            expires_at: expires_at,
        } = Jwt::generate(user.id, user.email.clone(), jwt_secret)
            .map_err(|e| UserAuthenticationError::FailedJwt(anyhow!("{e}")))?;

        Ok(UserAuthenticationSuccessResponse {
            user,
            token,
            expires_at,
        })
    }
}
