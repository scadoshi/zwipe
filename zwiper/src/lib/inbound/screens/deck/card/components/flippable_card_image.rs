//! Card image with a built-in flip control for double-faced cards.
//!
//! Promoted to `zwipe-components` so zwiper and zite share one implementation;
//! re-exported here so the existing `flippable_card_image::…` paths keep
//! resolving without touching every call site.

pub(crate) use zwipe_components::{FlippableCardImage, reset_image_ease};
