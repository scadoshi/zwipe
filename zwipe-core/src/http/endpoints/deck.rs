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
        contracts::{
            deck::{
                HttpClearedSuppressions, HttpCloneDeck, HttpClonedDeck, HttpCreateDeckProfile,
                HttpDeckShareToken, HttpImportArchidektDeck, HttpSharedDeck, HttpSkipDeckCard,
                HttpUpdateDeckProfile,
            },
            deck_card::{HttpCreateDeckCard, HttpImportDeckCards, HttpPatchDeckCard},
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
use std::borrow::Cow;
use uuid::Uuid;

/// Every deck profile the user owns.
pub struct GetDeckProfiles;
impl Endpoint for GetDeckProfiles {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Vec<DeckProfile>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_DECK_PROFILES_ROUTE)
    }
}

/// Create a deck. Answers 201.
pub struct CreateDeck(pub HttpCreateDeckProfile);
impl Endpoint for CreateDeck {
    const METHOD: Method = Method::Post;
    type Request = HttpCreateDeckProfile;
    type Response = DeckProfile;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(CREATE_DECK_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// One deck with its entries and warnings.
pub struct GetDeck(pub Uuid);
impl Endpoint for GetDeck {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Deck;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_deck_route(self.0))
    }
}

/// One deck's profile only.
pub struct GetDeckProfile(pub Uuid);
impl Endpoint for GetDeckProfile {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = DeckProfile;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_deck_profile_route(self.0))
    }
}

/// Edit a deck's profile.
pub struct UpdateDeckProfile(pub Uuid, pub HttpUpdateDeckProfile);
impl Endpoint for UpdateDeckProfile {
    const METHOD: Method = Method::Patch;
    type Request = HttpUpdateDeckProfile;
    type Response = DeckProfile;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(update_deck_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
    }
}

/// Delete a deck. Answers 204.
pub struct DeleteDeck(pub Uuid);
impl Endpoint for DeleteDeck {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(delete_deck_route(self.0))
    }
}

/// Copy a deck into a new one. Answers 201.
pub struct CloneDeck(pub Uuid, pub HttpCloneDeck);
impl Endpoint for CloneDeck {
    const METHOD: Method = Method::Post;
    type Request = HttpCloneDeck;
    type Response = HttpClonedDeck;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(clone_deck_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
    }
}

/// Token-producing tokens the deck's cards create.
pub struct GetDeckTokens(pub Uuid);
impl Endpoint for GetDeckTokens {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_deck_tokens_route(self.0))
    }
}

/// The user's deck tag vocabulary.
pub struct GetDeckTags;
impl Endpoint for GetDeckTags {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Vec<DeckTagView>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(GET_DECK_TAGS_ROUTE)
    }
}

/// Start sharing a deck publicly.
pub struct ShareDeck(pub Uuid);
impl Endpoint for ShareDeck {
    const METHOD: Method = Method::Post;
    type Request = ();
    type Response = HttpDeckShareToken;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(share_deck_route(self.0))
    }
}

/// Stop sharing a deck. Answers 204.
pub struct UnshareDeck(pub Uuid);
impl Endpoint for UnshareDeck {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(share_deck_route(self.0))
    }
}

/// Suppress one card from a deck's suggestions. Answers 204.
pub struct SkipDeckCard(pub Uuid, pub HttpSkipDeckCard);
impl Endpoint for SkipDeckCard {
    const METHOD: Method = Method::Post;
    type Request = HttpSkipDeckCard;
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(skip_deck_card_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
    }
}

/// Lift one suppression. Answers 204.
pub struct UnskipDeckCard(pub Uuid, pub Uuid);
impl Endpoint for UnskipDeckCard {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(unskip_deck_card_route(self.0, self.1))
    }
}

/// Lift every suppression on a deck.
pub struct ClearDeckSuppressions(pub Uuid);
impl Endpoint for ClearDeckSuppressions {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = HttpClearedSuppressions;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(clear_deck_suppressions_route(self.0))
    }
}

/// Add a card to a deck. Answers 201.
pub struct CreateDeckCard(pub Uuid, pub HttpCreateDeckCard);
impl Endpoint for CreateDeckCard {
    const METHOD: Method = Method::Post;
    type Request = HttpCreateDeckCard;
    type Response = DeckCard;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(create_deck_card_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
    }
}

/// Edit one deck card.
pub struct UpdateDeckCard(pub Uuid, pub Uuid, pub HttpPatchDeckCard);
impl Endpoint for UpdateDeckCard {
    const METHOD: Method = Method::Patch;
    type Request = HttpPatchDeckCard;
    type Response = DeckCard;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(update_deck_card_route(self.0, self.1))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.2)
    }
}

/// Remove one deck card.
pub struct DeleteDeckCard(pub Uuid, pub Uuid);
impl Endpoint for DeleteDeckCard {
    const METHOD: Method = Method::Delete;
    type Request = ();
    type Response = ();
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(delete_deck_card_route(self.0, self.1))
    }
}

/// Import a decklist from pasted text.
pub struct ImportDeckCards(pub Uuid, pub HttpImportDeckCards);
impl Endpoint for ImportDeckCards {
    const METHOD: Method = Method::Post;
    type Request = HttpImportDeckCards;
    type Response = ImportDeckCardsResult;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(import_deck_cards_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
    }
}

/// Import a public Archidekt deck by URL.
pub struct ImportArchidektDeck(pub Uuid, pub HttpImportArchidektDeck);
impl Endpoint for ImportArchidektDeck {
    const METHOD: Method = Method::Post;
    type Request = HttpImportArchidektDeck;
    type Response = ImportDeckCardsResult;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(import_archidekt_deck_route(self.0))
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.1)
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
    type Request = ();
    type Response = HttpSharedDeck;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(get_shared_deck_route(self.0))
    }
}
