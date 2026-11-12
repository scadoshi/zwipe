//! The melded cards behind meld halves, fetched at most once a session for
//! the shared meld toggle on card images and card details.

use std::collections::{HashMap, HashSet};

use components::{MeldFetch, MeldSource};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;

/// Provides the session's [`MeldSource`] under the root: the melded cards
/// asked for so far, and the fetch that brings one, by its Scryfall id, from
/// the public card route. A failed fetch is logged and left as
/// [`MeldFetch::Failed`], so the half keeps showing and a later tap tries
/// again. Call once, in the root component, after the client is provided.
pub fn use_meld_source(client: Signal<ZwipeClient>) {
    let mut results: Signal<HashMap<Uuid, MeldFetch>> =
        use_hook(|| Signal::new_in_scope(HashMap::new(), ScopeId::ROOT));
    let request = use_callback(move |id: Uuid| {
        results.write().insert(id, MeldFetch::Loading);
        spawn(async move {
            let http = client.peek().clone();
            let fetched = match http.fetch_card(id).await {
                Ok(card) => MeldFetch::Ready(Box::new(card.scryfall_data)),
                Err(e) => {
                    tracing::warn!("melded card {id} failed to load: {e}");
                    MeldFetch::Failed
                }
            };
            results.write().insert(id, fetched);
        });
    });
    let on: Signal<HashSet<Uuid>> =
        use_hook(|| Signal::new_in_scope(HashSet::new(), ScopeId::ROOT));
    use_context_provider(|| MeldSource {
        results,
        on,
        request,
    });
}
