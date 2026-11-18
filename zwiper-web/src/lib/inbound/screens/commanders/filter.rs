//! The Commanders screen's one filter, over both lists: the action dock's
//! funnel and the filter sheet it opens.

use components::DockFilter;
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;

use crate::inbound::card_filter::CardFilterPanel;

/// The funnel for the screen's `filter`, its dot while a filter or a sort is set and its reset
/// while a filter is on, opening the sheet through `filter_open`.
#[component]
pub(super) fn FilterButton(filter: Signal<CardQueryBuilder>, filter_open: Signal<bool>) -> Element {
    let mut filter = filter;
    let mut filter_open = filter_open;
    let applied = filter.read().clone();
    rsx! {
        div { class: "action-dock action-dock-right",
            DockFilter {
                label: "Filter",
                dot: filter::shows_dot(&applied),
                active: !applied.is_empty(),
                onclick: move |()| filter_open.set(true),
                on_reset: move |()| filter.set(CardQueryBuilder::new()),
                reset_label: "Reset filter",
            }
        }
    }
}

/// The filter sheet for the screen's `filter`, open while `filter_open`.
#[component]
pub(super) fn FilterSheet(filter: Signal<CardQueryBuilder>, filter_open: Signal<bool>) -> Element {
    let mut filter = filter;
    rsx! {
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig::default(),
            quiet_reset: true,
            on_apply: move |applied: CardQueryBuilder| filter.set(applied),
        }
    }
}
