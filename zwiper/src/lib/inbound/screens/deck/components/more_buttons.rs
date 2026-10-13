use crate::inbound::{
    components::{
        auth::authed::use_authed,
        telemetry::{
            flush_loop::flush_once,
            usage_buffer::UsageBuffer,
            vocabulary::{DeckScreen, Screen, component},
        },
    },
    router::Router,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_components::{AlertDialog, BottomSheet, Button, DialogAction, TOAST_QUICK};
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
    let authed = use_authed(Screen::Deck(DeckScreen::View));
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
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };

    let share_deck = use_callback(move |()| {
        show_more_sheet.set(false);
        spawn(async move {
            if let Some(result) = authed
                .run_at(component::MORE_BUTTONS, "share_deck", |c, s| async move {
                    c.share_deck(deck_id, &s).await
                })
                .await
            {
                share_token.set(Some(result.share_token));
                copy_link(result.share_token);
            }
        });
    });

    let copy_shared_link = use_callback(move |()| {
        show_more_sheet.set(false);
        if let Some(token) = *share_token.peek() {
            copy_link(token);
        }
    });

    let stop_sharing = use_callback(move |()| {
        show_more_sheet.set(false);
        spawn(async move {
            if authed
                .run_at(component::MORE_BUTTONS, "unshare_deck", |c, s| async move {
                    c.unshare_deck(deck_id, &s).await
                })
                .await
                .is_some()
            {
                share_token.set(None);
                toast.info(
                    "Link disabled".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    let clear_skips = use_callback(move |()| {
        show_more_sheet.set(false);
        spawn(async move {
            // Flush pending skips first so this window's not-yet-sent skips
            // are wiped too, not re-suppressed by the next flush.
            flush_once(&usage_buffer(), &client, &session).await;
            if authed
                .run_at(
                    component::MORE_BUTTONS,
                    "clear_suppressions",
                    |c, s| async move { c.clear_deck_suppressions(deck_id, &s).await },
                )
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });

    let buy_description = if tcg_url.is_none() && ck_url.is_none() {
        "No buy links are available for this deck."
    } else {
        "Buy this deck's cards from a retailer."
    };
    let buy_actions: Vec<DialogAction> = std::iter::once(DialogAction::Dismiss("Close".into()))
        .chain(tcg_url.map(|href| DialogAction::Link {
            label: "TCGplayer \u{2197}".into(),
            href,
        }))
        .chain(ck_url.map(|href| DialogAction::Link {
            label: "Card Kingdom \u{2197}".into(),
            href,
        }))
        .collect();

    let (share_description, share_actions) = if share_token().is_some() {
        (
            "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one.",
            vec![
                DialogAction::Dismiss("Close".into()),
                DialogAction::Confirm {
                    label: "Copy link".into(),
                    danger: false,
                    on_confirm: copy_shared_link,
                },
                DialogAction::Confirm {
                    label: "Stop share".into(),
                    danger: true,
                    on_confirm: stop_sharing,
                },
            ],
        )
    } else {
        (
            "Anyone with the link can view this deck. You can stop sharing at any time.",
            vec![
                DialogAction::Dismiss("Close".into()),
                DialogAction::Confirm {
                    label: "Create link".into(),
                    danger: false,
                    on_confirm: share_deck,
                },
            ],
        )
    };

    rsx! {
        AlertDialog {
            open: show_buy_dialog,
            title: "Buy deck",
            actions: buy_actions,
            "{buy_description}"
        }

        AlertDialog {
            open: show_clear_skips_dialog,
            title: "Clear skips",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_skips,
                },
            ],
            "Cards you've skipped or removed will start showing up again when adding cards. This can't be undone."
        }

        // One dialog covers the whole share lifecycle (mirrors the Buy-deck
        // dialog). "Stop sharing" acts directly from here; the dialog itself
        // is the deliberate surface, and the description carries the caveat,
        // so no second confirmation dialog stacks on top.
        AlertDialog {
            open: show_share_dialog,
            title: "Share deck",
            actions: share_actions,
            "{share_description}"
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
