/// An object containing daily price information for this card,
/// including usd, usd_foil, usd_etched, eur, eur_foil,
/// eur_etched, and tix prices, as strings.
use serde::{Deserialize, Serialize};

#[derive(Clone, Serialize, Deserialize)]
pub struct Prices {
    pub usd: String,
    pub usd_foil: String,
    pub usd_etched: String,
    pub eur: String,
    pub eur_foil: String,
    pub eur_etched: String,
    pub tix: String,
}
