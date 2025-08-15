use axum::{
    async_trait,
    extract::FromRequestParts,
    http::{request::Parts, StatusCode},
};

use axum_extra::{
    headers::{authorization::Bearer, Authorization},
    TypedHeader,
};

use crate::adapters::AppState;
use crate::domain::auth::jwt::validate_jwt;

pub struct AuthenticatedUser {
    pub user_id: i32,
    pub email: String,
}

#[async_trait]
impl FromRequestParts<AppState> for AuthenticatedUser {
    type Rejection = StatusCode;

    async fn from_request_parts(
        parts: &mut Parts,
        state: &AppState,
    ) -> Result<Self, Self::Rejection> {
        let TypedHeader(Authorization(bearer)) =
            TypedHeader::<Authorization<Bearer>>::from_request_parts(parts, state)
                .await
                // this is set to bad request because
                // the structuring of this variable
                // requires the type to follow the `Aurhorization<Bearer>`
                // pattern to result in an `Ok(TypedHeader)`
                // (technically a TypedHeader<Authorization<Bearer>>)
                // so the request has to have the "Authorization" header
                // and that auth header has to be of type "Bearer"
                .map_err(|_| StatusCode::BAD_REQUEST)?;

        // since this is us actually validating
        // the given token
        // error means unauthorized
        let claims = validate_jwt(bearer.token(), &state.jwt_config.secret)
            .map_err(|_| StatusCode::UNAUTHORIZED)?;

        Ok(AuthenticatedUser {
            user_id: claims.user_id,
            email: claims.email,
        })
    }
}

#[cfg(test)]
mod tests {
    // Test valid JWT extraction
    // Test missing Authorization header (400)
    // Test malformed Bearer token (400)
    // Test invalid JWT signature (401)
    // Test expired JWT (401)
}
