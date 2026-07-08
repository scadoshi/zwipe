use crate::{
    inbound::{
        components::{
            alert_dialog::{
                AlertDialogAction, AlertDialogActions, AlertDialogCancel, AlertDialogContent,
                AlertDialogDescription, AlertDialogRoot, AlertDialogTitle,
            },
            auth::ensure_session::EnsureFresh,
            bottom_sheet::BottomSheet,
            telemetry::{flush_loop::flush_once, usage_buffer::UsageBuffer},
        },
        router::Router,
    },
    outbound::client::{
        ZwipeClient,
        deck::{clear_deck_suppressions::ClientClearDeckSuppressions, share_deck::ClientShareDeck},
    },
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use uuid::Uuid;
use zwipe_components::Button;
use zwipe_core::domain::{auth::models::session::Session, site::WEB_BASE};

#[component]
pub(crate) fn MoreButtons(
    deck_id: Uuid,
    show_buy_dialog: Signal<bool>,
    show_more_sheet: Signal<bool>,
    show_delete_dialog: Signal<bool>,
    show_clone_dialog: Signal<bool>,
    /// Live share-link token: `Some` when the deck has a public link. Updated
    /// in place by the Share / Stop-sharing actions.
    share_token: Signal<Option<Uuid>>,
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
    let mut show_share_dialog = use_signal(|| false);

    // Copies a shared-deck link to the clipboard and toasts. The token is a
    // UUID, so it needs no escaping, but we serialize for safety anyway.
    let copy_link = move |token: Uuid| {
        let url = format!("{WEB_BASE}/deck/{token}");
        let js = format!(
            "navigator.clipboard.writeText({})",
            serde_json::to_string(&url).unwrap_or_default()
        );
        document::eval(&js);
        toast.info(
            "Share link copied".to_string(),
            ToastOptions::default().duration(Duration::from_millis(2000)),
        );
    };

    let share_deck = move || {
        spawn(async move {
            let Ok(fresh) = session.ensure_fresh(client).await else {
                toast.error("Session expired".to_string(), ToastOptions::default());
                return;
            };
            match client().share_deck(deck_id, &fresh).await {
                Ok(result) => {
                    share_token.set(Some(result.share_token));
                    copy_link(result.share_token);
                }
                Err(e) => {
                    tracing::warn!("share deck failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

    let stop_sharing = move || {
        spawn(async move {
            let Ok(fresh) = session.ensure_fresh(client).await else {
                toast.error("Session expired".to_string(), ToastOptions::default());
                return;
            };
            match client().unshare_deck(deck_id, &fresh).await {
                Ok(_) => {
                    share_token.set(None);
                    toast.info(
                        "Link disabled".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("unshare deck failed: {e}");
                    toast.error(e.to_user_message(), ToastOptions::default());
                }
            }
        });
    };

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
                    AlertDialogDescription { "No buy links are available for this deck." }
                } else {
                    AlertDialogDescription { "Buy this deck's cards from a retailer." }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| show_buy_dialog.set(false),
                        "Close"
                    }
                    if let Some(ref url) = tcg_url {
                        a {
                            class: "alert-dialog-action",
                            href: "{url}",
                            target: "_blank",
                            rel: "noopener",
                            onclick: move |_| show_buy_dialog.set(false),
                            "TCGplayer ↗"
                        }
                    }
                    if let Some(ref url) = ck_url {
                        a {
                            class: "alert-dialog-action",
                            href: "{url}",
                            target: "_blank",
                            rel: "noopener",
                            onclick: move |_| show_buy_dialog.set(false),
                            "Card Kingdom ↗"
                        }
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

        // One dialog covers the whole share lifecycle (mirrors the Buy-deck
        // dialog). "Stop sharing" acts directly from here — the dialog itself
        // is the deliberate surface, and the description carries the caveat,
        // so no second confirmation dialog stacks on top.
        AlertDialogRoot {
            open: show_share_dialog(),
            on_open_change: move |open| show_share_dialog.set(open),
            AlertDialogContent {
                AlertDialogTitle { "Share deck" }
                hr { class: "dialog-rule" }
                if share_token().is_some() {
                    AlertDialogDescription {
                        "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one."
                    }
                } else {
                    AlertDialogDescription {
                        "Anyone with the link can view this deck. You can stop sharing at any time."
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogCancel {
                        on_click: move |_| show_share_dialog.set(false),
                        "Close"
                    }
                    if let Some(token) = share_token() {
                        AlertDialogAction {
                            on_click: move |_| {
                                show_share_dialog.set(false);
                                show_more_sheet.set(false);
                                copy_link(token);
                            },
                            "Copy link"
                        }
                        AlertDialogAction {
                            danger: true,
                            on_click: move |_| {
                                show_share_dialog.set(false);
                                show_more_sheet.set(false);
                                stop_sharing();
                            },
                            "Stop share"
                        }
                    } else {
                        AlertDialogAction {
                            on_click: move |_| {
                                show_share_dialog.set(false);
                                show_more_sheet.set(false);
                                share_deck();
                            },
                            "Create link"
                        }
                    }
                }
            }
        }

        BottomSheet { open: show_more_sheet, title: "More actions",
            Button {
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ImportDeck { deck_id });
                },
                "Import cards"
            }
            Button {
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    navigator.push(Router::ExportDeck { deck_id });
                },
                "Export cards"
            }
            Button {
                disabled: !has_cards,
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_buy_dialog.set(true);
                },
                "Buy deck"
            }
            Button {
                onclick: move |_| {
                    show_more_sheet.set(false);
                    show_clone_dialog.set(true);
                },
                "Clone deck"
            }
            Button {
                onclick: move |_| {
                    show_share_dialog.set(true);
                },
                "Share deck"
            }
            Button {
                danger: true,
                onclick: move |_| {
                    show_clear_skips_dialog.set(true);
                },
                "Clear skips"
            }
            Button {
                danger: true,
                onclick: move |_| {
                    show_delete_dialog.set(true);
                },
                "Delete deck"
            }
        }
    }
}
