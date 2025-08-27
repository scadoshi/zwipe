use serde::{Deserialize, Serialize};
use strum::{Display, EnumString};

#[derive(Default, Debug, Clone, Serialize, Deserialize, PartialEq, Display, EnumString)]
#[strum(serialize_all = "snake_case")]
pub enum MtgFormat {
    #[default]
    Standard,
    Pioneer,
    Modern,
    Legacy,
    Pauper,
    Vintage,
    Commander,
}

impl From<String> for MtgFormat {
    fn from(value: String) -> Self {
        value.parse().unwrap_or_default()
    }
}
