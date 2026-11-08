//! The generated preview image a shared deck's link unfurls with.
//!
//! zwipe.net's `/deck/:token` page points its `og:image` at
//! `/api/share/deck/{token}/preview.png?v={layout}-{updated_at}`. zerver
//! draws the image here: [`layout`] turns the shared deck into an SVG in its
//! owner's theme ([`palette`]), and [`render`] rasterizes it with resvg.
//! Cloudflare caches the PNG by URL, and the version in the URL changes
//! whenever the deck or the drawing does, so each version is drawn about once
//! per edge. Behind the edge,
//! [`SharePreview`] keeps the last few renders and makes concurrent requests
//! for the same image share one render.

pub mod layout;
pub mod palette;
pub mod render;

use std::collections::VecDeque;
use std::sync::{Arc, Mutex};
use std::time::Instant;

use axum::body::Bytes;
use layout::Preview;
use palette::Palette;
use tokio::sync::OnceCell;
use uuid::Uuid;
use zwipe_core::domain::user::theme::ThemeConfig;
use zwipe_core::http::contracts::deck::HttpSharedDeck;
use zwipe_core::http::paths::shared_deck_preview_version;

use crate::outbound::scryfall_art::ArtSource;

/// Most rendered images kept in memory. One is 0.5 to 1 MB.
const RENDERED_ENTRIES: usize = 32;

/// One image: the deck's share token, its version as the preview URL's `v`
/// names it (the layout and the deck's update), and the theme drawn in.
type Key = (Uuid, String, String);

/// A render in flight or done: `None` inside when it failed.
type Slot = Arc<OnceCell<Option<Bytes>>>;

/// Draws shared decks' preview images, keeping recent ones.
#[derive(Debug)]
pub struct SharePreview {
    art: ArtSource,
    rendered: Mutex<VecDeque<(Key, Slot)>>,
}

impl SharePreview {
    /// A renderer that takes card art from `art`.
    #[must_use]
    pub fn new(art: ArtSource) -> Self {
        Self {
            art,
            rendered: Mutex::new(VecDeque::new()),
        }
    }

    /// `deck`'s preview image as PNG, shared by `token`, or `None` when it
    /// couldn't be drawn. A failure isn't kept, so the next request tries
    /// again.
    pub async fn png(&self, token: Uuid, deck: &HttpSharedDeck) -> Option<Bytes> {
        let theme = deck.theme.clone().unwrap_or_default();
        let version = shared_deck_preview_version(deck.updated_at.map_or(0, |t| t.timestamp()));
        let key: Key = (token, version, theme.css_class());
        let slot = self.slot(&key)?;
        let image = slot.get_or_init(|| self.draw(deck, &theme)).await.clone();
        if image.is_none() {
            self.forget(&key);
        }
        image
    }

    /// The slot for `key`, made (and the oldest dropped) when it's new.
    fn slot(&self, key: &Key) -> Option<Slot> {
        let mut rendered = self.rendered.lock().ok()?;
        if let Some((_, slot)) = rendered.iter().find(|(k, _)| k == key) {
            return Some(Arc::clone(slot));
        }
        if rendered.len() >= RENDERED_ENTRIES {
            rendered.pop_front();
        }
        let slot: Slot = Arc::new(OnceCell::new());
        rendered.push_back((key.clone(), Arc::clone(&slot)));
        Some(slot)
    }

    fn forget(&self, key: &Key) {
        if let Ok(mut rendered) = self.rendered.lock() {
            rendered.retain(|(k, _)| k != key);
        }
    }

    /// Fetches the art, lays the image out and renders it off the async
    /// workers.
    async fn draw(&self, deck: &HttpSharedDeck, theme: &ThemeConfig) -> Option<Bytes> {
        let started = Instant::now();
        let preview = Preview::of(deck);
        let palette = Palette::of(theme);
        let mut drawn = Vec::new();
        let mut images = Vec::new();
        for art in &preview.art {
            if let Some(image) = self.art.fetch(&art.url).await {
                drawn.push(art);
                images.push(image.to_vec());
            }
        }
        let svg = layout::svg(&preview, &palette, &drawn);
        let rendered = tokio::task::spawn_blocking(move || render::png(&svg, images)).await;
        match rendered {
            Ok(Ok(png)) => {
                tracing::info!(
                    ms = started.elapsed().as_millis(),
                    bytes = png.len(),
                    art = drawn.len(),
                    "share preview drawn"
                );
                Some(Bytes::from(png))
            }
            Ok(Err(e)) => {
                tracing::error!(error = %e, "share preview did not render");
                None
            }
            Err(e) => {
                tracing::error!(error = %e, "share preview render task failed");
                None
            }
        }
    }
}
