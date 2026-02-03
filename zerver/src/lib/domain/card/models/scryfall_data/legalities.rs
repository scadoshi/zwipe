use serde::{Deserialize, Serialize};

/// Card legality status across all Magic: The Gathering formats.
///
/// Each field represents a different format's legality status for a card.
/// `None` means legality information is not available for that format.
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Legalities {
    /// Standard format (most recent sets, ~2 years rotation).
    pub standard: Option<LegalityKind>,
    /// Future format (cards not yet released but previewed).
    pub future: Option<LegalityKind>,
    /// Historic format (MTG Arena eternal format).
    pub historic: Option<LegalityKind>,
    /// Timeless format (MTG Arena eternal format with power-level focus).
    pub timeless: Option<LegalityKind>,
    /// Gladiator format (MTG Arena 100-card singleton).
    pub gladiator: Option<LegalityKind>,
    /// Pioneer format (Return to Ravnica forward).
    pub pioneer: Option<LegalityKind>,
    /// Modern format (8th Edition/Mirrodin forward).
    pub modern: Option<LegalityKind>,
    /// Legacy format (all sets, smaller ban list than Vintage).
    pub legacy: Option<LegalityKind>,
    /// Pauper format (commons only).
    pub pauper: Option<LegalityKind>,
    /// Vintage format (all sets, restricted list instead of bans).
    pub vintage: Option<LegalityKind>,
    /// Penny Dreadful format (cards worth ≤0.02 tix on MTGO).
    pub penny: Option<LegalityKind>,
    /// Commander/EDH format (100-card singleton, 1 legendary commander).
    pub commander: Option<LegalityKind>,
    /// Oathbreaker format (60-card, planeswalker + signature spell).
    pub oathbreaker: Option<LegalityKind>,
    /// Standard Brawl format (Standard card pool, 60-card singleton).
    pub standardbrawl: Option<LegalityKind>,
    /// Brawl format (60-card singleton, commander variant).
    pub brawl: Option<LegalityKind>,
    /// Alchemy format (MTG Arena digital-only cards allowed).
    pub alchemy: Option<LegalityKind>,
    /// Pauper Commander format (uncommon commander, common 99).
    pub paupercommander: Option<LegalityKind>,
    /// Duel Commander format (1v1 Commander variant with separate ban list).
    pub duel: Option<LegalityKind>,
    /// Old School format (93/94, Alpha through Fallen Empires).
    pub oldschool: Option<LegalityKind>,
    /// Premodern format (4th Edition through Scourge).
    pub premodern: Option<LegalityKind>,
    /// PreDH format (Commander with pre-Modern card pool).
    pub predh: Option<LegalityKind>,
    /// Explorer format (MTG Arena Pioneer equivalent).
    pub explorer: Option<LegalityKind>,
    /// Historic Brawl format (MTG Arena Brawl with Historic pool).
    pub historicbrawl: Option<LegalityKind>,
}

/// Card legality status within a format.
///
/// # Variants
/// - **Legal**: Card is legal to play in the format
/// - **NotLegal**: Card is not legal (not in format's card pool)
/// - **Restricted**: Card is restricted to 1 copy (Vintage only)
/// - **Banned**: Card is explicitly banned in the format
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LegalityKind {
    /// Card is legal to play in this format.
    #[default]
    Legal,
    /// Card is not legal (not in format's card pool).
    NotLegal,
    /// Card is restricted to 1 copy per deck (Vintage-specific).
    Restricted,
    /// Card is explicitly banned in this format.
    Banned,
}
