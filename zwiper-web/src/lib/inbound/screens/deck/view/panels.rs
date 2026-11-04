//! The two panels at the foot of the deck view: Data holds Import and
//! Export, Danger zone holds Clear skips and Delete, titled and buttoned
//! in red.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_ui::screens::deck_view::{DANGER_ZONE, DATA, MoreItem};

use super::dialogs::DeckDialogs;
use crate::inbound::router::Route;

/// The Data panel for `deck_id`: Import, and Export once the deck has a
/// card.
#[component]
pub fn DataPanel(deck_id: Uuid, has_cards: bool) -> Element {
    let navigator = use_navigator();
    rsx! {
        div { class: "settings-list",
            div { class: "settings-list-header",
                span { class: "settings-list-title", {DATA} }
            }
            ActionRow {
                item: MoreItem::ImportCards,
                onclick: move |()| {
                    navigator.push(Route::ImportDeck { deck_id });
                },
            }
            ActionRow {
                item: MoreItem::ExportCards,
                disabled: !has_cards,
                onclick: move |()| {
                    navigator.push(Route::ExportDeck { deck_id });
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
        div { class: "settings-list danger-zone",
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
