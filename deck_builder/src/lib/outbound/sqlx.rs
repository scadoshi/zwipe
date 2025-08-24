pub mod auth;
pub mod card;
pub mod health;
pub mod postgres;
pub mod user;

// thinking about building something like this
// to limit commits in certain functions in the card module
// it's just a wrapper struct for transaction to limit where commits can occur

// use sqlx::{PgPool, PgTransaction};
// pub struct TxWoComm<'a>(PgTransaction<'a>);

// impl<'a> TxWoComm<'a> {
//     fn tx(&mut self) -> &'a mut PgTransaction<'a> {
//         &mut self.0
//     }

//     pub async fn new(pool: PgPool) -> Result<Self, sqlx::Error> {
//         let tx = pool.begin().await?;
//         Ok(Self(tx))
//     }
// }
