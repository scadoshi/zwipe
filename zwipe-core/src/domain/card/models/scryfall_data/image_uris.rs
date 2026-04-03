use serde::{Deserialize, Serialize};

/// Card image URLs at various resolutions and crop styles.
///
/// Scryfall provides images in multiple formats optimized for different use cases:
/// - **Size variants**: small, normal, large, png (increasing quality/resolution)
/// - **Crop variants**: border_crop (card edges cropped), art_crop (only artwork)
///
/// # Image Sizes (approximate)
/// - **small**: ~146×204px - thumbnails, list views
/// - **normal**: ~488×680px - standard card display
/// - **large**: ~672×936px - high-quality display
/// - **png**: ~745×1040px - highest quality, lossless
///
/// All fields are `Option<String>` as not all cards have all image variants available.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct ImageUris {
    /// Thumbnail size (~146×204px). `None` if unavailable.
    pub small: Option<String>,
    /// Standard size (~488×680px). `None` if unavailable.
    pub normal: Option<String>,
    /// Large size (~672×936px). `None` if unavailable.
    pub large: Option<String>,
    /// Highest quality PNG (~745×1040px). `None` if unavailable.
    pub png: Option<String>,
    /// Cropped to card edges (removes outer border). `None` if unavailable.
    pub border_crop: Option<String>,
    /// Cropped to artwork only (no frame/text). `None` if unavailable.
    pub art_crop: Option<String>,
}
