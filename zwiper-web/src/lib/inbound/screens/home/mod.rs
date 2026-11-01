//! Home: the counters, the color map and the featured flavor card.
//!
//! On a wide window the hero sits at the top of zite's page and the color
//! map and flavor panels share the two columns under it. Below the
//! breakpoint it is the app's stack. The app's buttons that only navigate
//! (`Decks`, `Account`, `Config`, `Activity`, `Playstyle`) go, since the
//! nav holds them. The flavor card's `Buy` is a link per retailer, as
//! zite's is, in place of the app's dialog.

pub mod color_map;
pub mod hero;

use std::sync::atomic::{AtomicBool, Ordering};

use chrono::Utc;
use components::{TOAST_QUICK, ThemeConfig};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::deck_metrics;
use zwipe_core::domain::user::models::preferences::PipStyle;

use self::color_map::HomePlayStyle;
use self::hero::HomeHeroScoped;
use crate::inbound::caches::FlavorCard;
use crate::inbound::cards::buy_links::CardBuyLinks;
use crate::inbound::cards::image_preview::ImagePreview;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::shell::ScreenFrame;
use crate::outbound::session_store::SessionStore;

/// Whether this visit has greeted the user. Home mounts again on every
/// return to it; the greeting shows once per page load.
static GREETED: AtomicBool = AtomicBool::new(false);

/// Home. Each mount refreshes the user (warning while the email is
/// unverified, greeting once per visit otherwise), the preferences (theme
/// and pips) and, once the hour's card has expired, the featured flavor.
#[component]
pub fn Home() -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let mut session: Signal<Option<Session>> = use_context();
    let toast = use_toast();
    let hint_open = use_one_time_hint(OneTimeHint::FirstLogin);
    let mut theme: Signal<ThemeConfig> = use_context();
    let mut pip_style: Signal<PipStyle> = use_context();
    let mut preview: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let mut flavor: Signal<Option<FlavorCard>> = use_context();

    // The user, for a current verification state, then the greeting.
    use_hook(move || {
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_user(&current).await {
                Ok(fresh) => {
                    let unverified = fresh.email_verified_at.is_none();
                    let username = fresh.username.to_string();
                    let latest = session.peek().clone();
                    if let Some(mut latest) = latest {
                        latest.user = fresh;
                        SessionStore::browser().infallible_save(&latest);
                        session.set(Some(latest));
                    }
                    if unverified {
                        toast.warning(
                            "Verify your email!".to_string(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    } else if !GREETED.swap(true, Ordering::Relaxed) {
                        toast.info(
                            format!("Hello, {username}!"),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                }
                Err(e) => {
                    tracing::warn!("home user fetch failed: {e}");
                    let cached = session.peek().clone();
                    if let Some(cached) = cached
                        && cached.user.email_verified_at.is_some()
                        && !GREETED.swap(true, Ordering::Relaxed)
                    {
                        toast.info(
                            format!("Hello, {}!", cached.user.username),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                    }
                }
            }
        });
    });

    // The preferences, so a theme changed on another device shows here.
    use_hook(move || {
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_preferences(&current).await {
                Ok(prefs) => {
                    theme.set(components::theme_from_preferences(&prefs));
                    pip_style.set(prefs.pip_style);
                }
                Err(e) => tracing::warn!("preferences fetch failed: {e}"),
            }
        });
    });

    // The hour's card: fetched when there is none or it expired, checked
    // again whenever the session refreshes. A failure keeps the last card.
    use_effect(move || {
        let _ = session();
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

    rsx! {
        ScreenFrame { title: "Home",
            div { class: "screen-content home-content web-home",
                div { class: "web-page web-home-page",
                    div { class: "web-hero", HomeHeroScoped {} }
                    div { class: "web-columns",
                        div { class: "web-column", HomePlayStyle {} }
                        div { class: "web-column",
                            FlavorPanel {
                                flavor: flavor(),
                                on_image: move |sd: ScryfallData| preview.set(Some((sd, 0))),
                            }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::FirstLogin, open: hint_open }
        ImagePreview { card: preview }
    }
}

/// The featured flavor card: its quote, then its name, its USD price when it
/// has one, `Image`, and when priced the `TCGplayer` and `Card Kingdom`
/// links. A card without flavor text leaves the panel empty; before any card
/// it ghosts.
#[component]
fn FlavorPanel(flavor: Option<FlavorCard>, on_image: EventHandler<ScryfallData>) -> Element {
    let body = match flavor {
        Some(entry) => {
            let sd = entry.card.scryfall_data;
            let Some(flavor_text) = sd.flavor_text.clone() else {
                return rsx! {
                    div { class: "container-sm text-center flex-col home-flavor content-enter-delayed" }
                };
            };
            let price = deck_metrics::card_price(&sd, PriceCurrency::Usd)
                .map(|p| PriceCurrency::Usd.format_amount(p));
            let sd_for_image = sd.clone();
            rsx! {
                div { class: "settings-list-header", style: "justify-content: center;",
                    span { class: "settings-list-title", "Featured flavor" }
                }
                div { class: "flavor-quote", style: "padding: 1rem;", "{flavor_text} " }
                div { class: "flavor-buy-row",
                    span { class: "flavor-name", "{sd.name}" }
                    if let Some(price) = price.as_ref() {
                        span { class: "flavor-price", "{price}" }
                    }
                    button {
                        class: "card-action-btn",
                        onclick: move |_| on_image.call(sd_for_image.clone()),
                        "Image"
                    }
                    if price.is_some() {
                        CardBuyLinks { name: sd.name.clone(), tcgplayer_id: sd.tcgplayer_id }
                    }
                }
            }
        }
        None => rsx! {
            div { class: "skeleton-card-header", style: "display: flex; justify-content: center;",
                div { class: "skeleton-bar skeleton-section-label", style: "width: 45%; margin-bottom: 0;" }
            }
            div { class: "skeleton-flavor", style: "padding: 1rem;",
                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-1" }
                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-2" }
                div { class: "skeleton-bar skeleton-flavor-line skeleton-flavor-line-3" }
            }
            div { class: "flavor-buy-row",
                div { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                for i in 0..3 {
                    div { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                }
            }
        },
    };
    rsx! {
        div { class: "container-sm text-center flex-col home-flavor content-enter-delayed", {body} }
    }
}
