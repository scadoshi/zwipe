//! What the signed-in user's collection holds, fetched once per user and again when a screen
//! finds it may have changed, kept above the router, so a swipe screen knows whether From offers
//! Collection.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::HttpCollectionSummary;
use zwipe_ui::screens::card_filter;

/// The collection summary as last fetched, with the user it belongs to: another user's copy is
/// never read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CachedCollection {
    /// The user it belongs to.
    pub user_id: Uuid,
    /// What the server answered.
    pub summary: HttpCollectionSummary,
}

/// The root's copy of [`CachedCollection`], empty until a screen asks for it. Provided once,
/// above the router.
#[derive(Debug, Clone, Copy)]
pub struct CollectionCache(pub Signal<Option<CachedCollection>>);

impl CollectionCache {
    /// Asks the server for `signed_in`'s summary again, as a background probe that only logs a
    /// failure, and keeps the answer. The copy held meanwhile still answers, so a refresh never
    /// hides Collection while it runs.
    pub fn refresh(self, client: Signal<ZwipeClient>, signed_in: Session) {
        let Self(mut cache) = self;
        let user_id = signed_in.user.id;
        spawn(async move {
            match client().fetch_collection_summary(&signed_in).await {
                Ok(summary) => cache.set(Some(CachedCollection { user_id, summary })),
                Err(e) => tracing::warn!("collection summary not fetched: {e}"),
            }
        });
    }
}

/// Whether From offers Collection on this screen. Fetches the summary once per signed-in user,
/// unless a screen refreshed it ([`CollectionCache::refresh`]): until a reply arrives, and on a
/// failure or an older server, the answer is no.
pub fn use_offers_owned() -> Memo<bool> {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let collection: CollectionCache = use_context();
    let CollectionCache(cache) = collection;
    use_hook(move || {
        let Some(signed_in) = session.peek().clone() else {
            return;
        };
        if cache
            .peek()
            .is_some_and(|cached| cached.user_id == signed_in.user.id)
        {
            return;
        }
        collection.refresh(client, signed_in);
    });
    use_memo(move || {
        let user_id = session.read().as_ref().map(|s| s.user.id);
        let cached = *cache.read();
        let summary = cached
            .filter(|cached| Some(cached.user_id) == user_id)
            .map(|cached| cached.summary);
        card_filter::offers_owned(summary.as_ref())
    })
}
