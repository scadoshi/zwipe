//! Nightly upkeep (maintenance) domain — zervice-only.
//!
//! Named for the Magic phase: this IS the "at the beginning of your upkeep"
//! step. Home for aggregate/diagnostic cleanups (error/crash retention, the
//! global expired-session dusting). Per-key invariants (e.g. the per-user
//! session cap) belong on their write paths instead — see
//! `context/plans/client-error-reporting.md` for the drive-by vs upkeep rule.

pub mod ports;
pub mod services;
