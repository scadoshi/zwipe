//! The mark in a panel with the three counters, on home and the auth
//! screens. The auth screens show everyone's counts; home's scope button
//! switches between everyone's and the user's own.

use std::time::{Duration, Instant};

use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::logo;
use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::router::Router;
use crate::outbound::count_scope_store::CountScope;
use crate::outbound::counts_store::PersistCounts;

/// How long a fetched set of counters serves before the next screen asks
/// zerver again. Hopping between home and the auth screens reuses it.
const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// The counters as last fetched, kept above the router (as
/// `Signal<Option<CachedCounts>>`) so every screen with the panel shares one
/// copy and one fetch per minute.
#[derive(Clone)]
pub struct CachedCounts {
    /// What zerver answered.
    pub metrics: HttpPublicMetrics,
    /// When it answered this launch; `None` for a copy restored from the last
    /// launch, which serves until a fetch replaces it.
    pub fetched_at: Option<Instant>,
}

impl CachedCounts {
    /// The copy saved by the last launch, if there is one.
    pub fn restored() -> Option<Self> {
        HttpPublicMetrics::infallible_load().map(|metrics| Self {
            metrics,
            fetched_at: None,
        })
    }

    /// Older than `COUNTS_FRESH`, or restored, so the next screen asks again.
    pub fn is_stale(&self) -> bool {
        self.fetched_at
            .is_none_or(|fetched_at| fetched_at.elapsed() >= COUNTS_FRESH)
    }
}

/// The user's own counters as last fetched, kept above the router (as
/// `Signal<Option<CachedMyCounts>>`) beside [`CachedCounts`] on the same
/// minute. Never saved across launches.
#[derive(Clone)]
pub struct CachedMyCounts {
    /// The user the counts belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered.
    pub metrics: HttpLifetimeCounters,
    /// When it answered.
    pub fetched_at: Instant,
}

