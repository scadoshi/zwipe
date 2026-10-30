mod featured_flavor;
mod page_meta;
mod pip_toggle;
mod stats_strip;
pub(crate) use featured_flavor::{FeaturedFlavor, dismiss_flavor_overlay};
pub(crate) use page_meta::PageMeta;
pub(crate) use pip_toggle::PipToggle;
pub(crate) use stats_strip::StatsStrip;

/// The full-art overlay's exit animation, `sd-image-out` in `assets/style.css`.
pub(crate) const OVERLAY_EXIT_MS: u32 = 200;

/// Browser `setTimeout` as a future. No-op off wasm (the server render never
/// runs the dismiss timers).
pub(crate) async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ms;
}
