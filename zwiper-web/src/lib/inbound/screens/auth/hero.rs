//! The mark over three counters: the panel Home's scoped hero draws in,
//! and everybody's counts on the public screens, with the hour's featured
//! flavor card beside their forms.

use chrono::Utc;
use components::{Decode, Panel, StatsStrip};
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::logo;
use zwipe_ui::home::{Figures, Scope};
use zwipe_ui::screens::home::{self, Flavor, FlavorCard};

use crate::domain::counts::CachedCounts;
use crate::inbound::cards::image_preview::ImagePreview;
use crate::inbound::screens::home::FlavorPanel;
use crate::outbound::counts_store::CountsStore;

/// The strip's three figures with their labels, `None` each while loading.
pub fn strip_figures(figures: Option<Figures>) -> Vec<(Option<u64>, &'static str)> {
    let values = figures.unwrap_or_default().values();
    values.into_iter().zip(Figures::LABELS).collect()
}

/// The public screens' hero: the mark over decks created, searches run and
/// cards swiped across everyone from the root's copy, tagged `Everybody`
/// and, once the copy comes from this visit, `Live`. A restored or stale
/// copy is fetched again on mount and stored for the next visit; a failed
/// fetch leaves the copy on screen.
#[component]
pub fn PublicHero() -> Element {
    let mut counts: Signal<Option<CachedCounts>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    use_effect(move || {
        let stale = counts
            .peek()
            .as_ref()
            .is_none_or(|c| c.is_stale(Utc::now()));
        if !stale {
            return;
        }
        spawn(async move {
            let http = client.peek().clone();
            match http.public_metrics().await {
                Ok(metrics) => {
                    CountsStore::browser().save(&metrics);
                    counts.set(Some(CachedCounts::fetched(metrics, Utc::now())));
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    });
    let (figures, live) = {
        let copy = counts.read();
        (
            copy.as_ref().map(|c| Figures::from(&c.metrics)),
            copy.as_ref().is_some_and(CachedCounts::is_live),
        )
    };
    rsx! {
        div { class: "web-hero",
            HeroPanel {
                figures: strip_figures(figures),
                roll_key: "all",
                source: rsx! {
                    if live {
                        span { class: "stat-chip", {home::LIVE} }
                    }
                    span { class: "stat-chip", "{Scope::Everybody.label()}" }
                },
            }
        }
    }
}

/// The public screens' featured flavor card: Home's panel over the root's
/// copy, fetched on mount when none is held or the hour's pick expired. A
/// failed fetch leaves the panel ghosting. `Image` opens the preview.
#[component]
pub fn PublicFlavor() -> Element {
    let mut flavor: Signal<Option<FlavorCard>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let mut preview: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    use_effect(move || {
        let stale = flavor
            .peek()
            .as_ref()
            .is_none_or(|f| f.is_expired(Utc::now()));
        if !stale {
            return;
        }
        spawn(async move {
            let http = client.peek().clone();
            match http.featured_flavor().await {
                Ok(card) => flavor.set(Some(FlavorCard::fetched(card, Utc::now()))),
                Err(e) => tracing::warn!("featured flavor fetch failed: {e}"),
            }
        });
    });
    let (quote, card) = {
        let held = flavor.read();
        (
            held.as_ref()
                .map_or(Flavor::Loading, |f| home::quote(&f.card)),
            held.as_ref().map(|f| f.card.scryfall_data.clone()),
        )
    };
    rsx! {
        div { class: "web-auth-flavor",
            FlavorPanel {
                flavor: quote,
                card,
                on_image: move |sd: ScryfallData| preview.set(Some((sd, 0))),
            }
        }
        ImagePreview { card: preview }
    }
}

/// The panel both heroes draw: the mark over the strip, with `source`
/// under it, then zite's tagline under a full-bleed rule. A new `roll_key`
/// mounts the strip afresh, so its figures roll to the new counts.
#[component]
pub fn HeroPanel(
    figures: Vec<(Option<u64>, &'static str)>,
    roll_key: &'static str,
    source: Element,
) -> Element {
    rsx! {
        div { class: "home-hero",
            Panel {
                div { class: "hero-head",
                    div { class: "logo", Decode { text: logo::ZWIPE } }
                    for roll_key in [roll_key] {
                        StatsStrip {
                            key: "{roll_key}",
                            figures: figures.clone(),
                            compact: true,
                            source: source.clone(),
                        }
                    }
                }
                hr { class: "panel-rule" }
                p { class: "web-hero-tagline",
                    "The "
                    a {
                        href: "https://magic.wizards.com/en",
                        "Magic: The Gathering"
                    }
                    " deck builder. Swipe through cards for any format, and pick a commander to get the best synergy first."
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::http::contracts::metrics::HttpPublicMetrics;

    use super::*;

    #[test]
    fn figures_wait_while_loading_and_keep_the_app_order() {
        assert!(strip_figures(None).iter().all(|(n, _)| n.is_none()));
        let metrics = HttpPublicMetrics {
            cards_swiped: 3,
            searches: 2,
            decks_created: 1,
        };
        assert_eq!(
            strip_figures(Some(Figures::from(&metrics))),
            vec![
                (Some(1), "Decks created"),
                (Some(2), "Searches run"),
                (Some(3), "Cards swiped"),
            ]
        );
    }
}
