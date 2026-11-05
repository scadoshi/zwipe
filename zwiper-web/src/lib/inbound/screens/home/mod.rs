//! Home: the hero, the Decks and Colors panels and the featured flavor card.
//!
//! What the screen knows and does is `zwipe_ui::screens::home`; this file
//! draws its state and performs its effects, keeping the root's caches
//! current as replies land. On a wide window the hero sits at the top of
//! zite's page, Decks and Colors share one column under it and the flavor
//! card the other. Below the breakpoint it is the app's stack. The flavor
//! card's `Buy` is a link per retailer, as zite's is, in place of the app's
//! dialog.

pub mod hero;

use std::sync::atomic::{AtomicBool, Ordering};

use chrono::{DateTime, Utc};
use components::{HomeColors, HomeDecks, ThemeConfig};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use dioxus_primitives::toast::{Toasts, use_toast};
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::user::models::settings::PipColor;
use zwipe_ui::generation::Ticket;
use zwipe_ui::home::Figures;
use zwipe_ui::screens::home::{
    self, Action, Age, Cached, CachedFigures, DeckTile, Destination, Effect, Flavor, FlavorCard,
    State,
};

use self::hero::HomeHeroScoped;
use crate::domain::counts::CachedCounts;
use crate::inbound::caches::{CachedDecks, CachedMyCounts, CachedPlayStyle};
use crate::inbound::cards::buy_links::CardBuyLinks;
use crate::inbound::cards::image_preview::ImagePreview;
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::chrome::hint_topic::HintTopic;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, open_and_record_hint};
use crate::inbound::chrome::toast::raise;
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::outbound::counts_store::CountsStore;
use crate::outbound::scope_store::ScopeStore;
use crate::outbound::session_store::SessionStore;

/// Whether this page load has said hello. Home mounts again on every return
/// to it; the hello shows once per page load, the verify-email nudge on every
/// visit while the email is unverified.
static GREETED: AtomicBool = AtomicBool::new(false);

/// What the root keeps for Home, so a return draws it at once.
#[derive(Clone, Copy)]
struct Caches {
    counts: Signal<Option<CachedCounts>>,
    my_counts: Signal<Option<CachedMyCounts>>,
    play_style: Signal<Option<CachedPlayStyle>>,
    flavor: Signal<Option<FlavorCard>>,
    decks: Signal<Option<CachedDecks>>,
}

impl Caches {
    /// What the caches hold for `session`'s user at `now`, for
    /// [`Action::Load`]. The scope is the one stored by the last visit.
    fn snapshot(self, session: Option<&Session>, now: DateTime<Utc>) -> Cached {
        let user = session.map(|s| s.user.id);
        let everybody = self.counts.peek().as_ref().map(|c| CachedFigures {
            figures: Figures::from(&c.metrics),
            age: if !c.is_live() {
                Age::Restored
            } else if c.is_stale(now) {
                Age::Stale
            } else {
                Age::Fresh
            },
        });
        let mine = self
            .my_counts
            .peek()
            .as_ref()
            .filter(|c| Some(c.user_id) == user)
            .map(|c| CachedFigures {
                figures: Figures::from(&c.metrics),
                age: if user.is_some_and(|id| c.is_fresh_for(id, now)) {
                    Age::Fresh
                } else {
                    Age::Stale
                },
            });
        let play_style = self
            .play_style
            .peek()
            .as_ref()
            .filter(|c| Some(c.user_id) == user)
            .map(|c| c.summary.clone());
        let decks = self
            .decks
            .peek()
            .as_ref()
            .filter(|c| Some(c.user_id) == user)
            .map(|c| c.decks.clone());
        Cached {
            user,
            scope: ScopeStore::browser().load(),
            everybody,
            mine,
            play_style,
            flavor: self.flavor.peek().clone(),
            decks,
            now,
        }
    }

    /// Whether the user's own counters held are theirs and under a minute
    /// old at `now`.
    fn mine_fresh(self, session: Option<&Session>, now: DateTime<Utc>) -> bool {
        let user = session.map(|s| s.user.id);
        self.my_counts
            .peek()
            .as_ref()
            .is_some_and(|c| user.is_some_and(|id| c.is_fresh_for(id, now)))
    }
}

