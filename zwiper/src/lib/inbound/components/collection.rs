//! What the signed-in user's collection holds, fetched once per user and kept above the router,
//! so a server-searched filter sheet knows whether to offer "In my collection".

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

/// Whether this screen's filter sheet offers "In my collection". Fetches the summary once per
/// signed-in user, as a background probe that only logs a failure, the way Swipe select's
/// searches do: until a reply arrives, and on a failure or an older server, the answer is no.
pub fn use_offers_owned() -> Memo<bool> {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let CollectionCache(mut cache) = use_context();
    use_hook(move || {
        let Some(signed_in) = session.peek().clone() else {
            return;
        };
        let user_id = signed_in.user.id;
        if cache.peek().is_some_and(|cached| cached.user_id == user_id) {
            return;
        }
        spawn(async move {
            match client().fetch_collection_summary(&signed_in).await {
                Ok(summary) => cache.set(Some(CachedCollection { user_id, summary })),
                Err(e) => tracing::warn!("collection summary not fetched: {e}"),
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
