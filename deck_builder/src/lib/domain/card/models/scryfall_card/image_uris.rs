// external
use serde::{Deserialize, Serialize};

/// stores image uri data against ScryfallCard and CardFace
/// against image_uris field
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ImageUris {
    pub small: Option<String>,
    pub normal: Option<String>,
    pub large: Option<String>,
    pub png: Option<String>,
    pub border_crop: Option<String>,
    pub art_crop: Option<String>,
}
