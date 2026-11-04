//! Home's hero, whose scope chip switches between everybody's counters and
//! the user's own, and Activity's header of the user's own counters, which
//! fetches them itself.

use chrono::Utc;
use components::{Panel, StatsStrip};
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::metrics::HttpLifetimeCounters;
use zwipe_ui::home::{Figures, Scope};
use zwipe_ui::screens::home;

use crate::inbound::caches::CachedMyCounts;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::screens::auth::hero::{HeroPanel, strip_figures};

/// The signed-in user's counters from the root's copy, fetched quietly while
/// `scope` is `Only me` and the copy is missing, another user's or a minute
/// old. Another user's copy is never returned.
pub fn use_my_counts(scope: Signal<Scope>) -> Option<HttpLifetimeCounters> {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedMyCounts>> = use_context();
    let authed = use_authed();
    let user_id = session.peek().as_ref().map(|s| s.user.id);
    use_effect(move || {
        if scope() != Scope::OnlyMe {
            return;
        }
        let Some(user_id) = user_id else {
            return;
        };
        let fresh = cache
            .peek()
            .as_ref()
            .is_some_and(|c| c.is_fresh_for(user_id, Utc::now()));
        if fresh {
            return;
        }
        spawn(async move {
            if let Some(metrics) = authed
                .run_quiet("my_metrics", |c, s| async move { c.my_metrics(&s).await })
                .await
            {
                cache.set(Some(CachedMyCounts {
                    user_id,
                    metrics,
                    fetched_at: Utc::now(),
                }));
            }
        });
    });
    cache
        .read()
        .as_ref()
        .filter(|c| Some(c.user_id) == user_id)
        .map(|c| c.metrics.clone())
}

/// Home's hero: the mark over everybody's or the user's own three
/// `figures`, tagged `Live` once the figures on screen come from this visit,
/// then the scope button, which calls `ontoggle`. A new scope rolls the
/// figures to the other counts while its label fades in. Home's model holds
/// the figures and the scope.
#[component]
pub fn HomeHeroScoped(
    figures: Figures,
    live: bool,
    scope: Scope,
    ontoggle: EventHandler,
) -> Element {
    let label = scope.label();
    rsx! {
        HeroPanel {
            figures: strip_figures(Some(figures)),
            roll_key: label,
            source: rsx! {
                if live {
                    span { class: "stat-chip", {home::LIVE} }
                }
                button {
                    class: "card-action-btn hero-scope",
                    style: "width:calc({label.chars().count()}ch + 1.2rem + 2px);",
                    onclick: move |_| ontoggle.call(()),
                    for label in [label] {
                        span { key: "{label}", class: "hero-scope-label", "{label}" }
                    }
                }
            },
        }
    }
}

/// Activity's header: the user's own counters in Home's strip without the
/// mark, tagged `Live` once shown and `Only me`. The figures keep rolling
/// until `ready`, so they land with the rest of the screen; they are fetched
/// meanwhile all the same.
#[component]
pub fn MyCountsPanel(ready: bool) -> Element {
    let scope = use_signal(|| Scope::OnlyMe);
    let mine = use_my_counts(scope);
    let shown = mine.as_ref().filter(|_| ready);
    let live = shown.is_some();
    rsx! {
        div { class: "home-hero activity-counts",
            Panel {
                StatsStrip {
                    figures: strip_figures(shown.map(Figures::from)),
                    compact: true,
                    source: rsx! {
                        if live {
                            span { class: "stat-chip", "Live" }
                        }
                        span { class: "stat-chip", "{Scope::OnlyMe.label()}" }
                    },
                }
            }
        }
    }
}
