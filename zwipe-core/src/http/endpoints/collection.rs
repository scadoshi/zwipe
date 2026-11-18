//! Collection calls.

use std::borrow::Cow;

use uuid::Uuid;

use crate::domain::card::Card;
use crate::http::contracts::collection::{
    HttpCollectionImportSummary, HttpCollectionSummary, HttpImportCollection,
};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::{
    COLLECTION_SUMMARY_ROUTE, IMPORT_COLLECTION_ROUTE, collection_printings_route,
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
