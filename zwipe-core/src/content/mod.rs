//! Shared static content served across all three edges (zerver, zwiper, zite).
//!
//! Unlike [`crate::domain`], these are not entities with invariants or
//! behavior; they are canonical content the whole app renders. They live in
//! core because it is the only crate every edge shares, and they depend on
//! nothing external.

/// Release history, served at `/api/changelog` and rendered by the app + site.
pub mod changelog;
