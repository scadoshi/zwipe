//! Nightly upkeep (maintenance) domain, zervice-only.
//!
//! Named for the Magic phase: this IS the "at the beginning of your upkeep"
//! step. Home for aggregate/diagnostic cleanups (error/crash retention, the
//! global expired-session dusting). Per-key invariants (e.g. the per-user
//! session cap) belong on their write paths instead, where the write that could
//! break one enforces it; upkeep sweeps only what no single write owns.

pub mod ports;
pub mod services;
