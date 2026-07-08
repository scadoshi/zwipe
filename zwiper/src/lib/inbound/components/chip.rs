//! Selectable chip button.
//!
//! Re-exported from the shared [`zwipe_components`] crate so `zwiper` and
//! `zite` render the exact same chip. The component moved out of this file on
//! 2026-07-07; this re-export keeps the existing
//! `crate::inbound::components::chip::Chip` import path working.

pub use zwipe_components::Chip;
