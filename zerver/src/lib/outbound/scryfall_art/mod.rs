//! Card art for the share preview image, fetched from Scryfall's image host.
//!
//! The commander's art crop is drawn into a shared deck's preview image. One
//! commander's art serves every deck that leads with it, so fetched images are
//! kept in a small in-memory cache by URL.

use std::collections::{HashMap, VecDeque};
use std::sync::Mutex;
use std::time::Duration;

use axum::body::Bytes;

use crate::outbound::archidekt::USER_AGENT_PRODUCT;

/// The only host art is fetched from, so a stored URL can't point the server
/// anywhere else.
const ART_HOST: &str = "https://cards.scryfall.io/";

/// How long one art fetch may take before the image is drawn without it.
const FETCH_TIMEOUT: Duration = Duration::from_secs(4);

/// Most art images kept in memory. An art crop is 50 to 150 KB.
const CACHE_ENTRIES: usize = 128;

/// Largest image accepted, so a bad response can't take the server's memory.
const MAX_ART_BYTES: usize = 2 * 1024 * 1024;

/// Where art comes from.
#[derive(Debug)]
enum Source {
    /// Scryfall's image host, over this client.
    Scryfall(reqwest::Client),
    /// The same image for every URL, or none: for tests.
    Fixed(Option<Bytes>),
    /// The image each URL maps to, none for any other: for tests.
    ByUrl(HashMap<String, Bytes>),
}

/// Fetches card art by URL, caching what it fetched.
#[derive(Debug)]
pub struct ArtSource {
    source: Source,
    cache: Mutex<VecDeque<(String, Bytes)>>,
}

impl ArtSource {
    /// Art from Scryfall's image host, under a User-Agent that names the
    /// app and `contact_url` (the deployment's public web base URL), as
    /// Scryfall asks.
    pub fn scryfall(contact_url: &str) -> Self {
        let client = reqwest::Client::builder()
            .timeout(FETCH_TIMEOUT)
            .user_agent(format!("{USER_AGENT_PRODUCT} (+{contact_url})"))
            .build()
            .unwrap_or_default();
        Self::with(Source::Scryfall(client))
    }

    /// `image` for every URL, or no art at all when `None`. For tests.
    #[must_use]
    pub fn fixed(image: Option<Bytes>) -> Self {
        Self::with(Source::Fixed(image))
    }

    /// The image each URL in `images` maps to, and none for any other. For
    /// tests that need a different image per card.
    #[must_use]
    pub fn by_url(images: HashMap<String, Bytes>) -> Self {
        Self::with(Source::ByUrl(images))
    }

    fn with(source: Source) -> Self {
        Self {
            source,
            cache: Mutex::new(VecDeque::new()),
        }
    }

    /// The image at `url`, or `None` when it isn't on Scryfall's image host,
    /// the fetch fails or times out, or the image is too large. A failure is
    /// logged and not cached, so the next request tries again.
    pub async fn fetch(&self, url: &str) -> Option<Bytes> {
        match &self.source {
            Source::Fixed(image) => image.clone(),
            Source::ByUrl(images) => images.get(url).cloned(),
            Source::Scryfall(client) => {
                if !url.starts_with(ART_HOST) {
                    return None;
                }
                if let Some(hit) = self.cached(url) {
                    return Some(hit);
                }
                let image = download(client, url).await?;
                self.remember(url, image.clone());
                Some(image)
            }
        }
    }

    fn cached(&self, url: &str) -> Option<Bytes> {
        let cache = self.cache.lock().ok()?;
        cache.iter().find(|(u, _)| u == url).map(|(_, b)| b.clone())
    }

    fn remember(&self, url: &str, image: Bytes) {
        let Ok(mut cache) = self.cache.lock() else {
            return;
        };
        if cache.len() >= CACHE_ENTRIES {
            cache.pop_front();
        }
        cache.push_back((url.to_owned(), image));
    }
}

/// One GET of `url`, its body when it succeeds and fits.
async fn download(client: &reqwest::Client, url: &str) -> Option<Bytes> {
    let response = match client.get(url).send().await {
        Ok(response) if response.status().is_success() => response,
        Ok(response) => {
            tracing::warn!(status = %response.status(), url, "share preview art fetch refused");
            return None;
        }
        Err(e) => {
            tracing::warn!(error = %e, url, "share preview art fetch failed");
            return None;
        }
    };
    let body = response.bytes().await.ok()?;
    (body.len() <= MAX_ART_BYTES).then_some(body)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn a_fixed_source_answers_every_url() {
        let art = ArtSource::fixed(Some(Bytes::from_static(b"art")));
        assert_eq!(
            art.fetch("https://example.com/x").await.as_deref(),
            Some(&b"art"[..])
        );
        assert_eq!(ArtSource::fixed(None).fetch("any").await, None);
    }

    #[tokio::test]
    async fn art_only_comes_from_scryfalls_image_host() {
        let art = ArtSource::scryfall("test");
        assert_eq!(art.fetch("https://example.com/art.jpg").await, None);
        assert_eq!(art.fetch("http://cards.scryfall.io/art.jpg").await, None);
    }

    #[test]
    fn the_cache_keeps_the_newest_entries() {
        let art = ArtSource::fixed(None);
        for i in 0..=CACHE_ENTRIES {
            art.remember(&format!("u{i}"), Bytes::from_static(b"x"));
        }
        assert!(art.cached("u0").is_none(), "the oldest left");
        assert!(art.cached(&format!("u{CACHE_ENTRIES}")).is_some());
    }
}
