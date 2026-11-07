//! The mark in a panel with the three counters, on home and the auth
//! screens, and the user's own counters heading the Activity screen. The
//! auth screens show everyone's counts; home's scope button switches between
//! everyone's and the user's own.

use std::time::{Duration, Instant};

use chrono::{DateTime, Utc};
use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_client::call::Dated;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::logo;
use zwipe_core::http::contracts::metrics::{HttpLifetimeCounters, HttpPublicMetrics};
use zwipe_ui::home::{CountsTag, Figures, Scope};
use zwipe_ui::screens::home;

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::outbound::count_scope_store::CountScope;
use crate::outbound::counts_store::PersistCounts;

/// How long a fetched set of counters serves before the next screen asks
/// zerver again. Hopping between home and the auth screens reuses it.
const COUNTS_FRESH: Duration = Duration::from_secs(60);

/// The counters as last fetched, kept above the router (as
/// `Signal<Option<CachedCounts>>`) so every screen with the panel shares one
/// copy and one fetch per minute.
#[derive(Debug, Clone)]
pub struct CachedCounts {
    /// What zerver answered.
    pub metrics: HttpPublicMetrics,
    /// When it answered this launch; `None` for a copy restored from the last
    /// launch, which serves until a fetch replaces it.
    pub fetched_at: Option<Instant>,
    /// When the server's copy was made, which an edge cache can put well
    /// before `fetched_at`; `None` when it did not say.
    pub made_at: Option<DateTime<Utc>>,
}

impl CachedCounts {
    /// The copy saved by the last launch, if there is one.
    pub fn restored() -> Option<Self> {
        HttpPublicMetrics::infallible_load().map(|metrics| Self {
            metrics,
            fetched_at: None,
            made_at: None,
        })
    }

    /// Older than `COUNTS_FRESH`, or restored, so the next screen asks again.
    pub fn is_stale(&self) -> bool {
        self.fetched_at
            .is_none_or(|fetched_at| fetched_at.elapsed() >= COUNTS_FRESH)
    }

    /// Marks the copy stale, so the next screen asks again.
    pub fn expire(&mut self) {
        if let Some(fetched_at) = self.fetched_at.as_mut() {
            *fetched_at = backdated(*fetched_at);
        }
    }
}

/// The user's own counters as last fetched, kept above the router (as
/// `Signal<Option<CachedMyCounts>>`) beside [`CachedCounts`] on the same
/// minute. Never saved across launches.
#[derive(Debug, Clone)]
pub struct CachedMyCounts {
    /// The user the counts belong to; another user's copy is never shown.
    pub user_id: Uuid,
    /// What zerver answered.
    pub metrics: HttpLifetimeCounters,
    /// When it answered.
    pub fetched_at: Instant,
}

impl CachedMyCounts {
    /// Older than `COUNTS_FRESH`, so the next screen asks again.
    pub fn is_stale(&self) -> bool {
        self.fetched_at.elapsed() >= COUNTS_FRESH
    }

    /// Marks the copy stale, so the next screen asks again.
    pub fn expire(&mut self) {
        self.fetched_at = backdated(self.fetched_at);
    }
}

/// `fetched_at` moved back by `COUNTS_FRESH`, so it reads as stale. The
/// monotonic clock stops while the device sleeps, so the time elapsed since
/// a fetch undercounts a night asleep.
fn backdated(fetched_at: Instant) -> Instant {
    fetched_at.checked_sub(COUNTS_FRESH).unwrap_or(fetched_at)
}

