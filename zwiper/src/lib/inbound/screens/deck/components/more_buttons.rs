use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
    AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
};
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::bottom_sheet::BottomSheet;
use crate::inbound::components::telemetry::{flush_loop::flush_once, usage_buffer::UsageBuffer};
use crate::inbound::router::Router;
use crate::outbound::client::ZwipeClient;
use crate::outbound::client::deck::clear_deck_suppressions::ClientClearDeckSuppressions;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_core::domain::auth::models::session::Session;

#[component]
pub(crate) fn MoreButtons(
    deck_id: Uuid,
    show_buy_dialog: Signal<bool>,
    show_more_sheet: Signal<bool>,
    show_delete_dialog: Signal<bool>,
    show_clone_dialog: Signal<bool>,
    has_cards: bool,
    tcg_url: Option<String>,
    ck_url: Option<String>,
) -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    let toast = use_toast();
    let mut show_clear_skips_dialog = use_signal(|| false);

    let clear_skips = move || {
        spawn(async move {
            // Flush pending skips first so this window's not-yet-sent skips
            // are wiped too, not re-suppressed by the next flush.
            flush_once(&usage_buffer(), &client, &session).await;
            let Ok(fresh) = session.ensure_fresh(client).await else {
                toast.error("Session expired".to_string(), ToastOptions::default());
                return;
            };
            match client().clear_deck_suppressions(deck_id, &fresh).await {
                Ok(_) => {
                    toast.info(
                        "Skips cleared".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("clear suppressions failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    rsx! {
        AlertDialogRoot {
            open: show_buy_dialog(),
            on_open_change: move |open| show_buy_dialog.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Buy deck" }
                hr { class: "dialog-rule" }
                if tcg_url.is_none() && ck_url.is_none() {
                    AlertDialogDescription { "No buy links available for this deck." }
                }
                if let Some(ref url) = tcg_url {
                    a {
                        class: "btn",
                        href: "{url}",
                        target: "_blank",
                        style: "text-decoration: none; text-align: center;",
                        onclick: move |_| show_buy_dialog.set(false),
                        "TCGplayer ↗"
                    }
                }
                if let Some(ref url) = ck_url {
                    a {
                        class: "btn",
                        href: "{url}",
                        target: "_blank",
                        style: "text-decoration: none; text-align: center;",
                        onclick: move |_| show_buy_dialog.set(false),
                        "Card Kingdom ↗"
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| show_buy_dialog.set(false),
                        "Close"
                    }
                }
            }
        }

        AlertDialogRoot {
            open: show_clear_skips_dialog(),
            on_open_change: move |open| show_clear_skips_dialog.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Clear skips" }
                hr { class: "dialog-rule" }
                AlertDialogDescription {
                    "Cards you've skipped or removed will start showing up again when adding cards. This can't be undone."
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| show_clear_skips_dialog.set(false),
                        "Cancel"
                    }
                    AlertDialogAction {
                        danger: true,
                        on_click: move |_| {
                            show_clear_skips_dialog.set(false);
                            show_more_sheet.set(false);
                            clear_skips();
                        },
                        "Clear"
                    }
                }
            }
        }

        BottomSheet { open: show_more_sheet, title: "More actions",
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::AddDeckCard { deck_id });
                },
                "Add cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::RemoveDeckCard { deck_id });
                },
                "Remove cards"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ImportDeck { deck_id });
                },
                "Import cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ExportDeck { deck_id });
                },
                "Export cards"
            }
            button {
                class: "btn",
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_buy_dialog.set(true);
                },
                "Buy deck"
            }
            button {
                class: "btn",
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_clone_dialog.set(true);
                },
                "Clone deck"
            }
            button {
                class: "btn btn-danger",
                onclick: move |_| {
                    show_clear_skips_dialog.set(true);
                },
                "Clear skips"
            }
            button {
                class: "btn btn-danger",
                onclick: move |_| {
                    show_delete_dialog.set(true);
                },
                "Delete deck"
            }
        }
    }
}