/// Home's model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    theme: Signal<ThemeConfig>,
    pip_style: Signal<PipColor>,
    caches: Caches,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        match effect {
            Effect::FetchUser(ticket) => self.fetch_user(ticket),
            Effect::FetchSettings => self.fetch_settings(),
            Effect::FetchCounts(ticket) => self.fetch_counts(ticket),
            Effect::FetchMyCounts(ticket) => self.fetch_my_counts(ticket),
            Effect::FetchPlayStyle(ticket) => self.fetch_play_style(ticket),
            Effect::FetchDecks(ticket) => self.fetch_decks(ticket),
            Effect::FetchFlavor(ticket) => self.fetch_flavor(ticket),
            Effect::OfferHint(hint) => {
                open_and_record_hint(hint, self.session, self.client, self.hint_open);
            }
            Effect::SaveScope(next) => ScopeStore::browser().save(next),
            Effect::MarkGreeted => GREETED.store(true, Ordering::Relaxed),
            Effect::Navigate(destination) => {
                self.navigator.push(route(destination));
            }
            Effect::Toast(raised) => raise(self.toast, &raised),
        }
    }

    /// Refreshes the user in the session, then answers with the user or,
    /// when the refresh failed, the session's copy to greet instead.
    fn fetch_user(self, ticket: Ticket) {
        let Self {
            mut session,
            client,
            ..
        } = self;
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            let action = match http.fetch_user(&current).await {
                Ok(fresh) => {
                    let latest = session.peek().clone();
                    if let Some(mut latest) = latest {
                        latest.user = fresh.clone();
                        SessionStore::browser().infallible_save(&latest);
                        session.set(Some(latest));
                    }
                    Action::UserLoaded {
                        ticket,
                        user: Box::new(fresh),
                        greeted: GREETED.load(Ordering::Relaxed),
                    }
                }
                Err(e) => {
                    tracing::warn!("home user fetch failed: {e}");
                    let cached = session.peek().as_ref().map(|s| Box::new(s.user.clone()));
                    Action::UserFailed {
                        ticket,
                        cached,
                        greeted: GREETED.load(Ordering::Relaxed),
                    }
                }
            };
            self.dispatch(action);
        });
    }

    /// Applies fresh settings, so a theme changed on another device shows
    /// here.
    fn fetch_settings(self) {
        let Self {
            session,
            client,
            mut theme,
            mut pip_style,
            ..
        } = self;
        let Some(current) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let http = client.peek().clone();
            match http.fetch_settings(&current).await {
                Ok(settings) => {
                    theme.set(components::theme_from_settings(&settings));
                    pip_style.set(settings.pip_style);
                }
                Err(e) => tracing::warn!("settings fetch failed: {e}"),
            }
        });
    }

    /// Fetches everybody's counters into the cache and browser storage.
    fn fetch_counts(self, ticket: Ticket) {
        let client = self.client;
        let mut counts = self.caches.counts;
        spawn(async move {
            let http = client.peek().clone();
            match http.public_metrics().await {
                Ok(metrics) => {
                    CountsStore::browser().save(&metrics);
                    let figures = Figures::from(&metrics);
                    counts.set(Some(CachedCounts::fetched(metrics, Utc::now())));
                    self.dispatch(Action::CountsLoaded { ticket, figures });
                }
                Err(e) => tracing::warn!("public counters not refreshed: {e}"),
            }
        });
    }

    /// Fetches the user's own counters into the cache, reporting a failure
    /// without a toast.
    fn fetch_my_counts(self, ticket: Ticket) {
        let authed = self.authed;
        let mut my_counts = self.caches.my_counts;
        let Some(user_id) = self.session.peek().as_ref().map(|s| s.user.id) else {
            return;
        };
        spawn(async move {
            if let Some(metrics) = authed
                .run_quiet("my_metrics", |c, s| async move { c.my_metrics(&s).await })
                .await
            {
                let figures = Figures::from(&metrics);
                my_counts.set(Some(CachedMyCounts {
                    user_id,
                    metrics,
                    fetched_at: Utc::now(),
                }));
                self.dispatch(Action::MyCountsLoaded { ticket, figures });
            }
        });
    }

    /// Fetches the playstyle summary into the cache, reporting a failure
    /// without a toast.
    fn fetch_play_style(self, ticket: Ticket) {
        let authed = self.authed;
        let mut play_style = self.caches.play_style;
        let Some(user_id) = self.session.peek().as_ref().map(|s| s.user.id) else {
            return;
        };
        spawn(async move {
            let action = match authed
                .run_quiet("play_style_summary", |c, s| async move {
                    c.play_style_summary(&s).await
                })
                .await
            {
                Some(summary) => {
                    play_style.set(Some(CachedPlayStyle {
                        user_id,
                        summary: summary.clone(),
                    }));
                    Action::PlayStyleLoaded { ticket, summary }
                }
                None => Action::PlayStyleFailed { ticket },
            };
            self.dispatch(action);
        });
    }

    /// Fetches the user's decks into the cache, reporting a failure without
    /// a toast.
    fn fetch_decks(self, ticket: Ticket) {
        let authed = self.authed;
        let mut cache = self.caches.decks;
        let Some(user_id) = self.session.peek().as_ref().map(|s| s.user.id) else {
            return;
        };
        spawn(async move {
            let action = match authed
                .run_quiet("deck_profiles", |c, s| async move {
                    c.fetch_deck_profiles(&s).await
                })
                .await
            {
                Some(profiles) => {
                    let decks: Vec<DeckTile> = profiles.iter().map(DeckTile::from).collect();
                    cache.set(Some(CachedDecks {
                        user_id,
                        decks: decks.clone(),
                    }));
                    Action::DecksLoaded { ticket, decks }
                }
                None => Action::DecksFailed { ticket },
            };
            self.dispatch(action);
        });
    }

    /// Fetches the hour's featured flavor card into the cache. A failure
    /// keeps the last card.
    fn fetch_flavor(self, ticket: Ticket) {
        let client = self.client;
        let mut flavor = self.caches.flavor;
        spawn(async move {
            let http = client.peek().clone();
            let action = match http.featured_flavor().await {
                Ok(card) => {
                    let fetched = FlavorCard::fetched(card, Utc::now());
                    flavor.set(Some(fetched.clone()));
                    Action::FlavorLoaded {
                        ticket,
                        flavor: Box::new(fetched),
                    }
                }
                Err(e) => {
                    tracing::warn!("featured flavor fetch failed: {e}");
                    Action::FlavorFailed { ticket }
                }
            };
            self.dispatch(action);
        });
    }
}

