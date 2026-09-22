//! Card catalog, search and metadata endpoints.

use crate::{
    domain::card::{
        Card, card_role::CardRoleView, oracle_tag::OracleTag,
        scryfall_data::universe::UbFranchiseView, search_card::card_filter::CardQuery,
    },
    http::{
        endpoint::{Endpoint, Method},
        paths::{
            FEATURED_FLAVOR_ROUTE, GET_ARTISTS_ROUTE, GET_CARD_ROLES_ROUTE, GET_CARD_TYPES_ROUTE,
            GET_KEYWORD_REMINDERS_ROUTE, GET_KEYWORDS_ROUTE, GET_LANGUAGES_ROUTE,
            GET_ORACLE_TAGS_ROUTE, GET_ORACLE_WORDS_ROUTE, GET_SETS_ROUTE, GET_UB_FRANCHISES_ROUTE,
            SEARCH_CARDS_ROUTE, SEARCH_COMMANDERS_ROUTE, get_card_route, get_printings_route,
        },
    },
};
use std::borrow::Cow;
use uuid::Uuid;

/// Builds an unauthed GET returning `$resp` from a fixed path.
macro_rules! public_get {
    ($(#[$doc:meta])* $name:ident => $path:expr, $resp:ty) => {
        $(#[$doc])*
        pub struct $name;
        impl Endpoint for $name {
            const METHOD: Method = Method::Get;
            const AUTH: bool = false;
            type Request = ();
            type Response = $resp;
            fn path(&self) -> Cow<'static, str> {
                Cow::Borrowed($path)
            }
        }
    };
}

public_get!(
    /// The hour's featured flavor card.
    FeaturedFlavor => FEATURED_FLAVOR_ROUTE, Card
);
public_get!(
    /// Every artist name in the catalog.
    GetArtists => GET_ARTISTS_ROUTE, Vec<String>
);
public_get!(
    /// Every card type in the catalog.
    GetCardTypes => GET_CARD_TYPES_ROUTE, Vec<String>
);
public_get!(
    /// Every keyword in the catalog.
    GetKeywords => GET_KEYWORDS_ROUTE, Vec<String>
);
public_get!(
    /// Reminder text for each keyword, by keyword.
    GetKeywordReminders => GET_KEYWORD_REMINDERS_ROUTE, std::collections::HashMap<String, String>
);
public_get!(
    /// Every language in the catalog.
    GetLanguages => GET_LANGUAGES_ROUTE, Vec<String>
);
public_get!(
    /// Every oracle word in the catalog.
    GetOracleWords => GET_ORACLE_WORDS_ROUTE, Vec<String>
);
public_get!(
    /// Every set code in the catalog.
    GetSets => GET_SETS_ROUTE, Vec<String>
);
public_get!(
    /// The card-role catalog.
    GetCardRoles => GET_CARD_ROLES_ROUTE, Vec<CardRoleView>
);
public_get!(
    /// The oracle-tag catalog.
    GetOracleTags => GET_ORACLE_TAGS_ROUTE, Vec<OracleTag>
);

/// One card by its Scryfall printing id.
pub struct GetCard(pub Uuid);
impl Endpoint for GetCard {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Request = ();
    type Response = Card;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_card_route(self.0))
    }
}

/// Every printing of one oracle id.
pub struct GetPrintings(pub Uuid);
impl Endpoint for GetPrintings {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Request = ();
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_printings_route(self.0))
    }
}

/// Filtered card search. The filter is pre-serialized so this type stays free
/// of the query builder's generics.
pub struct SearchCards(pub CardQuery);
impl Endpoint for SearchCards {
    const METHOD: Method = Method::Post;
    type Request = CardQuery;
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(SEARCH_CARDS_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// Filtered search restricted to commander-eligible cards.
pub struct SearchCommanders(pub CardQuery);
impl Endpoint for SearchCommanders {
    const METHOD: Method = Method::Post;
    type Request = CardQuery;
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(SEARCH_COMMANDERS_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

public_get!(
    /// Franchises the Universes Beyond exceptions picker offers.
    GetUbFranchises => GET_UB_FRANCHISES_ROUTE, Vec<UbFranchiseView>
);
