//! Card image with a built-in flip control for double-faced cards.
//!
//! Lives in `components` so zwiper and zite share one implementation, and is
//! re-exported here for the deck card screens.

pub(crate) use components::{FlippableCardImage, reset_image_ease};
