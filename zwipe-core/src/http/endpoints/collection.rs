//! Collection calls.

use std::borrow::Cow;

use uuid::Uuid;

use crate::domain::card::Card;
use crate::domain::collection::{CollectionPageSize, CollectionSearch};
use crate::http::contracts::collection::{
    HttpCollectionExport, HttpCollectionImportSummary, HttpCollectionPage, HttpCollectionSummary,
    HttpImportCollection, HttpSetCollectionCard, HttpSetCollectionCardReply,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    COLLECTION_CARDS_ROUTE, COLLECTION_ROUTE, COLLECTION_SUMMARY_ROUTE, EXPORT_COLLECTION_ROUTE,
    IMPORT_COLLECTION_ROUTE, collection_page_query, collection_printings_route,
};

/// Import a collection export. Answers 200 with what landed; 413 past the body cap, 422 for a
/// file it can't read, 429 past the rate limit.
#[derive(Debug)]
pub struct ImportCollection(pub HttpImportCollection);
impl Endpoint for ImportCollection {
    const METHOD: Method = Method::Post;
    type Request = HttpImportCollection;
    type Response = HttpCollectionImportSummary;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(IMPORT_COLLECTION_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// How many cards and printings the caller's collection holds. Only a server with the owned
/// filter answers it, so a client offers "In my collection" only after a reply.
#[derive(Debug)]
pub struct GetCollectionSummary;
impl Endpoint for GetCollectionSummary {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCollectionSummary;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COLLECTION_SUMMARY_ROUTE)
    }
}

/// The caller's owned printings of one card, by oracle id, oldest release first.
#[derive(Debug)]
pub struct GetOwnedPrintings(pub Uuid);
impl Endpoint for GetOwnedPrintings {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = Vec<Card>;
    fn path(&self) -> Cow<'static, str> {
        Cow::Owned(collection_printings_route(self.0))
    }
}

/// Give one row of the caller's collection its copies, 0 to remove it. Answers 200 with the row
/// as it is now; 422 for a printing that isn't a card the collection takes, or a value this
/// server doesn't know; 429 past the rate limit.
#[derive(Debug)]
pub struct SetCollectionCard(pub HttpSetCollectionCard);
impl Endpoint for SetCollectionCard {
    const METHOD: Method = Method::Put;
    type Request = HttpSetCollectionCard;
    type Response = HttpSetCollectionCardReply;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COLLECTION_CARDS_ROUTE)
    }
    fn body(&self) -> Option<&Self::Request> {
        Some(&self.0)
    }
}

/// One page of the caller's collection, sorted by card name. Answers 422 for a search past
/// [`CollectionSearch::MAX_CHARS`] or a page size outside 1 to [`CollectionPageSize::MAX`].
#[derive(Debug)]
pub struct GetCollection {
    /// Card names to match; empty lists every row.
    pub name: CollectionSearch,
    /// Rows before this page.
    pub offset: u32,
    /// Rows on this page.
    pub limit: CollectionPageSize,
}
impl Endpoint for GetCollection {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCollectionPage;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(COLLECTION_ROUTE)
    }
    fn query(&self) -> Option<String> {
        Some(collection_page_query(&self.name, self.offset, self.limit))
    }
}

/// The caller's collection as a ManaBox export, which a collection import reads back. Answers
/// 429 past the rate limit, which it shares with the import.
#[derive(Debug)]
pub struct ExportCollection;
impl Endpoint for ExportCollection {
    const METHOD: Method = Method::Get;
    type Request = ();
    type Response = HttpCollectionExport;
    fn path(&self) -> Cow<'static, str> {
        Cow::Borrowed(EXPORT_COLLECTION_ROUTE)
    }
}
