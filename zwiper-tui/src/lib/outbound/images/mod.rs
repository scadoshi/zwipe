//! Card images from Scryfall's image CDN, the only host images are fetched from.

use anyhow::{Context, Result, bail};
use image::DynamicImage;
use reqwest::Url;

/// Scryfall's image CDN.
const IMAGE_HOST: &str = "cards.scryfall.io";

/// The largest image accepted, in bytes. A normal-size card is about 100 KB.
const MAX_BYTES: usize = 5 * 1024 * 1024;

/// Whether `url` is an https link to Scryfall's image CDN.
pub(crate) fn allowed(url: &str) -> bool {
    Url::parse(url).is_ok_and(|u| u.scheme() == "https" && u.host_str() == Some(IMAGE_HOST))
}

/// Downloads and decodes the image at `url`, off the async threads.
///
/// # Errors
///
/// Fails when `url` is not on Scryfall's image CDN, the download fails or is larger than 5 MB, or the bytes are not an image.
pub(crate) async fn fetch(client: &reqwest::Client, url: &str) -> Result<DynamicImage> {
    if !allowed(url) {
        bail!("not a Scryfall image: {url}");
    }
    let response = client.get(url).send().await?.error_for_status()?;
    let too_big = |len: usize| len > MAX_BYTES;
    if response
        .content_length()
        .is_some_and(|n| usize::try_from(n).map_or(true, too_big))
    {
        bail!("image too large");
    }
    let bytes = response.bytes().await?;
    if too_big(bytes.len()) {
        bail!("image too large");
    }
    tokio::task::spawn_blocking(move || image::load_from_memory(&bytes))
        .await
        .context("image decode stopped")?
        .context("not an image")
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn only_https_scryfall_images_are_fetched() {
        assert!(allowed(
            "https://cards.scryfall.io/normal/front/4/a/x.jpg?1783939188"
        ));
        assert!(!allowed("http://cards.scryfall.io/normal/front/4/a/x.jpg"));
        assert!(!allowed("https://cards.scryfall.io.evil.example/x.jpg"));
        assert!(!allowed("https://example.com/x.jpg"));
        assert!(!allowed("file:///etc/passwd"));
        assert!(!allowed("not a url"));
    }
}
