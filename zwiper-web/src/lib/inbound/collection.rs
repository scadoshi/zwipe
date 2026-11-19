//! What the signed-in user's collection holds, fetched once per user and again after every change
//! to it, and kept at the root, so a swipe screen knows whether From offers Collection.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::HttpCollectionSummary;
use zwipe_ui::screens::card_filter;

use crate::inbound::chrome::authed::{Authed, use_authed};

/// The collection summary as last fetched, with the user it belongs to: another user's copy is
/// never read.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct CachedCollection {
    /// The user it belongs to.
    pub user_id: Uuid,
    /// What the server answered.
    pub summary: HttpCollectionSummary,
}

/// The root's copy of [`CachedCollection`], empty until a screen asks for it.
#[derive(Debug, Clone, Copy)]
pub struct CollectionCache(pub Signal<Option<CachedCollection>>);

impl CollectionCache {
    /// Asks the server for the signed-in user's summary again, quietly, and keeps the answer. The
    /// copy held meanwhile still answers, so a refresh never hides Collection while it runs; a
    /// failure keeps it as it was.
    pub fn refresh(self, authed: Authed) {
        let Self(mut cache) = self;
        spawn(async move {
            let fetched = authed
                .run_quiet("collection_summary", |c, s| async move {
                    let user_id = s.user.id;
                    c.fetch_collection_summary(&s)
                        .await
                        .map(|summary| (user_id, summary))
                })
                .await;
            if let Some((user_id, summary)) = fetched {
                cache.set(Some(CachedCollection { user_id, summary }));
            }
        });
    }
}

/// Whether From offers Collection on this screen. Fetches the summary once per signed-in user,
/// quietly, unless a collection change refreshed it ([`CollectionCache::refresh`]): until a reply
/// arrives, and on a failure or an older server, the answer is no.
pub fn use_offers_owned() -> Memo<bool> {
    let session: Signal<Option<Session>> = use_context();
    let collection: CollectionCache = use_context();
    let CollectionCache(cache) = collection;
    let authed = use_authed();
    use_hook(move || {
        let Some(user_id) = session.peek().as_ref().map(|s| s.user.id) else {
            return;
        };
        if cache.peek().is_some_and(|cached| cached.user_id == user_id) {
            return;
        }
        collection.refresh(authed);
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