/// Everyone's counters from the cache above the router, refetched when
/// stale, with their tag (`Live`, or "as of" an edge-cached copy's time)
/// once the copy came from this launch. A stale copy keeps showing while the
/// refetch is out and is only replaced on success.
fn use_public_counts() -> (Option<HttpPublicMetrics>, Option<String>) {
    let client: Signal<ZwipeClient> = use_context();
    let mut cache: Signal<Option<CachedCounts>> = use_context();
    use_effect(move || {
        if cache.peek().as_ref().is_some_and(|c| !c.is_stale()) {
            return;
        }
        spawn(async move {
            match client().fetch_public_metrics_dated().await {
                Ok(Dated {
                    value: metrics,
                    made_at,
                }) => {
                    metrics.infallible_save();
                    cache.set(Some(CachedCounts {
                        metrics,
                        fetched_at: Some(Instant::now()),
                        made_at,
                    }));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });
    let value = cache.read();
    let metrics = value.as_ref().map(|c| c.metrics.clone());
    // A tag only for an answer from this launch, not a restored copy.
    let tag = value
        .as_ref()
        .filter(|c| c.fetched_at.is_some())
        .map(|c| CountsTag::at(c.made_at, Utc::now()).label());
    (metrics, tag)
}

/// The signed-in user's counters from the cache above the router, fetched
/// while `scope` is [`CountScope::JustMe`] and the copy is missing, another
/// user's or older than `COUNTS_FRESH`. A failed fetch is only logged.
fn use_my_counts(scope: Signal<CountScope>, screen: Screen) -> Option<HttpLifetimeCounters> {
    let session: Signal<Option<Session>> = use_context();
    let mut cache: Signal<Option<CachedMyCounts>> = use_context();
    let authed = use_authed(screen);
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
        // A sum past i64 is no number to show, so it waits like a loading one.
        metrics.and_then(|m| {
            m.swipes_right
                .checked_add(m.swipes_left)?
                .checked_add(m.swipes_up)?
                .checked_add(m.swipes_down)
        }),
    )
}

/// The mark above the cards swiped, searches run and decks created across
/// everyone, with tags saying where they come from. Unauthenticated, for the
/// auth screens; the figures roll until they arrive. A failed ask keeps the
/// copy on screen, restored from the last launch if need be, and is only
/// logged: these numbers are decoration, not something the user asked for.
#[component]
pub fn HomeHero() -> Element {
    let (metrics, tag) = use_public_counts();
    rsx! {
        HeroPanel {
            figures: public_figures(metrics.as_ref()),
            roll_key: "all",
            source: rsx! {
                if let Some(tag) = tag {
                    span { key: "{tag}", class: "stat-chip", "{tag}" }
                }
                span { class: "stat-chip", "{CountScope::AllUsers.label()}" }
            },
        }
    }
}

/// Home's hero: [`HomeHero`] with an Activity button, and its "Everybody"
/// tag as a button after the "Live" tag that switches the figures to the
/// user's own counts ("Only me") and back. Home's model holds the scope,
/// the figures and whether they are live; every switch rolls the figures
/// to the other counts, rolling on while the user's are fetched the first
/// time, and fades the button's label in.
#[component]
pub fn HomeHeroScoped(
    figures: Figures,
    /// The tag before the scope button: `Live`, "as of" a time, or nothing.
    tag: Option<String>,
    scope: Scope,
    ontoggle: EventHandler,
    onactivity: EventHandler,
) -> Element {
    let label = scope.label();
    let figures: Vec<(Option<u64>, &'static str)> =
        figures.values().into_iter().zip(Figures::LABELS).collect();
    rsx! {
        HeroPanel {
            figures,
            roll_key: label,
            source: rsx! {
                if let Some(tag) = tag {
                    span { key: "{tag}", class: "stat-chip", "{tag}" }
                }
                button {
                    class: "card-action-btn hero-scope",
                    // Sized to its label, one monospace character a `ch`, so
                    // a switch eases between the two labels' widths.
                    style: "width:calc({label.chars().count()}ch + 1.2rem + 2px);",
                    onclick: move |_| ontoggle.call(()),
                    // Keyed, so each label mounts fresh and fades in.
                    for label in [label] {
                        span { key: "{label}", class: "hero-scope-label", "{label}" }
                    }
                }
                button {
                    class: "card-action-btn",
                    onclick: move |_| onactivity.call(()),
                    {home::ACTIVITY}
                }
            },
        }
    }
}

/// The Activity screen's header: the user's own counters in Home's strip,
/// without the mark, tagged "Live" once they come from this launch and
/// "Only me". Shares Home's cached copy, so coming from Home's "Only me"
/// has them at once. The figures keep rolling until `ready`, which the
/// screen sets when the rest of it lands, so the counts resolve together
/// with it; they are fetched meanwhile all the same.
#[component]
pub fn MyCountsPanel(ready: bool) -> Element {
    let scope = use_signal(|| CountScope::JustMe);
    let mine = use_my_counts(scope, Screen::Activity);
    let shown = mine.as_ref().filter(|_| ready);
    let live = shown.is_some();
    rsx! {
        div { class: "home-hero activity-counts",
            Panel {
                StatsStrip {
                    figures: my_figures(shown),
                    compact: true,
                    source: rsx! {
                        if live {
                            span { class: "stat-chip", "Live" }
                        }
                        span { class: "stat-chip", "{CountScope::JustMe.label()}" }
                    },
                }
            }
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
    fn my_cards_swiped_past_i64_waits_instead_of_overflowing() {
        let mine = HttpLifetimeCounters {
            user_id: Uuid::nil(),
            swipes_right: i64::MAX,
            swipes_left: 1,
            swipes_up: 0,
            swipes_down: 0,
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
                (None, "Cards swiped"),
            ]
        );
    }

    #[test]
    fn figures_wait_while_counts_load() {
        assert!(my_figures(None).iter().all(|(n, _)| n.is_none()));
        assert!(public_figures(None).iter().all(|(n, _)| n.is_none()));
    }
}
