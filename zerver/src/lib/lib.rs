//! ZERVER - Backend server library for the ZWIPE MTG deck building application.
//!
//! This crate provides the complete backend implementation including domain logic,
//! database access, HTTP API, and business rules for Magic: The Gathering deck building.
//!
//! # Architecture
//!
//! The crate follows a layered architecture:
//!
//! - **[`config`]**: Application configuration from environment variables
//! - **[`domain`]**: Core business logic, models, and validation rules
//! - **[`inbound`]**: HTTP API layer (Axum handlers and routing)
//! - **[`outbound`]**: Database and external service integrations

#![warn(missing_docs)]

/// Application configuration from environment variables.
///
/// Provides [`Config`](config::Config) for loading and validating startup configuration.
#[cfg(feature = "zerver")]
pub mod config;

/// Core domain models and business logic.
///
/// Contains the heart of the application: cards, decks, users, and authentication.
/// All validation and business rules are enforced at this layer.
pub mod domain;

/// HTTP API layer for external communication.
///
/// Axum-based handlers, routing, and request/response types.
pub mod inbound;

/// Database and external service integrations.
///
/// PostgreSQL repositories, Scryfall API client, and data mapping.
#[cfg(feature = "zerver")]
pub mod outbound;
