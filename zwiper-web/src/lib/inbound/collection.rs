//! What the signed-in user's collection holds, fetched once per user and kept at the root, so
//! a swipe screen knows whether From offers Collection.

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::collection::HttpCollectionSummary;
use zwipe_ui::screens::card_filter;

use crate::inbound::chrome::authed::use_authed;

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

/// Whether From offers Collection on this screen. Fetches the summary once per
/// signed-in user, quietly: until a reply arrives, and on a failure or an older server, the
/// answer is no.
pub fn use_offers_owned() -> Memo<bool> {
    let session: Signal<Option<Session>> = use_context();
    let CollectionCache(mut cache) = use_context();
    let authed = use_authed();
    use_hook(move || {
        let Some(user_id) = session.peek().as_ref().map(|s| s.user.id) else {
            return;
        };
        if cache.peek().is_some_and(|cached| cached.user_id == user_id) {
            return;
        }
        spawn(async move {
            let summary = authed
                .run_quiet("collection_summary", |c, s| async move {
                    c.fetch_collection_summary(&s).await
                })
                .await;
            if let Some(summary) = summary {
                cache.set(Some(CachedCollection { user_id, summary }));
            }
        });
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
