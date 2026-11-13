//! The two panels at the foot of the deck view: Data holds Import (the
//! owner's) and Export, Danger zone holds Clear skips and Delete, titled and buttoned
//! in red.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_ui::screens::deck_view::{DANGER_ZONE, DATA, MoreItem};

use super::dialogs::DeckDialogs;
use crate::inbound::router::Route;

/// The Data panel: Import where the owner's `import` route is given, and
/// Export (to the `export` route) once the deck has a card. A shared deck
/// offers Export alone.
#[component]
pub fn DataPanel(import: Option<Route>, export: Route, has_cards: bool) -> Element {
    let navigator = use_navigator();
    rsx! {
        div { class: "settings-list", "data-settle-key": "data",
            div { class: "settings-list-header",
                span { class: "settings-list-title", {DATA} }
            }
            if let Some(route) = import {
                ActionRow {
                    item: MoreItem::ImportCards,
                    onclick: move |()| {
                        navigator.push(route.clone());
                    },
                }
            }
            ActionRow {
                item: MoreItem::ExportCards,
                disabled: !has_cards,
                onclick: move |()| {
                    navigator.push(export.clone());
                },
            }
        }
    }
}

/// The Danger zone panel: Clear skips and Delete, each opening its dialog.
#[component]
pub fn DangerZonePanel(dialogs: DeckDialogs) -> Element {
    let DeckDialogs {
        mut clear,
        mut delete,
        ..
    } = dialogs;
    rsx! {
        div { class: "settings-list danger-zone", "data-settle-key": "danger",
            div { class: "settings-list-header",
                span { class: "settings-list-title danger-zone-title", {DANGER_ZONE} }
            }
            ActionRow { item: MoreItem::ClearSkips, onclick: move |()| clear.set(true) }
            ActionRow { item: MoreItem::DeleteDeck, onclick: move |()| delete.set(true) }
        }
    }
}

/// One row of a panel: the action's name, then its button, red for a
/// destructive one.
#[component]
fn ActionRow(
    item: MoreItem,
    #[props(default)] disabled: bool,
    onclick: EventHandler<()>,
) -> Element {
    rsx! {
        div { class: "settings-row",
            span { class: "settings-row-label", {item.label()} }
            div { class: "settings-row-value",
                Button {
                    variant: ButtonVariant::Action,
                    disabled,
                    danger: item.is_danger(),
                    onclick: move |_| onclick.call(()),
                    {item.short_label()}
                }
            }
        }
    }
}
