//! HTTP contract types shared between frontend and backend.
//!
//! This module contains path constants, helper types, and request/response
//! structs that define the API contract between zerver and zwiper.

/// HTTP request/response contract structs organized by domain.
pub mod contracts;
/// Partial-update helpers (`Opdate`).
pub mod helpers;
/// Path constants shared between frontend and backend for URL consistency.
pub mod paths;
