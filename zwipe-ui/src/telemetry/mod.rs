//! Usage telemetry every client sends: the buffer that counts swipes, searches, suggestion signals, pile ends and handled errors between flushes, and the closed vocabulary error reports name screens with.
//!
//! The buffer is a plain struct: recording takes `&mut self` and [`UsageBuffer::snapshot_and_zero`] drains it into one [`HttpUsageBatch`](zwipe_core::http::contracts::metrics::HttpUsageBatch). Each client owns its lock, its flush timer and its send, so nothing here spawns, waits or names a runtime.

mod handled_error;
mod usage_buffer;
pub mod vocabulary;

pub use handled_error::{ErrorKind, HandledError};
pub use usage_buffer::{SwipeDirection, UsageBuffer};
