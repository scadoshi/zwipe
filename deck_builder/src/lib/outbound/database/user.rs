use std::future::Future;

use crate::domain::{
    models::user::{User, UserCreationError, UserCreationRequest},
    ports::repositories::user::UserRepository,
};
use crate::outbound::database::postgres::Postgres;

impl UserRepository for Postgres {
    async fn create_user(&self, req: &UserCreationRequest) -> Result<User, UserCreationError> {
        let mut _tx = self
            .pool
            .begin()
            .await
            .unwrap_or_else(|e| panic!("failed to start PostgreSQL transaction: {}", e));

        //
        //
        //
        todo!()
    }
}
