//! Universes Beyond exceptions sheet: chip picker for the franchises still
//! served while the Universes Beyond setting is on Hide. The Show/Hide
//! toggle itself lives on the Config screen; this sheet edits only
//! the whitelist. The server applies both at serve time.

use components::{BottomSheet, Button, ButtonVariant, HintDialog, HintLine, use_toast};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_UNIVERSES_BEYOND_EXCEPTIONS;
use zwipe_core::http::contracts::user::HttpUpdatePreferences;
use zwipe_ui::filter::fields;
use zwipe_ui::preferences::{self, PreferenceChange};

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen};
use crate::inbound::components::{guide_link, hint_dialog, ui_toast};

/// Bottom sheet editing the exceptions whitelist. `exceptions` is the parent's
/// saved list: the sheet copies it into a draft on open and writes it back on
/// a successful Save; Back and the backdrop discard the draft. `hint_open` is
/// owned by the Config screen so its row-level "?" opens the same explainer
/// as the sheet's own header "?".
#[component]
pub fn UniversesBeyondExceptionsSheet(
    mut open: Signal<bool>,
    mut exceptions: Signal<Vec<String>>,
    hint_open: Signal<bool>,
) -> Element {
    // session + client feed the hint recorder, which stays outside the facade
    // (it is not an authed data call).
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();
    let toast = use_toast();

    // Server-driven franchise catalog; no compiled fallback. A failed fetch
    // renders no chips and is reported by the catalog-failure toast, which is
    // the right outcome: the picker writes a preference, so it needs the
    // server regardless of what it could render.
    use_effect(move || cache.ensure_ub_franchises(client));
    let authed = use_authed(Screen::Profile(ProfileScreen::Preferences));

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
                hint_dialog::open_and_record_hint(
                    HINT_UNIVERSES_BEYOND_EXCEPTIONS,
                    session,
                    client,
                    hint_open,
                );
            }
        }
    });

    let discard = use_callback(move |()| {
        if !fields::same_set(&draft.peek(), &exceptions.peek()) {
            ui_toast::raise(toast, preferences::exceptions_unchanged());
        }
        open.set(false);
    });

    let mut save = move || {
        let change = PreferenceChange::Exceptions(draft());
        let request = HttpUpdatePreferences::from(change.clone());
        open.set(false);
        spawn(async move {
            if let Some(prefs) = authed
                .run("update_preferences", |c, s| async move {
                    c.update_preferences(request, &s).await
                })
                .await
            {
                let saved = change.saved_toast(&prefs);
                exceptions.set(prefs.universes_beyond_exceptions);
                ui_toast::raise(toast, saved);
            }
        });
    };

    let unchanged = fields::same_set(&draft(), &exceptions());

    rsx! {
        BottomSheet {
            open,
            title: "Exceptions".to_string(),
            hint: hint_open,
            on_dismiss: move |()| discard.call(()),
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| discard.call(()),
                    "Back"
                }
                Button {
                    variant: ButtonVariant::Action,
                    disabled: unchanged,
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
                    // Served, not compiled: a new crossover release becomes
                    // selectable on a deploy. Already sorted by the server.
                    let franchises = cache
                        .ub_franchises
                        .cell()
                        .read()
                        .loaded()
                        .cloned()
                        .unwrap_or_default();
                    rsx! {
                        for franchise in franchises {
                            {
                                let is_selected = draft().contains(&franchise.slug);
                                rsx! {
                                    div {
                                        class: if is_selected { "chip selected" } else { "chip" },
                                        onclick: move |_| draft.set(fields::toggled(draft(), &franchise.slug)),
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
            actions: guide_link::one_time(HINT_UNIVERSES_BEYOND_EXCEPTIONS),
            title: "Franchise exceptions",
            HintLine { "Picked franchises still show while Universes Beyond is hidden" }
        }
    }
}
