//! The Activity screen: how the user picks, read from their swipes. A
//! heatmap of the last year, what their swipes came to, their rates by card
//! role and oracle tag, and the cards they keep passing on. Reached from
//! the Home counters.

/// The screen's accordion sections and hard passes group.
pub(crate) mod sections;

use components::{
    ActionBar, Button, ButtonVariant, HintBullet, HintBullets, HintDialog, HintKey, HintLine,
};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::user::models::hints::HINT_ACTIVITY;
use zwipe_core::http::contracts::suggestions::HttpActivity;

use self::sections::{ACTIVITY_TITLE, ActivitySections, HardPassesPanel, HeatGhost};
use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::hint_dialog::use_one_time_hint;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{Screen, screen};
use crate::inbound::screens::deck::card::components::card_row::{
    OtagDescribe, OtagExamplesOpen, ShowRowArt,
};
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::inbound::screens::deck::card::components::printing_sheet::PrintingSheet;
use crate::inbound::screens::oracle_tag_examples::OracleTagExamples;

/// Whether the activity has anything to draw: a day, an outcome, a rate or a
/// pass.
fn has_anything(activity: &HttpActivity) -> bool {
    let o = &activity.outcomes;
    !activity.days.is_empty()
        || o.added > 0
        || o.skipped > 0
        || o.maybed > 0
        || o.removed > 0
        || !activity.roles.is_empty()
        || !activity.tags.is_empty()
        || !activity.passes.is_empty()
}

/// The screen while its first fetch is out, in the deck view's skeleton
/// grammar: the Activity section open on its loaded shape, the year's grid
/// as empty squares ([`HeatGhost`]), its three tags, a rule, then Totals as
/// its title as a plain ghost chip over four bars and their labels; Card
/// roles and Oracle tags collapsed to their headers; then the Most skipped
/// group as Playstyle's list ghosts it, its header a bar and its rows a
/// thumb and a bar.
#[component]
fn ActivitySkeleton() -> Element {
    rsx! {
        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
            div { class: "skeleton-stats",
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
                for i in 0..2 {
                    div { key: "{i}", class: "skeleton-chart-box",
                        div { class: "skeleton-bar skeleton-section-label" }
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
    }
}

/// How the user picks: the Activity, Card roles and Oracle tags sections in
/// one accordion with Activity open on entry, then their hard passes as card
/// rows. A failed first fetch says so; an account with nothing swiped yet
/// gets one line.
#[component]
pub fn Activity() -> Element {
    let navigator = use_navigator();
    let auth_client: Signal<ZwipeClient> = use_context();
    let authed = use_authed(Screen::Activity);
    // Auto-opens once per account, reopenable from the header "?".
    let hint_open = use_one_time_hint(HINT_ACTIVITY);
    // Art-crop thumbnails on the hard pass rows, as on Staples.
    let show_row_art = use_signal(|| true);
    use_context_provider(|| ShowRowArt(show_row_art));

    // Oracle-tag descriptions and Examples, wired as on Playstyle.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        cache.ensure_oracle_tags(auth_client);
    });
    let describe_tag = use_callback(move |slug: String| {
        cache.oracle_tags.cell().read().loaded().and_then(|tags| {
            tags.iter()
                .find(|t| t.slug == slug)
                .and_then(|t| t.description.clone())
        })
    });
    let mut otag_examples_open = use_signal(|| false);
    let mut otag_examples_slug = use_signal(String::new);
    let open_examples = use_callback(move |slug: String| {
        otag_examples_slug.set(slug);
        otag_examples_open.set(true);
    });
    use_context_provider(|| OtagDescribe(describe_tag));
    use_context_provider(|| OtagExamplesOpen(open_examples));

    let expanded_card: Signal<Option<Uuid>> = use_signal(|| None);
    let preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing = use_signal(|| false);
    let mut printing_card: Signal<Option<Card>> = use_signal(|| None);
    let mut printing_open = use_signal(|| false);

    let mut activity: Signal<Option<HttpActivity>> = use_signal(|| None);
    let mut is_loading = use_signal(|| true);
    let mut load_failed = use_signal(|| false);
    use_effect(move || {
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
        div { class: "screen",
            ScreenHeader { title: "Activity", hint: hint_open }

            HintDialog {
                open: hint_open,
                title: "Activity",
                HintLine { "How you pick, read from your swipes" }
                HintBullets {
                    HintBullet { HintKey { "Activity" } " maps your swipes and searches over the last year, and what the cards dealt to your decks came to" }
                    HintBullet { HintKey { "Card roles" } " and " HintKey { "Oracle tags" } " show how often you add, skip, save or remove each one" }
                    HintBullet { HintKey { "Most skipped" } " lists cards you skip that other players usually keep" }
                }
                HintLine { "Each section's ? explains it" }
            }

            div { class: "screen-content",
                // Bounded like Playstyle: screen-content centers its children,
                // so an unbounded column would grow to content width.
                div {
                    class: "flex-col",
                    style: "max-width: 40rem; width: 100%; padding: 0 1rem;",
                    if is_loading() {
                        ActivitySkeleton {}
                    } else if load_failed() {
                        div { class: "message-empty",
                            span { class: "chip-note", "Could not load your activity" }
                        }
                    } else if let Some(found) = activity().filter(has_anything) {
                        div { style: "width:100%;display:flex;flex-direction:column;gap:var(--panel-gap);",
                            ActivitySections { activity: found.clone(), open_section }
                            HardPassesPanel {
                                passes: found.passes,
                                expanded_card,
                                preview_card,
                                preview_dismissing,
                                on_printing: move |c: Card| {
                                    printing_card.set(Some(c));
                                    printing_open.set(true);
                                },
                            }
                        }
                    } else {
                        div { class: "message-empty",
                            span { class: "chip-note", "Swipe some cards to see your activity" }
                        }
                    }
                }
            }

            ActionBar {
                Button {
                    variant: ButtonVariant::Action,
                    onclick: move |_| navigator.go_back(),
                    "Back"
                }
            }

            // Pick a printing: swap that pass's card in place and keep the
            // row expanded under its new scryfall id.
            if let Some(card) = printing_card() {
                PrintingSheet {
                    card,
                    open: printing_open,
                    host_screen: screen::ACTIVITY,
                    on_save: move |new_card: Card| {
                        let target = new_card.scryfall_data.oracle_id;
                        if target.is_some() {
                            let mut expanded = expanded_card;
                            expanded.set(Some(new_card.scryfall_data.id));
                            if let Some(found) = activity.write().as_mut()
                                && let Some(slot) = found
                                    .passes
                                    .iter_mut()
                                    .find(|p| p.card.scryfall_data.oracle_id == target)
                            {
                                slot.card = new_card;
                            }
                        }
                    },
                }
            }

            if otag_examples_open() {
                OracleTagExamples { open: otag_examples_open, slug: otag_examples_slug() }
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }
        }
    }
}
