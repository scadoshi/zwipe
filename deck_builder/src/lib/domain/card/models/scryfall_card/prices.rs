use serde::{Deserialize, Serialize};

/// An object containing daily price information for this card,
/// including usd, usd_foil, usd_etched, eur, eur_foil,
/// eur_etched, and tix prices, as strings.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Prices {
    pub usd: Option<String>,
    pub usd_foil: Option<String>,
    pub usd_etched: Option<String>,
    pub eur: Option<String>,
    pub eur_foil: Option<String>,
    pub eur_etched: Option<String>,
    pub tix: Option<String>,
}
