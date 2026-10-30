//! User metrics HTTP handlers.

pub mod check_completion;
pub mod get_my_metrics;
pub mod get_public_metrics;
pub mod record_anonymous_event;
pub mod record_crash;
pub mod record_usage;

pub use get_my_metrics::get_my_metrics;
pub use get_public_metrics::get_public_metrics;
pub use record_anonymous_event::record_anonymous_event;
pub use record_crash::record_crash;
pub use record_usage::record_usage;
