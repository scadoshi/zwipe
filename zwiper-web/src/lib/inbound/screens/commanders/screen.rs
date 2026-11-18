//! The Commanders screen: [`MaybeboardPanel`] and [`SuggestedPanel`] in the
//! page grammar's two columns on a wide window, Maybeboard on the left, and
//! one after the other below the breakpoint. The dock's Filter is the
//! screen's: what it sets narrows both lists, and each list keeps its own
//! Show row on top of that.

use components::{ActionDock, DockFilter};
use dioxus::prelude::*;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_ui::filter;
use zwipe_ui::filter::draft::SheetConfig;
use zwipe_ui::telemetry::vocabulary::Screen;

use super::maybeboard::MaybeboardPanel;
use super::suggested::SuggestedPanel;
use crate::inbound::card_filter::CardFilterPanel;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The screen's title, which the nav's entry also reads.
pub const TITLE: &str = "Commanders";

/// The Commanders screen, at `/commanders`.
#[component]
pub fn CommanderMaybeboard() -> Element {
    telemetry::use_telemetry_screen(Screen::CommanderMaybeboard);
    let hint_open = use_one_time_hint(OneTimeHint::CommanderMaybeboard);
    let mut filter = use_signal(CardQueryBuilder::new);
    let mut filter_open = use_signal(|| false);

    rsx! {
        ScreenFrame {
            title: TITLE,
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content",
                div { class: "web-page web-commanders content-enter",
                    div { class: "web-columns",
                        MaybeboardPanel { filter, filter_open }
                        SuggestedPanel { filter }
                    }
                }
            }
            // Filter, the screen's action, floats at the bottom right as the
            // dock's funnel, its dot on while a filter is set.
            ActionDock {
                DockFilter {
                    label: "Filter",
                    dot: filter::shows_dot(&filter.read()),
                    onclick: move |()| filter_open.set(true),
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::CommanderMaybeboard, open: hint_open }
        CardFilterPanel {
            open: filter_open,
            filter,
            config: SheetConfig::default(),
            quiet_reset: true,
            on_apply: move |applied: CardQueryBuilder| filter.set(applied),
        }
    }
}
