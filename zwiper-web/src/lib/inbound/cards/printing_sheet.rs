//! The printing sheet: components' sheet over the session's printings fetch, with the web's toasts.

use std::collections::{HashMap, HashSet};

use components::{
    OwnedPrintingsSource, PrintingSheet as SharedPrintingSheet, PrintingsFetch, PrintingsSource,
    TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_ui::telemetry::vocabulary::component;

use crate::inbound::chrome::authed::use_authed;
use crate::inbound::collection::use_offers_owned;

/// The printings of `card`, opening on the one shown, fetched for the signed-in session. With an
/// `on_save`, the sheet picks: Select or Enter hands it the showing printing and toasts
/// `saved_message`, and leaving with a different printing showing toasts `Printing unchanged`.
/// Without one it only browses: no Select, Enter does nothing, and nothing toasts. The printings
/// open on `face`, the face `card`'s row shows, or without one on the face its details drawer
/// handed over.
#[component]
pub fn PrintingSheet(
    card: Card,
    open: Signal<bool>,
    #[props(default)] on_save: Option<EventHandler<Card>>,
    #[props(default = "Printing saved")] saved_message: &'static str,
    #[props(default)] face: Option<usize>,
) -> Element {
    use_printings_source();
    let toast = use_toast();
    let on_save = on_save.map(|on_save| {
        EventHandler::new(move |picked: Card| {
            on_save.call(picked);
            toast.info(
                saved_message.to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        })
    });
    let on_unchanged = EventHandler::new(move |()| {
        toast.info(
            "Printing unchanged".to_string(),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    });
    rsx! {
        SharedPrintingSheet { card, open, on_save, on_unchanged, face }
    }
}

/// Provides the sheet's [`PrintingsSource`] under it: each fetch runs for the session, its
/// failure toasted and reported under the printing sheet. Signed out (a shared deck's reader) it
/// runs on the public route with no session, and a failure offers Retry.
pub(crate) fn use_printings_source() {
    let authed = use_authed().in_component(component::PRINTING_SHEET);
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut results: Signal<HashMap<Uuid, PrintingsFetch>> = use_signal(HashMap::new);
    let request = use_callback(move |oracle_id: Uuid| {
        results.write().insert(oracle_id, PrintingsFetch::Loading);
        let signed_in = session.peek().is_some();
        spawn(async move {
            let fetched = if signed_in {
                authed
                    .run("load_printings", |c, _| async move {
                        c.fetch_printings(oracle_id).await
                    })
                    .await
            } else {
                let http = client.peek().clone();
                http.fetch_printings(oracle_id).await.ok()
            };
            let fetch = fetched.map_or(PrintingsFetch::Failed, PrintingsFetch::Ready);
            results.write().insert(oracle_id, fetch);
        });
    });
    use_context_provider(|| PrintingsSource { results, request });

    // The user's owned printings, fetched only when they have a collection, quietly: a failure
    // leaves Owned unoffered rather than toasting over the sheet.
    let offers_owned = use_offers_owned();
    let mut owned: Signal<HashMap<Uuid, HashSet<Uuid>>> = use_signal(HashMap::new);
    // The card asked for before the summary said there is a collection, sent once it does.
    let mut waiting: Signal<Option<Uuid>> = use_signal(|| None);
    let fetch_owned = use_callback(move |oracle_id: Uuid| {
        if owned.peek().contains_key(&oracle_id) {
            return;
        }
        spawn(async move {
            let fetched = authed
                .run_quiet("load_owned_printings", |c, s| async move {
                    c.fetch_owned_printings(oracle_id, &s).await
                })
                .await;
            if let Some(cards) = fetched {
                let ids = cards.into_iter().map(|c| c.scryfall_data.id).collect();
                owned.write().insert(oracle_id, ids);
            }
        });
    });
    let request_owned = use_callback(move |oracle_id: Uuid| {
        if *offers_owned.peek() {
            fetch_owned.call(oracle_id);
        } else {
            waiting.set(Some(oracle_id));
        }
    });
    use_effect(move || {
        if offers_owned()
            && let Some(oracle_id) = waiting.write().take()
        {
            fetch_owned.call(oracle_id);
        }
    });
    use_context_provider(|| OwnedPrintingsSource {
        results: owned,
        request: request_owned,
    });
}
