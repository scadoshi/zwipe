//! The printing sheet: components' sheet over the session's printings fetch, with the web's toasts.

use std::collections::HashMap;

use components::{
    PrintingSheet as SharedPrintingSheet, PrintingsFetch, PrintingsSource, TOAST_QUICK,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_ui::telemetry::vocabulary::component;

use crate::inbound::chrome::authed::use_authed;

/// The printings of `card`, opening on the one shown, fetched for the signed-in session. With an
/// `on_save`, the sheet picks: Select or Enter hands it the showing printing and toasts
/// `saved_message`, and leaving with a different printing showing toasts `Printing unchanged`.
/// Without one it only browses: no Select, Enter does nothing, and nothing toasts.
#[component]
pub fn PrintingSheet(
    card: Card,
    open: Signal<bool>,
    #[props(default)] on_save: Option<EventHandler<Card>>,
    #[props(default = "Printing saved")] saved_message: &'static str,
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
        SharedPrintingSheet { card, open, on_save, on_unchanged }
    }
}

/// Provides the sheet's [`PrintingsSource`] under it: each fetch runs for the session, its
/// failure toasted and reported under the printing sheet.
fn use_printings_source() {
    let authed = use_authed().in_component(component::PRINTING_SHEET);
    let mut results: Signal<HashMap<Uuid, PrintingsFetch>> = use_signal(HashMap::new);
    let request = use_callback(move |oracle_id: Uuid| {
        results.write().insert(oracle_id, PrintingsFetch::Loading);
        spawn(async move {
            let fetched = authed
                .run("load_printings", |c, _| async move {
                    c.fetch_printings(oracle_id).await
                })
                .await;
            let fetch = fetched.map_or(PrintingsFetch::Failed, PrintingsFetch::Ready);
            results.write().insert(oracle_id, fetch);
        });
    });
    use_context_provider(|| PrintingsSource { results, request });
}
