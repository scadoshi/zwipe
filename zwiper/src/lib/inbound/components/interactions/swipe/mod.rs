//! Swipe gesture detection and handling.
//!
//! Provides a complete swipe gesture system for card swiping interactions,
//! supporting both touch and mouse events with configurable thresholds.
//!
//! The primary consumer is [`stack::SwipeStack`] — a peeking card stack where
//! only the top card is interactive, commits exit fully off-screen, and undo
//! brings the last-committed card back in from the direction it exited.

/// Axis locking for swipe gestures (X or Y).
pub mod axis;
/// Swipe gesture configuration (thresholds, allowed directions).
pub mod config;
/// Swipe direction enum (Left, Right, Up, Down).
pub mod direction;
/// Mouse event handlers for desktop swipe emulation.
pub mod onmouse;
/// Core swipe event processing shared by touch and mouse handlers.
pub mod onswipe;
/// Touch event handlers for mobile swipe gestures.
pub mod ontouch;
/// Screen offset utilities for position calculations.
pub mod screen_offset;
/// Peeking card stack component.
pub mod stack;
/// Swipe state management and gesture tracking.
pub mod state;
/// Timestamped point for velocity calculations.
pub mod time_point;

pub use stack::{STACK_DEPTH, SwipeStack};
