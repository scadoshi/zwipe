//! HTTP API client for backend communication.
//!
//! Provides the [`ZwipeClient`] struct and sub-modules for each API domain:
//! authentication, cards, decks, deck cards, and user management.

/// Authentication endpoints (login, logout, register, refresh).
pub mod auth;
/// The one place an API request is built, sent and decoded.
pub mod call;
/// Card data endpoints (search, get, types, artists, sets, languages).
pub mod card;
/// Changelog endpoint (release history).
pub mod changelog;
/// Deck CRUD operations.
pub mod deck;
/// Deck-card relationship operations (add/remove cards from decks).
pub mod deck_card;
/// Client-side error type (transport failures + wire-error decoding).
pub mod error;
/// User metrics operations.
pub mod metrics;
/// User profile operations.
pub mod user;
/// Client version gate operations.
pub mod version;

pub use error::ClientError;
use reqwest::{Client, Url};

/// HTTP client for the ZWIPE backend API.
///
/// The base URL arrives at construction: the crate reads no environment and
/// owns no default, so each app supplies the value the way it already sources
/// it (zwiper from its `.env`, zite from a const).
#[derive(Debug, Clone)]
pub struct ZwipeClient {
    /// The underlying HTTP client.
    pub client: Client,
    /// Scheme, host and port every request path is joined onto.
    pub base_url: Url,
}

impl ZwipeClient {
    /// Builds a client pointed at `base_url`.
    pub fn new(base_url: Url) -> Self {
        Self {
            client: Client::new(),
            base_url,
        }
    }
}
