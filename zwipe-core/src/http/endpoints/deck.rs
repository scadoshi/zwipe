//! Decks, their cards, sharing and suppressions.

use crate::{
    domain::{
        card::Card,
        deck::{
            Deck,
            models::{deck_card::DeckCard, deck_profile::DeckProfile, deck_tag::DeckTagView},
            requests::import_deck_cards::ImportDeckCardsResult,
        },
    },
    http::{
        contracts::deck::{
            HttpClearedSuppressions, HttpClonedDeck, HttpDeckShareToken, HttpSharedDeck,
        },
        endpoint::{Endpoint, Method},
        paths::{
            CREATE_DECK_ROUTE, GET_DECK_PROFILES_ROUTE, GET_DECK_TAGS_ROUTE,
            clear_deck_suppressions_route, clone_deck_route, create_deck_card_route,
            delete_deck_card_route, delete_deck_route, get_deck_profile_route, get_deck_route,
            get_deck_tokens_route, get_shared_deck_route, import_archidekt_deck_route,
            import_deck_cards_route, share_deck_route, skip_deck_card_route,
            unskip_deck_card_route, update_deck_card_route, update_deck_route,
        },
    },
};
use serde_json::Value;
use uuid::Uuid;

/// Every deck profile the user owns.
pub struct GetDeckProfiles;
impl Endpoint for GetDeckProfiles {
    const METHOD: Method = Method::Get;
    type Response = Vec<DeckProfile>;
    fn path(&self) -> String {
        GET_DECK_PROFILES_ROUTE.to_string()
    }
}

/// Create a deck. Answers 201.
pub struct CreateDeck(pub Value);
impl Endpoint for CreateDeck {
    const METHOD: Method = Method::Post;
    type Response = DeckProfile;
    fn path(&self) -> String {
        CREATE_DECK_ROUTE.to_string()
    }
    fn body(&self) -> Option<Value> {
        Some(self.0.clone())
    }
}

/// One deck with its entries and warnings.
pub struct GetDeck(pub Uuid);
impl Endpoint for GetDeck {
    const METHOD: Method = Method::Get;
    type Response = Deck;
    fn path(&self) -> String {
        get_deck_route(self.0)
    }
}

/// One deck's profile only.
pub struct GetDeckProfile(pub Uuid);
impl Endpoint for GetDeckProfile {
    const METHOD: Method = Method::Get;
    type Response = DeckProfile;
    fn path(&self) -> String {
        get_deck_profile_route(self.0)
    }
}

/// Edit a deck's profile.
pub struct UpdateDeckProfile(pub Uuid, pub Value);
impl Endpoint for UpdateDeckProfile {
    const METHOD: Method = Method::Patch;
    type Response = DeckProfile;
    fn path(&self) -> String {
        update_deck_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Delete a deck. Answers 204.
pub struct DeleteDeck(pub Uuid);
impl Endpoint for DeleteDeck {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        delete_deck_route(self.0)
    }
}

/// Copy a deck into a new one. Answers 201.
pub struct CloneDeck(pub Uuid, pub Value);
impl Endpoint for CloneDeck {
    const METHOD: Method = Method::Post;
    type Response = HttpClonedDeck;
    fn path(&self) -> String {
        clone_deck_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Token-producing tokens the deck's cards create.
pub struct GetDeckTokens(pub Uuid);
impl Endpoint for GetDeckTokens {
    const METHOD: Method = Method::Get;
    type Response = Vec<Card>;
    fn path(&self) -> String {
        get_deck_tokens_route(self.0)
    }
}

/// The user's deck tag vocabulary.
pub struct GetDeckTags;
impl Endpoint for GetDeckTags {
    const METHOD: Method = Method::Get;
    type Response = Vec<DeckTagView>;
    fn path(&self) -> String {
        GET_DECK_TAGS_ROUTE.to_string()
    }
}

/// Start sharing a deck publicly.
pub struct ShareDeck(pub Uuid);
impl Endpoint for ShareDeck {
    const METHOD: Method = Method::Post;
    type Response = HttpDeckShareToken;
    fn path(&self) -> String {
        share_deck_route(self.0)
    }
}

/// Stop sharing a deck. Answers 204.
pub struct UnshareDeck(pub Uuid);
impl Endpoint for UnshareDeck {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        share_deck_route(self.0)
    }
}

/// Suppress one card from a deck's suggestions. Answers 204.
pub struct SkipDeckCard(pub Uuid, pub Value);
impl Endpoint for SkipDeckCard {
    const METHOD: Method = Method::Post;
    type Response = ();
    fn path(&self) -> String {
        skip_deck_card_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Lift one suppression. Answers 204.
pub struct UnskipDeckCard(pub Uuid, pub Uuid);
impl Endpoint for UnskipDeckCard {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        unskip_deck_card_route(self.0, self.1)
    }
}

/// Lift every suppression on a deck.
pub struct ClearDeckSuppressions(pub Uuid);
impl Endpoint for ClearDeckSuppressions {
    const METHOD: Method = Method::Delete;
    type Response = HttpClearedSuppressions;
    fn path(&self) -> String {
        clear_deck_suppressions_route(self.0)
    }
}

/// Add a card to a deck. Answers 201.
pub struct CreateDeckCard(pub Uuid, pub Value);
impl Endpoint for CreateDeckCard {
    const METHOD: Method = Method::Post;
    type Response = DeckCard;
    fn path(&self) -> String {
        create_deck_card_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Edit one deck card.
pub struct UpdateDeckCard(pub Uuid, pub Uuid, pub Value);
impl Endpoint for UpdateDeckCard {
    const METHOD: Method = Method::Patch;
    type Response = DeckCard;
    fn path(&self) -> String {
        update_deck_card_route(self.0, self.1)
    }
    fn body(&self) -> Option<Value> {
        Some(self.2.clone())
    }
}

/// Remove one deck card.
pub struct DeleteDeckCard(pub Uuid, pub Uuid);
impl Endpoint for DeleteDeckCard {
    const METHOD: Method = Method::Delete;
    type Response = ();
    fn path(&self) -> String {
        delete_deck_card_route(self.0, self.1)
    }
}

/// Import a decklist from pasted text.
pub struct ImportDeckCards(pub Uuid, pub Value);
impl Endpoint for ImportDeckCards {
    const METHOD: Method = Method::Post;
    type Response = ImportDeckCardsResult;
    fn path(&self) -> String {
        import_deck_cards_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Import a public Archidekt deck by URL.
pub struct ImportArchidektDeck(pub Uuid, pub Value);
impl Endpoint for ImportArchidektDeck {
    const METHOD: Method = Method::Post;
    type Response = ImportDeckCardsResult;
    fn path(&self) -> String {
        import_archidekt_deck_route(self.0)
    }
    fn body(&self) -> Option<Value> {
        Some(self.1.clone())
    }
}

/// Read a deck through its public share token.
///
/// Unauthenticated: the share link is meant to open for anyone, including
/// people with no account. A revoked or unknown token answers 404.
pub struct GetSharedDeck(pub Uuid);
impl Endpoint for GetSharedDeck {
    const METHOD: Method = Method::Get;
    const AUTH: bool = false;
    type Response = HttpSharedDeck;
    fn path(&self) -> String {
        get_shared_deck_route(self.0)
    }
}
