//! External service integrations (API client, session management).
//!
//! Handles communication with external services from the frontend:
//! - **Client**: HTTP API client for backend communication
//! - **Session**: User session and authentication state management

/// HTTP API client for backend communication.
pub mod client;
/// User session and authentication state management.
pub mod session;
