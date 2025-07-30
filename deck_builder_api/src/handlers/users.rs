// // External crate imports
// use axum::{extract::State, http::StatusCode, response::Json};
// use diesel::{
//     prelude::*,
//     r2d2::{ConnectionManager, Pool},
//     PgConnection, RunQueryDsl,
// };
// use serde_json::{json, Value};

// // Internal crate imports (our code)
// use crate::models::users;
// use crate::schema::users;

// // define DbPool from the more complex type
// type DbPool = Pool<ConnectionManager<PgConnection>>;
