//! Collection calls.

use std::borrow::Cow;

use crate::http::contracts::collection::{HttpCollectionImportSummary, HttpImportCollection};
use crate::http::endpoint::{Endpoint, Method};
use crate::http::paths::IMPORT_COLLECTION_ROUTE;

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