/// The web's route for a place Home sends the user.
fn route(destination: Destination) -> Route {
    match destination {
        Destination::Decks => Route::DeckList {},
        Destination::Deck(deck_id) => Route::ViewDeck { deck_id },
        Destination::CreateDeck => Route::CreateDeck {},
        Destination::Activity => Route::Activity {},
        Destination::Playstyle => Route::Playstyle {},
    }
}

/// Home. Each mount refreshes the user (warning while the email is
/// unverified, greeting once per page load otherwise), the settings (theme
/// and pips), the counters, the decks, the playstyle summary and, once the
/// hour's card has expired, the featured flavor, as the model asks.
#[component]
pub fn Home() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let caches = Caches {
        counts: use_context(),
        my_counts: use_context(),
        play_style: use_context(),
        flavor: use_context(),
        decks: use_context(),
    };
    // Opening Home takes in what the caches hold, so the first render draws
    // it, and keeps what the model asks for to perform below.
    let mut opening = Vec::new();
    let model = use_signal(|| {
        let mut state = State::new();
        let cached = caches.snapshot(session.peek().as_ref(), Utc::now());
        opening = state.update(Action::Load(Box::new(cached)));
        state
    });
    let driver = Driver {
        model,
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(),
        session,
        client: use_context(),
        hint_open: use_signal(|| false),
        theme: use_context(),
        pip_style: use_context(),
        caches,
    };
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let mut preview: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);

    use_hook(move || {
        for effect in opening {
            driver.perform(effect);
        }
    });

    // The hour's card is checked again whenever the session refreshes, not
    // only on mount.
    use_effect(move || {
        let _ = session();
        driver.dispatch(Action::CheckFlavor { now: Utc::now() });
    });

    let state = driver.model.read();
    let figures = state.figures();
    let live = state.live();
    let scope = state.scope();
    let decks = state.decks();
    let play_style = state.play_style();
    let flavor = state.flavor();
    let card = state.flavor_card().map(|c| c.scryfall_data.clone());
    drop(state);

    rsx! {
        ScreenFrame { title: home::TITLE,
            div { class: "screen-content home-content web-home",
                div { class: "web-page web-home-page",
                    div { class: "web-hero",
                        HomeHeroScoped {
                            figures,
                            live,
                            scope,
                            ontoggle: move |()| {
                                let mine_fresh = caches.mine_fresh(session.peek().as_ref(), Utc::now());
                                driver.dispatch(Action::ToggleScope { mine_fresh });
                            },
                        }
                    }
                    div { class: "web-columns",
                        div { class: "web-column",
                            HomeDecks {
                                decks,
                                onopen: move |()| driver.dispatch(Action::Decks),
                                ondeck: move |id| driver.dispatch(Action::OpenDeck(id)),
                                oncreate: move |()| driver.dispatch(Action::CreateDeck),
                            }
                            HomeColors {
                                play_style,
                                onplaystyle: move |()| driver.dispatch(Action::Playstyle),
                                oninfo: move |()| hint.set(Some(HintTopic::HomeColors)),
                            }
                        }
                        div { class: "web-column",
                            FlavorPanel {
                                flavor,
                                card,
                                on_image: move |sd: ScryfallData| preview.set(Some((sd, 0))),
                            }
                        }
                    }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::FirstLogin, open: driver.hint_open }
        ImagePreview { card: preview }
    }
}

/// The featured flavor card as the model quotes it: its text, then its
/// name, its USD price when it has one, `Image`, and when priced the
/// `TCGplayer` and `Card Kingdom` links. `card` is the card behind the quote,
/// for the image preview and the links. A card without flavor text leaves
/// the panel empty; before any card it ghosts.
#[component]
fn FlavorPanel(
    flavor: Flavor,
    card: Option<ScryfallData>,
    on_image: EventHandler<ScryfallData>,
) -> Element {
    let body = match (flavor, card) {
        (Flavor::Quote(quote), Some(sd)) => {
            let sd_for_image = sd.clone();
            rsx! {
                div { class: "settings-list-header", style: "justify-content: center;",
                    span { class: "settings-list-title", {home::FEATURED_FLAVOR} }
                }
                div { class: "flavor-quote", style: "padding: 1rem;", "{quote.text} " }
                div { class: "flavor-buy-row",
                    span { class: "flavor-name", "{quote.name}" }
                    if let Some(price) = quote.price.as_ref() {
                        span { class: "flavor-price", "{price}" }
                    }
                    button {
                        class: "card-action-btn",
                        onclick: move |_| on_image.call(sd_for_image.clone()),
                        {home::IMAGE}
                    }
                    if quote.price.is_some() {
                        CardBuyLinks { name: sd.name.clone(), tcgplayer_id: sd.tcgplayer_id }
                    }
                }
            }
        }
        (Flavor::Blank | Flavor::Quote(_), _) => rsx! {},
        (Flavor::Loading, _) => rsx! {
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

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn every_destination_has_the_web_route() {
        assert_eq!(route(Destination::Decks), Route::DeckList {});
        let deck_id = Uuid::nil();
        assert_eq!(
            route(Destination::Deck(deck_id)),
            Route::ViewDeck { deck_id }
        );
        assert_eq!(route(Destination::CreateDeck), Route::CreateDeck {});
        assert_eq!(route(Destination::Activity), Route::Activity {});
        assert_eq!(route(Destination::Playstyle), Route::Playstyle {});
    }
}
