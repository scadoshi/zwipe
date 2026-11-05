//! Universes Beyond exceptions sheet: chip picker for the franchises still
//! served while the Universes Beyond setting is on Hide. The Show/Hide
//! toggle itself lives on the Config screen; this sheet edits only
//! the whitelist. The server applies both at serve time.

use components::{BottomSheet, Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::screens::config::{self as model, Action};

use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::OneTimeHintDialog;

/// Bottom sheet editing the exceptions whitelist. Config's model holds the
/// draft, seeded from the saved list on each open; the sheet draws `draft` and
/// `unchanged` from it and hands every tap on as an
/// [`Action`]. Save sends the draft; Back and the backdrop discard it.
/// `hint_open` is owned by the Config screen so its row-level "?" opens the
/// same explainer as the sheet's own header "?".
#[component]
pub fn UniversesBeyondExceptionsSheet(
    open: Signal<bool>,
    hint_open: Signal<bool>,
    draft: Vec<String>,
    unchanged: bool,
    on: EventHandler<Action>,
) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let cache: CatalogCache = use_context();

    // Server-driven franchise catalog; no compiled fallback. A failed fetch
    // renders no chips and is reported by the catalog-failure toast, which is
    // the right outcome: the picker writes a setting, so it needs the
    // server regardless of what it could render.
    use_effect(move || cache.ensure_ub_franchises(client));

    rsx! {
        BottomSheet {
            open,
            title: model::EXCEPTIONS.to_string(),
            hint: hint_open,
            on_dismiss: move |()| on.call(Action::DiscardExceptions),
            closable: true,
            footer: rsx! {
                Button {
                    variant: ButtonVariant::Action,
                    disabled: unchanged,
                    onclick: move |_| on.call(Action::SaveExceptions),
                    {model::SAVE}
                }
            },

            // Clear-all for the draft; like every edit here it only persists
            // through Save. Always mounted inside a collapsible so the count
            // and the clear button ease in and out instead of popping.
            div {
                class: if draft.is_empty() { "collapsible" } else { "collapsible open" },
                div { class: "collapsible-inner",
                    div { class: "label-row",
                        label { class: "label-xs", { model::selected_line(draft.len()) } }
                        button {
                            class: "clear-btn",
                            onclick: move |_| on.call(Action::ClearExceptions),
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
                                let is_selected = draft.contains(&franchise.slug);
                                let slug = franchise.slug.clone();
                                rsx! {
                                    div {
                                        class: if is_selected { "chip selected" } else { "chip" },
                                        onclick: move |_| on.call(Action::ToggleException(slug.clone())),
                                        { franchise.name }
                                    }
                                }
                            }
                        }
                    }
                }
            }
        }

        OneTimeHintDialog { open: hint_open, hint: OneTimeHint::UniversesBeyondExceptions }
    }
}
