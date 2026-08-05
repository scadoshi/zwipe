mod featured_flavor;
mod page_meta;
mod stats_strip;
pub use featured_flavor::FeaturedFlavor;
pub use page_meta::PageMeta;
pub use stats_strip::StatsStrip;

/// Browser `setTimeout` as a future. No-op off wasm (the server render never
/// runs the dismiss timers); the `.await` only exists on wasm, hence the allow.
#[allow(clippy::unused_async)]
pub(crate) async fn sleep_ms(ms: u32) {
    #[cfg(target_arch = "wasm32")]
    gloo_timers::future::TimeoutFuture::new(ms).await;
    #[cfg(not(target_arch = "wasm32"))]
    let _ = ms;
}
