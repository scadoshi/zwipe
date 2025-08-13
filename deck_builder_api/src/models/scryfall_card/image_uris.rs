use serde::{Deserialize, Serialize};

/// To be stored against various card objects
/// against the "image_uris" field usually
#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct ImageUris {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
    pub png: Option<String>,
    pub border_crop: Option<String>,
    pub art_crop: Option<String>,
}
