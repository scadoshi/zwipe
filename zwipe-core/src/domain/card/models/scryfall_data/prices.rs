use serde::{Deserialize, Serialize};

/// Current market prices for a card in various currencies and finishes.
///
/// Prices are sourced from TCGplayer (USD), Cardmarket (EUR), and MTGO (TIX).
/// All prices are stored as strings to preserve exact decimal precision.
/// `None` means the card is not available in that finish or market.
///
/// # Finishes
/// - **Regular (nonfoil)**: Standard printing
/// - **Foil**: Shiny foil finish (premium)
/// - **Etched**: Matte etched foil (special premium finish)
///
/// # Currencies
/// - **USD**: US Dollars (from TCGplayer)
/// - **EUR**: Euros (from Cardmarket)
/// - **TIX**: MTGO Event Tickets (Magic Online currency)
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq)]
pub struct Prices {
    /// US Dollar price for regular/nonfoil version. `None` if not available.
    pub usd: Option<String>,
    /// US Dollar price for foil version. `None` if not available.
    pub usd_foil: Option<String>,
    /// US Dollar price for etched foil version. `None` if not available.
    pub usd_etched: Option<String>,
    /// Euro price for regular/nonfoil version. `None` if not available.
    pub eur: Option<String>,
    /// Euro price for foil version. `None` if not available.
    pub eur_foil: Option<String>,
    /// Euro price for etched foil version. `None` if not available.
    pub eur_etched: Option<String>,
    /// MTGO Event Ticket price. `None` if not available on MTGO.
    pub tix: Option<String>,
}
