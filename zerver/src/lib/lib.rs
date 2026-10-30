//! ZERVER: Backend server library for the ZWIPE MTG deck building application.
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

pub mod config;
pub mod domain;
pub mod inbound;
pub mod outbound;
