//! ZERVER - Backend server library for the ZWIPE MTG deck building application.
//!
//! This crate provides the complete backend implementation including domain logic,
//! database access, HTTP API, and business rules for Magic: The Gathering deck building.

#![warn(missing_docs)]

#[cfg(feature = "zerver")]
pub mod config;
pub mod domain;
pub mod inbound;
#[cfg(feature = "zerver")]
pub mod outbound;
