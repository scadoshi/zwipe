//! Navigation plumbing shared across routes.

/// OS back-intent bridge (edge swipe / hardware back → router `go_back`).
pub mod back_handler;
/// Back-aware overlay stack (back gesture closes the top overlay before the route).
pub mod overlay_stack;
