//! Activity: how the user picks, read from their swipes. The year's
//! heatmap, what their swipes came to, their rates by card role and oracle
//! tag, and the cards they keep passing on. Reached from Home's counters.
//!
//! On a wide window the counts are the hero at the top of zite's page, the
//! Activity section spans the page under them so the year's heatmap reads
//! at size, and under it Most skipped takes the left column, Card roles and
//! Oracle tags the right, every section open. Below the
//! breakpoint it is the app's accordion with Activity open.

pub mod sections;

use components::{HoverStackHost, use_hover_stack};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::http::contracts::suggestions::HttpActivity;
use zwipe_ui::activity::has_anything;
use zwipe_ui::telemetry::vocabulary::Screen;

use self::sections::{ACTIVITY_TITLE, ActivityRates, ActivityYear, HardPassesPanel, HeatGhost};
use crate::inbound::cards::card_row::{CardOverlaysHost, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::cards::row_keys::use_card_row_keys;
use crate::inbound::cards::stack_view::{use_cards_view, use_row_faces, use_stack_details};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::ghost::{
    GhostChipBars, GhostChipRow, GhostHint, GhostSection, GhostText,
};
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::screens::home::hero::MyCountsPanel;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;
use crate::outbound::cards_view_store::CardsViewPlace;

/// The Card roles chart's ghost chips, as wide as the roles' short labels.
const ROLE_CHIPS: [u8; 6] = [4, 4, 5, 5, 5, 5];
/// The Oracle tags chart's ghost chips, as wide as typical tag slugs.
const TAG_CHIPS: [u8; 6] = [6, 9, 12, 8, 15, 7];

/// Activity while it loads, in the loaded screen's layout: the Activity
/// section (the heatmap, its three tags, the Totals chart) at the page's
/// width, then Most skipped beside Card roles and Oracle tags, each with
/// its `Show` row and bars. On a wide window every section is open, below
/// it only Activity.
#[component]
fn ActivitySkeleton() -> Element {
    telemetry::use_telemetry_screen(Screen::Activity);
    rsx! {
        div { class: "web-activity-year",
            GhostSection { chars: 8, open: true,
                div { class: "web-activity-year-body",
                    div { class: "web-activity-heat",
                        HeatGhost {}
                        div { class: "heat-readout",
                            div { class: "heat-readout-row",
                                for i in 0..3 {
                                    div { key: "{i}", class: "skeleton-bar skeleton-chip activity-ghost-tag" }
                                }
                            }
                        }
                    }
                    hr { class: "card-row-rule card-row-rule-muted activity-rule" }
                    div { class: "activity-outcomes activity-ghost-chart",
                        span { class: "chart-label",
                            span { class: "stat-chip chart-label-chip web-ghost-chip", GhostText { chars: 6 } }
                        }
                        div { class: "activity-ghost-bars",
                            for (i, pct) in [70, 100, 20, 35].into_iter().enumerate() {
                                div { key: "{i}", class: "activity-ghost-column",
                                    div { class: "skeleton-bar activity-ghost-bar", style: "height:{pct}%;" }
                                }
                            }
                        }
                        div { class: "activity-ghost-labels",
                            for (i, chars) in [5, 7, 10, 7].into_iter().enumerate() {
                                span { key: "{i}", class: "web-activity-ghost-label", GhostText { chars } }
                            }
                        }
                    }
                }
            }
        }
        div { class: "web-columns",
            div { class: "web-column",
                div { class: "maybeboard-list card-group row-enter",
                    div { class: "card-group-header card-group-header-with-hint",
                        span { GhostText { chars: 12 } }
                        GhostHint {}
                    }
                    for i in 0..10 {
                        div { key: "{i}", class: "skeleton-card-row web-ghost-card-row",
                            div { class: "skeleton-bar skeleton-card-thumb" }
                            div { class: "skeleton-bar skeleton-card-bar-row" }
                        }
                    }
                }
            }
            div { class: "web-column",
                div { class: "web-activity-rates",
                    for (i, (chars, chips)) in [(10, ROLE_CHIPS), (11, TAG_CHIPS)].into_iter().enumerate() {
                        GhostSection { key: "{i}", chars, open: false,
                            div { class: "activity-rates",
                                GhostChipRow {
                                    label: 4,
                                    chips: vec![5, 7, 10, 7],
                                    class: "activity-rate-chips",
                                }
                                GhostChipBars { chips: chips.to_vec(), suffixed: true }
                            }
                        }
                    }
                }
            }
        }
    }
}

/// Activity. The fetch runs once per visit with the skeleton up; a failure
/// toasts and says so, and an account with nothing swiped gets one line.
#[component]
pub fn Activity() -> Element {
    let authed = use_authed();
    let hint_open = use_one_time_hint(OneTimeHint::Activity);
    let mut hint_again = hint_open;
    let show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let hover_stack = use_hover_stack();
    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let view = use_cards_view(CardsViewPlace::MostSkipped);
    use_row_faces();

    let mut activity: Signal<Option<HttpActivity>> = use_signal(|| None);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    use_hook(move || {
        spawn(async move {
            match authed
                .try_run("load_activity", |c, s| async move { c.activity(&s).await })
                .await
            {
                Ok(found) => activity.set(Some(found)),
                Err(_) => load_failed.set(true),
            }
            is_loading.set(false);
        });
    });
    let open_section: Signal<Option<String>> = use_signal(|| Some(ACTIVITY_TITLE.to_string()));
    // Most skipped's cards, which the card keys and the stack act on.
    let pass_cards = use_memo(move || {
        activity
            .read()
            .as_ref()
            .map(|found| {
                found
                    .passes
                    .iter()
                    .map(|p| p.card.clone())
                    .collect::<Vec<_>>()
            })
            .unwrap_or_default()
    });
    let pass_ids = use_memo(move || {
        pass_cards
            .read()
            .iter()
            .map(|c| c.scryfall_data.id)
            .collect::<Vec<_>>()
    });
    use_stack_details(pass_ids);
    use_card_row_keys(pass_cards, expanded_card);

    rsx! {
        ScreenFrame {
            title: "Activity",
            on_hint: move |()| hint_again.set(true),
            div {
                class: "screen-content",
                onmousemove: move |e| hover_stack.track(e.client_coordinates().x),
                div { class: "flex-col web-activity web-page web-sections-open",
                    div { class: "web-hero",
                        MyCountsPanel { ready: !is_loading() }
                    }
                    if is_loading() {
                        ActivitySkeleton {}
                    } else if load_failed() {
                        div { class: "message-empty",
                            span { class: "chip-note", "Couldn't load your activity" }
                        }
                    } else if let Some(found) = activity().filter(has_anything) {
                        ActivityYear { activity: found.clone(), open_section }
                        div { class: "web-columns",
                            div { class: "web-column",
                                HardPassesPanel { passes: found.passes.clone(), expanded_card, view }
                            }
                            div { class: "web-column",
                                ActivityRates { activity: found, open_section }
                            }
                        }
                    } else {
                        div { class: "message-empty",
                            span { class: "chip-note", "Swipe some cards to see your activity" }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Activity, open: hint_open }
        OtagExamplesHost { examples }
        HoverStackHost { stack: hover_stack }
        CardOverlaysHost { overlays }
    }
}
