//! The cards' stats the card details drawer shows, each fetched at most once a session unless it
//! fails.

use std::collections::HashMap;

use components::CardStatsSource;
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_ui::screens::card_lookup::CardStats;

/// Provides the session's [`CardStatsSource`] under the root: the cards' stats asked for so far,
/// and the fetch that brings one, by the card's Oracle id, from the public card stats route. A
/// failed fetch is logged and left as [`CardStats::Failed`], which offers Retry. Call once, in the
/// root component, after the client is provided.
pub fn use_card_stats_source(client: Signal<ZwipeClient>) {
    let mut results: Signal<HashMap<Uuid, CardStats>> =
        use_hook(|| Signal::new_in_scope(HashMap::new(), ScopeId::ROOT));
    let request = use_callback(move |oracle_id: Uuid| {
        results.write().insert(oracle_id, CardStats::Loading);
        spawn(async move {
            let http = client.peek().clone();
            let fetched = match http.fetch_card_stats(oracle_id).await {
                Ok(stats) => CardStats::Loaded(stats),
                Err(e) => {
                    tracing::warn!("stats of card {oracle_id} failed to load: {e}");
                    CardStats::Failed
                }
            };
            results.write().insert(oracle_id, fetched);
        });
    });
    use_context_provider(|| CardStatsSource { results, request });
}
