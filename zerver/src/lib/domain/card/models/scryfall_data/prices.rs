use serde::{Deserialize, Serialize};

/// stores price data for ScryfallData
/// against prices field
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub eur_etched: Option<String>,
    pub tix: Option<String>,
}
