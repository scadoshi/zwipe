//! Universes Beyond exceptions sheet: chip picker for the franchises still
//! served while the profile's Universes Beyond setting is on Hide. The
//! Show/Hide toggle itself lives on the profile screen; this sheet edits only
//! the whitelist. The server applies both at serve time.

use crate::{
    inbound::components::{
        auth::ensure_session::EnsureFresh,
        bottom_sheet::BottomSheet,
        hint_dialog::{HintBullet, HintBullets, HintDialog, HintKey, open_and_record_hint},
        telemetry::{
            usage_buffer::UsageBuffer,
            vocabulary::{component, screen},
        },
    },
    outbound::client::{ZwipeClient, user::preferences::ClientUpdatePreferences},
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use std::time::Duration;
use zwipe_components::{Button, ButtonVariant};
use zwipe_core::{
    domain::{
        auth::models::session::Session, card::scryfall_data::universe::selectable_franchises,
        user::models::hints::HINT_UNIVERSES_BEYOND_EXCEPTIONS,
    },
    http::contracts::user::HttpUpdatePreferences,
};

/// Bottom sheet editing the exceptions whitelist. `exceptions` is the parent's
/// saved list: the sheet copies it into a draft on open and writes it back on
/// a successful Save; Back and the backdrop discard the draft. `hint_open` is
/// owned by the profile screen so its row-level "?" opens the same explainer
/// as the sheet's own header "?".
#[component]
pub fn UniversesBeyondExceptionsSheet(
    mut open: Signal<bool>,
    mut exceptions: Signal<Vec<String>>,
    hint_open: Signal<bool>,
) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();

    let mut draft: Signal<Vec<String>> = use_signal(Vec::new);

    // Exceptions explainer: auto-opens once per account the first time the
    // sheet opens. Gated like the filter sheet's hint, since this component
    // stays mounted while the sheet is closed.
    let mut hint_fired = use_signal(|| false);

    // Seed the draft from the saved list each open.
    use_effect(move || {
        if open() {
            draft.set(exceptions.peek().clone());
            if !*hint_fired.peek() {
                hint_fired.set(true);
                open_and_record_hint(HINT_UNIVERSES_BEYOND_EXCEPTIONS, session, client, hint_open);
            }
        }
    });

    let mut save = move || {
        let list = draft();
        let request = HttpUpdatePreferences {
            theme: None,
            dark_mode: None,
            exclude_universes_beyond: None,
            universes_beyond_exceptions: Some(list.clone()),
        };
        open.set(false);
        spawn(async move {
            let session_val = match session.ensure_fresh(client).await {
                Ok(session_val) => session_val,
                Err(e) => {
                    usage_buffer.peek().report_error(
                        screen::PROFILE_PREFERENCES,
                        component::NONE,
                        "update_preferences",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                    return;
                }
            };
            match client().update_preferences(request, &session_val).await {
                Ok(prefs) => {
                    exceptions.set(prefs.universes_beyond_exceptions);
                    toast.success(
                        "Exceptions saved".to_string(),
                        ToastOptions::default().duration(Duration::from_millis(1500)),
                    );
                }
                Err(e) => {
                    tracing::warn!("update preferences failed: {e}");
                    usage_buffer.peek().report_error(
                        screen::PROFILE_PREFERENCES,
                        component::NONE,
                        "update_preferences",
                        &e,
                    );
                    toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(Duration::from_millis(3000)),
                    );
                }
            }
        });
    };

    rsx! {
        BottomSheet {
            open,
            title: "Exceptions".to_string(),
            hint: hint_open,
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| open.set(false),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Util,
                    onclick: move |_| save(),
                    "Save"
                }
            },

            // Clear-all for the draft; like every edit here it only persists
            // through Save. Always mounted inside a collapsible so the count
            // and the clear button ease in and out instead of popping.
            div {
                class: if draft().is_empty() { "collapsible" } else { "collapsible open" },
                div { class: "collapsible-inner",
                    div { class: "label-row",
                        label { class: "label-xs", { format!("{} selected", draft().len()) } }
                        button {
                            class: "clear-btn",
                            onclick: move |_| draft.set(Vec::new()),
                            "\u{00d7}"
                        }
                    }
                }
            }

            div { class: "flex flex-wrap gap-1 flex-center",
                {
                    let mut franchises: Vec<_> = selectable_franchises().collect();
                    franchises.sort_by_key(|f| f.name);
                    rsx! {
                        for franchise in franchises {
                            {
                                let is_selected = draft().iter().any(|s| s == franchise.slug);
                                rsx! {
                                    div {
                                        class: if is_selected { "chip selected" } else { "chip" },
                                        onclick: move |_| {
                                            let mut list = draft();
                                            if let Some(i) = list.iter().position(|s| s == franchise.slug) {
                                                list.remove(i);
                                            } else {
                                                list.push(franchise.slug.to_string());
                                            }
                                            draft.set(list);
                                        },
                                        { franchise.name }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        HintDialog {
            open: hint_open,
            title: "Franchise exceptions",
            HintBullets {
                HintBullet {
                    "While Universes Beyond is hidden, cards from a selected franchise still show up in searches and commander picks"
                }
                HintBullet {
                    "Tap a franchise to select it, tap again to drop it, then "
                    HintKey { color: "--color-success", "Save" }
                }
            }
        }
    }
}
