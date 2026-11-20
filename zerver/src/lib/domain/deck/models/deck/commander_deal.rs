//! The commander deal: one page of the merged ranking over every commander a filter matches, the
//! user's decks and saved commanders leading and popularity ordering what they can't separate.

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::models::format::Format;
use zwipe_core::http::contracts::suggestions::{
    HttpCommanderDeal, MAX_DEAL_EXCLUDE, MAX_SUGGESTION_LIMIT, RankedBy,
};

use super::commander_suggestions::CommanderSuggestion;

/// A deal request as the service reads it: the limit clamped to [`MAX_SUGGESTION_LIMIT`],
/// `exclude` cut to its first [`MAX_DEAL_EXCLUDE`], and the format settled.
#[derive(Debug, Clone, PartialEq)]
pub struct CommanderDeal {
    /// Narrows the candidates; every commander when `None`.
    pub filter: Option<CardQuery>,
    /// The format whose commander rules a candidate meets.
    pub format: Format,
    /// Ranked commanders to skip.
    pub offset: u32,
    /// The page's size.
    pub limit: u32,
    /// Oracle ids already dealt, left out.
    pub exclude: Vec<Uuid>,
    /// Keeps the user's commander maybeboard among the candidates.
    pub keep_maybeboard: bool,
}

impl From<HttpCommanderDeal> for CommanderDeal {
    fn from(deal: HttpCommanderDeal) -> Self {
        let mut exclude = deal.exclude;
        exclude.truncate(MAX_DEAL_EXCLUDE);
        Self {
            filter: deal.filter,
            format: deal.format.unwrap_or(Format::Commander),
            offset: deal.offset,
            limit: deal.limit.min(MAX_SUGGESTION_LIMIT),
            exclude,
            keep_maybeboard: deal.keep_maybeboard,
        }
    }
}

impl CommanderDeal {
    /// Whether the ranking spreads across color identities, holding back each identity past its
    /// first few: only when the filter names no color identity, since one named identity has
    /// nothing to spread across.
    #[must_use]
    pub fn caps_identities(&self) -> bool {
        self.filter.as_ref().is_none_or(|filter| {
            let criteria = filter.criteria();
            criteria.color_identity_equals().is_none() && criteria.color_identity_within().is_none()
        })
    }
}

/// What the deal statement scores a page against, beside the request.
#[derive(Debug, Clone, PartialEq)]
pub struct DealSignal {
    /// The user, whose live dismissals are left out.
    pub user_id: Uuid,
    /// The user's decks as the suggestions statement reads them: a JSON array of
    /// `{"colors": [WUBRG letters], "profile": {otag: weight} | null}`.
    pub decks: serde_json::Value,
    /// The user's commander maybeboard: signal, whether or not it is left out.
    pub maybeboard: Vec<Uuid>,
    /// Every oracle id to leave out: the commanders the user leads, the maybeboard unless kept,
    /// and the request's own.
    pub exclude: Vec<Uuid>,
}

/// One page of the deal, best fit first, and what the ranking is built from.
#[derive(Debug, Clone)]
pub struct CommanderDealPage {
    /// The page's commanders, each with the reasons behind it.
    pub suggestions: Vec<CommanderSuggestion>,
    /// What the ranking is built from.
    pub ranked_by: RankedBy,
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::card::scryfall_data::colors::{Color, Colors};
    use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;

    use super::*;

    fn deal(filter: Option<CardQuery>) -> HttpCommanderDeal {
        HttpCommanderDeal {
            filter,
            format: None,
            offset: 0,
            limit: 10,
            exclude: vec![],
            keep_maybeboard: false,
        }
    }

    #[test]
    fn a_deal_clamps_its_limit_cuts_its_exclusions_and_reads_as_commander() {
        let mut http = deal(None);
        http.limit = 5_000;
        http.exclude = (0..900_u128).map(Uuid::from_u128).collect();
        let deal = CommanderDeal::from(http);
        assert_eq!(deal.limit, MAX_SUGGESTION_LIMIT);
        assert_eq!(deal.exclude.len(), MAX_DEAL_EXCLUDE);
        assert_eq!(
            deal.exclude.first(),
            Some(&Uuid::from_u128(0)),
            "the first kept"
        );
        assert_eq!(deal.format, Format::Commander);
    }

    #[test]
    fn identities_spread_only_while_the_filter_names_none() {
        assert!(CommanderDeal::from(deal(None)).caps_identities());
        let mut by_name = CardQueryBuilder::default();
        by_name.set_name_contains("atraxa");
        let by_name = by_name.build().unwrap();
        assert!(CommanderDeal::from(deal(Some(by_name))).caps_identities());
        let mut jeskai = CardQueryBuilder::default();
        jeskai.set_color_identity_equals(Colors::from([Color::White, Color::Blue, Color::Red]));
        let jeskai = jeskai.build().unwrap();
        assert!(!CommanderDeal::from(deal(Some(jeskai))).caps_identities());
    }
}
