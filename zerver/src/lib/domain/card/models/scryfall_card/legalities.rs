// external
use serde::{Deserialize, Serialize};

/// stores legality of ScryfallCard
/// across various formats
/// against the legalities field
#[derive(Debug, Clone, Serialize, Deserialize, Default, PartialEq)]
pub struct Legalities {
    pub standard: Option<LegalityKind>,
    pub future: Option<LegalityKind>,
    pub historic: Option<LegalityKind>,
    pub timeless: Option<LegalityKind>,
    pub gladiator: Option<LegalityKind>,
    pub pioneer: Option<LegalityKind>,
    pub modern: Option<LegalityKind>,
    pub legacy: Option<LegalityKind>,
    pub pauper: Option<LegalityKind>,
    pub vintage: Option<LegalityKind>,
    pub penny: Option<LegalityKind>,
    pub commander: Option<LegalityKind>,
    pub oathbreaker: Option<LegalityKind>,
    pub standardbrawl: Option<LegalityKind>,
    pub brawl: Option<LegalityKind>,
    pub alchemy: Option<LegalityKind>,
    pub paupercommander: Option<LegalityKind>,
    pub duel: Option<LegalityKind>,
    pub oldschool: Option<LegalityKind>,
    pub premodern: Option<LegalityKind>,
    pub predh: Option<LegalityKind>,
    pub explorer: Option<LegalityKind>,
    pub historicbrawl: Option<LegalityKind>,
}

/// possible legality kinds
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq, Default)]
#[serde(rename_all = "snake_case")]
pub enum LegalityKind {
    #[default]
    Legal,
    NotLegal,
    Restricted,
    Banned,
}
