//! The concrete service types `AppState` holds: each service over its Postgres repositories.
//!
//! Handlers name these aliases, so no sqlx type reaches the HTTP layer.

use crate::domain::{auth, card, deck, health, metrics, user};
use crate::outbound::sqlx::postgres::Postgres;

/// Registration, login, sessions and account changes.
pub type Auth = auth::services::Service<Postgres, Postgres>;
/// User profiles, settings and hints.
pub type Users = user::services::Service<Postgres>;
/// Database health checks.
pub type Health = health::services::Service<Postgres>;
/// Card search and lookups.
pub type Cards = card::services::Service<Postgres>;
/// Decks and their cards.
pub type Decks = deck::services::Service<Postgres, Postgres>;
/// Usage events, crash reports and counters.
pub type Metrics = metrics::services::Service<Postgres>;
