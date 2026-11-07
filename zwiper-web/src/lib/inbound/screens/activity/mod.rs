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
use zwipe_core::domain::card::Card;
use zwipe_core::http::contracts::suggestions::HttpActivity;
use zwipe_ui::activity::has_anything;

use self::sections::{ACTIVITY_TITLE, ActivityRates, ActivityYear, HardPassesPanel, HeatGhost};
use crate::inbound::cards::card_row::{CardOverlaysHost, ShowRowArt, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::screens::home::hero::MyCountsPanel;
use crate::inbound::shell::ScreenFrame;

/// The screen while its fetch is out, in the loaded layout: the Activity
/// section open on its loaded shape (the grid's height as empty space,
/// three tags, a rule, the Totals title over four bars and labels) over the
/// Most skipped group as a header and ten rows, then Card roles and Oracle
/// tags as collapsed headers.
#[component]
fn ActivitySkeleton() -> Element {
    rsx! {
        div { class: "web-columns",
            div { class: "web-column",
                div { class: "skeleton-info-list",
                    div { class: "skeleton-card-header card-group-header-with-hint",
                        div { class: "skeleton-bar skeleton-section-label" }
                        div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                    }
                    div { class: "activity-ghost-body",
                        HeatGhost {}
                        div { class: "heat-readout",
                            div { class: "heat-readout-row",
                                for i in 0..3 {
                                    div { key: "{i}", class: "skeleton-bar skeleton-chip activity-ghost-tag" }
                                }
                            }
                        }
                        hr { class: "card-row-rule card-row-rule-muted activity-rule" }
                        div { class: "activity-outcomes activity-ghost-chart",
                            div { class: "skeleton-bar skeleton-chip activity-ghost-title" }
                            div { class: "activity-ghost-bars",
                                for (i, pct) in [70, 100, 20, 35].into_iter().enumerate() {
                                    div { key: "{i}", class: "activity-ghost-column",
                                        div { class: "skeleton-bar activity-ghost-bar", style: "height:{pct}%;" }
                                    }
                                }
                            }
                            div { class: "activity-ghost-labels",
                                for i in 0..4 {
                                    div { key: "{i}", class: "skeleton-bar activity-ghost-label" }
                                }
                            }
                        }
                    }
                }
                div { class: "maybeboard-list card-group row-enter",
                    div { class: "card-group-header card-group-header-with-hint",
                        div { class: "skeleton-bar skeleton-card-bar-header-wide" }
                        div { class: "skeleton-bar skeleton-chip skeleton-info-button" }
                    }
                    for i in 0..10 {
                        div { key: "{i}", class: "skeleton-card-row",
                            div { class: "skeleton-bar skeleton-card-thumb" }
                            div { class: "skeleton-bar skeleton-card-bar-row" }
                        }
                    }
                }
            }
            div { class: "web-column",
                for i in 0..2 {
                    div { key: "{i}", class: "skeleton-chart-box",
                        div { class: "skeleton-bar skeleton-section-label" }
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
                            span { class: "chip-note", "Could not load your activity" }
                        }
                    } else if let Some(found) = activity().filter(has_anything) {
                        ActivityYear { activity: found.clone(), open_section }
                        div { class: "web-columns",
                            div { class: "web-column",
                                HardPassesPanel { passes: found.passes.clone(), expanded_card }
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
        CardOverlaysHost {
            overlays,
            on_printing_save: move |picked: Card| {
                let target = picked.scryfall_data.oracle_id;
                if target.is_none() {
                    return;
                }
                let mut expanded = expanded_card;
                expanded.set(Some(picked.scryfall_data.id));
                if let Some(found) = activity.write().as_mut()
                    && let Some(slot) = found
                        .passes
                        .iter_mut()
                        .find(|p| p.card.scryfall_data.oracle_id == target)
                {
                    slot.card = picked;
                }
            },
        }
    }
}
