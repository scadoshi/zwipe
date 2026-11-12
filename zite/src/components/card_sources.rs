//! The card data the shared components ask zite for: the melded card behind a meld half and a
//! card's printings, each fetched from the public card routes into a results signal the
//! components read. Components never call the API themselves.

use std::collections::{HashMap, HashSet};

use components::{MeldFetch, MeldSource, PrintingsFetch, PrintingsSource};
use dioxus::prelude::*;
use uuid::Uuid;

use crate::api;

/// Provides a [`MeldSource`] under the calling component: the melded cards asked for while it is
/// mounted, each fetched by its Scryfall id. A failed fetch stays [`MeldFetch::Failed`], so the
/// half keeps showing and a later tap tries again.
pub(crate) fn use_meld_source() {
    let mut results: Signal<HashMap<Uuid, MeldFetch>> = use_signal(HashMap::new);
    let request = use_callback(move |id: Uuid| {
        results.write().insert(id, MeldFetch::Loading);
        spawn(async move {
            let fetched = api::client()
                .fetch_card(id)
                .await
                .map_or(MeldFetch::Failed, |card| {
                    MeldFetch::Ready(Box::new(card.scryfall_data))
                });
            results.write().insert(id, fetched);
        });
    });
    let on: Signal<HashSet<Uuid>> = use_signal(HashSet::new);
    use_context_provider(|| MeldSource {
        results,
        on,
        request,
    });
}

/// Provides a [`PrintingsSource`] under the calling component: each card's printings, by its
/// Oracle id, fetched when the printing sheet asks. A failed fetch is
/// [`PrintingsFetch::Failed`], which the sheet offers to retry.
pub(crate) fn use_printings_source() {
    let mut results: Signal<HashMap<Uuid, PrintingsFetch>> = use_signal(HashMap::new);
    let request = use_callback(move |oracle_id: Uuid| {
        results.write().insert(oracle_id, PrintingsFetch::Loading);
        spawn(async move {
            let fetch = api::client()
                .fetch_printings(oracle_id)
                .await
                .map_or(PrintingsFetch::Failed, PrintingsFetch::Ready);
            results.write().insert(oracle_id, fetch);
        });
    });
    use_context_provider(|| PrintingsSource { results, request });
}