/// Everyone's counters from the cache above the router, refetched when
/// stale, with whether the copy came from this launch. A stale copy keeps
/// showing while the refetch is out and is only replaced on success.
fn use_public_counts() -> (Option<HttpPublicMetrics>, bool) {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    use_effect(move || {
        if cache.peek().as_ref().is_some_and(|c| !c.is_stale()) {
            return;
        }
        spawn(async move {
            match client().public_metrics().await {
                Ok(metrics) => {
                    metrics.infallible_save();
                    cache.set(Some(CachedCounts {
                        metrics,
                        fetched_at: Some(Instant::now()),
                    }));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });
    let value = cache.read();
    let metrics = value.as_ref().map(|c| c.metrics.clone());
    // "Live" only for an answer from this launch, not a restored copy.
    let live = value.as_ref().is_some_and(|c| c.fetched_at.is_some());
    (metrics, live)
}

/// The signed-in user's counters from the cache above the router, fetched
/// while `scope` is [`CountScope::JustMe`] and the copy is missing, another
/// user's or older than `COUNTS_FRESH`. A failed fetch is only logged.
fn use_my_counts(scope: Signal<CountScope>) -> Option<HttpLifetimeCounters> {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedMyCounts>> = use_context();
    let authed = use_authed(Screen::Home);
    let user_id = session.peek().as_ref().map(|s| s.user.id);
    use_effect(move || {
        if scope() != CountScope::JustMe {
            return;
        }
        let Some(user_id) = user_id else {
            return;
        };
        let fresh = cache
            .peek()
            .as_ref()
            .is_some_and(|c| c.user_id == user_id && c.fetched_at.elapsed() < COUNTS_FRESH);
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
                    fetched_at: Instant::now(),
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

/// The three figures in the strip's order, `None` while a count is out.
fn figures(
    decks: Option<i64>,
    searches: Option<i64>,
    swiped: Option<i64>,
) -> Vec<(Option<u64>, &'static str)> {
    let count = |n: Option<i64>| n.map(|n| u64::try_from(n).unwrap_or(0));
    vec![
        (count(decks), "Decks created"),
        (count(searches), "Searches run"),
        (count(swiped), "Cards swiped"),
    ]
}

/// Everyone's figures, `None` each while they load.
fn public_figures(metrics: Option<&HttpPublicMetrics>) -> Vec<(Option<u64>, &'static str)> {
    figures(
        metrics.map(|m| m.decks_created),
        metrics.map(|m| m.searches),
        metrics.map(|m| m.cards_swiped),
    )
}

/// The user's own figures, cards swiped being every direction together.
fn my_figures(metrics: Option<&HttpLifetimeCounters>) -> Vec<(Option<u64>, &'static str)> {
    figures(
        metrics.map(|m| i64::from(m.decks_created)),
        metrics.map(|m| m.searches),
        metrics.map(|m| m.swipes_right + m.swipes_left + m.swipes_up + m.swipes_down),
    )
}

/// The mark above the cards swiped, searches run and decks created across
/// everyone, with tags saying where they come from. Unauthenticated, for the
/// auth screens; the figures roll until they arrive. A failed ask keeps the
/// copy on screen, restored from the last launch if need be, and is only
/// logged: these numbers are decoration, not something the user asked for.
#[component]
pub fn HomeHero() -> Element {
    let (metrics, live) = use_public_counts();
    rsx! {
        HeroPanel {
            figures: public_figures(metrics.as_ref()),
            roll_key: "all",
            source: rsx! {
                span { class: "stat-chip", "{CountScope::AllUsers.label()}" }
                if live {
                    span { class: "stat-chip", "Live" }
                }
            },
        }
    }
}

/// Home's hero: [`HomeHero`] with an Activity button opening the Activity
/// screen, and its "All players" tag as a button after the "Live" tag that
/// switches the figures to the user's own counts ("Only me") and back. The
/// choice is saved, so Home opens on the last one. Every switch rolls the
/// figures to the other counts, rolling on while the user's are fetched the
/// first time, and fades the button's label in; "Live" shows once the
/// figures on screen come from this launch.
#[component]
pub fn HomeHeroScoped() -> Element {
    let mut scope: Signal<CountScope> = use_context();
    let navigator = use_navigator();
    let (public, public_live) = use_public_counts();
    let mine = use_my_counts(scope);
    let (figures, live) = match scope() {
        CountScope::AllUsers => (public_figures(public.as_ref()), public_live),
        CountScope::JustMe => (my_figures(mine.as_ref()), mine.is_some()),
    };
    let label = scope().label();
    rsx! {
        HeroPanel {
            figures,
            roll_key: label,
            source: rsx! {
                if live {
                    span { class: "stat-chip", "Live" }
                }
                button {
                    class: "card-action-btn hero-scope",
                    onclick: move |_| {
                        let next = scope.peek().toggled();
                        next.infallible_save();
                        scope.set(next);
                    },
                    // Keyed, so each label mounts fresh and fades in.
                    for label in [label] {
                        span { key: "{label}", class: "hero-scope-label", "{label}" }
                    }
                }
                button {
                    class: "card-action-btn",
                    onclick: move |_| {
                        navigator.push(Router::Activity {});
                    },
                    "Activity"
                }
            },
        }
    }
}

/// The panel both heroes draw: the mark over the strip, with `source`
/// under it. A new `roll_key` mounts the strip afresh, so its figures roll
/// to the new counts; a counter that has landed does not roll again on a
/// changed value by itself.
#[component]
fn HeroPanel(
    figures: Vec<(Option<u64>, &'static str)>,
    roll_key: &'static str,
    source: Element,
) -> Element {
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    // Keyed, since a key only takes effect on a list item.
                    for roll_key in [roll_key] {
                        StatsStrip {
                            key: "{roll_key}",
                            figures: figures.clone(),
                            compact: true,
                            source: source.clone(),
                        }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::Utc;

    use super::*;

    #[test]
    fn my_cards_swiped_counts_every_direction() {
        let mine = HttpLifetimeCounters {
            user_id: Uuid::nil(),
            swipes_right: 1,
            swipes_left: 2,
            swipes_up: 3,
            swipes_down: 4,
            searches: 5,
            decks_created: 6,
            decks_completed: 0,
            updated_at: Utc::now(),
        };
        assert_eq!(
            my_figures(Some(&mine)),
            vec![
                (Some(6), "Decks created"),
                (Some(5), "Searches run"),
                (Some(10), "Cards swiped"),
            ]
        );
    }

    #[test]
    fn figures_wait_while_counts_load() {
        assert!(my_figures(None).iter().all(|(n, _)| n.is_none()));
        assert!(public_figures(None).iter().all(|(n, _)| n.is_none()));
    }
}
