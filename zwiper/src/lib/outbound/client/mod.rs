//! HTTP API client for backend communication.
//!
//! Provides the [`ZwipeClient`] struct and sub-modules for each API domain:
//! authentication, cards, decks, deck cards, and user management.

/// Authentication endpoints (login, logout, register, refresh).
pub mod auth;
/// Card data endpoints (search, get, types, artists, sets, languages).
pub mod card;
/// Deck CRUD operations.
pub mod deck;
/// Deck-card relationship operations (add/remove cards from decks).
pub mod deck_card;
/// User profile operations.
pub mod user;

use crate::config::Config;
use reqwest::Client;
use std::sync::OnceLock;

/// HTTP client for communicating with the ZWIPE backend API.
#[derive(Debug, Clone)]
pub struct ZwipeClient {
    /// The underlying HTTP client.
    pub client: Client,
    /// Application configuration (API URLs, etc.).
    pub app_config: Config,
}

impl Default for ZwipeClient {
    fn default() -> Self {
        static CONFIG: OnceLock<Config> = OnceLock::new();
        let app_config = CONFIG.get_or_init(Config::from_env).clone();
        Self {
            client: Client::new(),
            app_config,
        }
    }
}

impl ZwipeClient {
    /// Creates a new ZwipeClient with default configuration.
    pub fn new() -> Self {
        Self::default()
    }
}
