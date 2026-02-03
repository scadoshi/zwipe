//! Shared domain logic for the ZWIPE application.
//!
//! This crate provides common domain utilities used across both client and server:
//! - **Error handling**: User-facing error message formatting
//! - **Language support**: MTG card language code translations

/// User-facing error message formatting.
pub mod error;
/// Language code to display name conversions.
pub mod language;
