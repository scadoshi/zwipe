use crate::adapters::database::postgres::Postgres;
use crate::domain::{
    models::user::{NewUser, NewUserError, User},
    repositories::user::UserRepository,
};

impl UserRepository for Postgres {
    async fn create_user(&self, req: &NewUser) -> Result<User, NewUserError> {
        let mut tx = self
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
