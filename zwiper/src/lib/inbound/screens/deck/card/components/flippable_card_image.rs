//! Card image with a built-in flip control for double-faced cards.
//!
//! Lives in `components` so zwiper and zite share one implementation;
//! re-exported here so the existing `flippable_card_image::…` paths keep
//! resolving without touching every call site.

pub(crate) use components::{FlippableCardImage, reset_image_ease};
