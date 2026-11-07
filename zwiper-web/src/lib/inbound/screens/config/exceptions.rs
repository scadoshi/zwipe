//! The Exceptions sheet, a centered modal on a wide window: the franchises
//! that still show while Universes Beyond is hidden.

use components::icons::{CHECK, DockIcon};
use components::{BottomSheet, SheetFrame, SheetWide, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::user::HttpUpdateSettings;
use zwipe_ui::filter::fields;

use crate::inbound::caches::UbFranchises;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{self, OneTimeHint};

/// Edits a draft of `exceptions`, the saved list, copied in on each open.
/// The dock's checkmark closes at once and writes the draft; the saved list
/// changes only when the server answers. The ×, a click outside, a drag down
/// and Esc discard the draft, saying `Exceptions unchanged` when it differed. The first open in a visit
/// to Config fires the one-time hint; `hint_open` is Config's, so the row's
/// `?` and the sheet's open the same dialog.
#[component]
pub(super) fn ExceptionsSheet(
    mut open: Signal<bool>,
    mut exceptions: Signal<Vec<String>>,
    hint_open: Signal<bool>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let franchises: UbFranchises = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let mut draft: Signal<Vec<String>> = use_signal(Vec::new);
    let mut hint_fired = use_signal(|| false);

    use_hook(move || franchises.ensure(client));
    use_effect(move || {
        if open() {
            draft.set(exceptions.peek().clone());
            if !*hint_fired.peek() {
                hint_fired.set(true);
                one_time_hint::open_and_record_hint(
                    OneTimeHint::UniversesBeyondExceptions,
                    session,
                    client,
                    hint_open,
                );
            }
        }
    });

    let discard = use_callback(move |()| {
        if !fields::same_set(&draft.peek(), &exceptions.peek()) {
            toast.info(
                "Exceptions unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        open.set(false);
    });

    let save = move |_| {
        let request = HttpUpdateSettings {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: Some(draft()),
            pip_style: None,
            swipe_confirmations: None,
        };
        open.set(false);
        spawn(async move {
            if let Some(saved) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_settings(request, &s).await
                })
                .await
            {
                exceptions.set(saved.universes_beyond_exceptions);
                toast.success(
                    "Exceptions saved".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    };

    let unchanged = fields::same_set(&draft(), &exceptions());
    // Served, not compiled, and already sorted by the server. Nothing shows
    // while it loads or after it failed.
    let offered = franchises.0.read().loaded().cloned().unwrap_or_default();

    rsx! {
        SheetFrame { wide: SheetWide::Modal,
            BottomSheet {
                open,
                title: "Exceptions".to_string(),
                hint: hint_open,
                on_dismiss: move |()| discard.call(()),
                closable: true,
                div { class: if draft().is_empty() { "collapsible" } else { "collapsible open" },
                    div { class: "collapsible-inner",
                        div { class: "label-row",
                            label { class: "label-xs", "{draft().len()} selected" }
                            button {
                                class: "clear-btn",
                                onclick: move |_| draft.set(Vec::new()),
                                "\u{00d7}"
                            }
                        }
                    }
                }
                div { class: "flex flex-wrap gap-1 flex-center",
                    for franchise in offered {
                        div {
                            key: "{franchise.slug}",
                            class: if draft().contains(&franchise.slug) { "chip selected" } else { "chip" },
                            onclick: move |_| {
                                let next = fields::toggled(draft.peek().clone(), &franchise.slug);
                                draft.set(next);
                            },
                            "{franchise.name}"
                        }
                    }
                }
                // Save floats at the sheet's bottom right, off while nothing
                // differs from the saved list.
                div { class: "action-dock action-dock-right sheet-dock",
                    button {
                        class: "action-dock-btn",
                        "aria-label": "Save",
                        disabled: unchanged,
                        onclick: save,
                        DockIcon { path: CHECK }
                    }
                }
            }
        }
    }
}
