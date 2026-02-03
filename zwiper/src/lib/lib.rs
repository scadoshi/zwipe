//! ZWIPER - Shared library for the ZWIPE MTG deck building application.
//!
//! This crate provides common utilities and domain logic shared between the
//! client (Leptos frontend) and server (Axum backend), including:
//! - Domain models and validation
//! - Error message formatting
//! - Language code translations
//! - Configuration types
//!
//! # Modules
//!
//! - [`domain`]: Shared domain logic (errors, language codes)
//! - [`config`]: Configuration types
//! - [`inbound`]: Request handling utilities
//! - [`outbound`]: External service integrations

#![warn(missing_docs)]

/// Configuration types and utilities.
pub mod config;
/// Shared domain logic (errors, language codes).
pub mod domain;
/// Request handling utilities.
pub mod inbound;
/// External service integrations.
pub mod outbound;
