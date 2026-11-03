//! Home/landing page screen.
//!
//! What the screen knows and does is `zwipe_ui::screens::home`; this file
//! draws its state and performs its effects, keeping the caches above the
//! router current as replies land.

use std::sync::atomic::{AtomicBool, Ordering};
use std::time::Instant;

use chrono::Utc;
use components::{
    AlertDialog, DialogAction, HintBullet, HintBullets, HintDialog, HintKey, ThemeConfig, Toasts,
    use_toast,
};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::user::models::hints::HINT_FIRST_LOGIN;
use zwipe_core::domain::user::preferences::PipColor;
use zwipe_ui::generation::Ticket;
use zwipe_ui::home::{Figures, Scope};
use zwipe_ui::screens::home::{
    self, Action, Age, Cached, CachedFigures, Destination, Effect, Flavor, State,
};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::auth::session_upkeep::FlavorCard;
use crate::inbound::components::hint_dialog::{self, HintColored};
use crate::inbound::components::home_hero::{CachedCounts, CachedMyCounts, HomeHeroScoped};
use crate::inbound::components::home_play_style::{CachedPlayStyle, HomePlayStyle};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::Screen;
use crate::inbound::components::{guide_link, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::image_preview::ImagePreview;
use crate::outbound::app_icon;
use crate::outbound::count_scope_store::CountScope;
use crate::outbound::counts_store::PersistCounts;

/// Whether this launch has already greeted. Home remounts on every navigation
/// back to it, so the greeting needs somewhere to remember itself that
/// outlives the component.
static GREETED: AtomicBool = AtomicBool::new(false);

/// What Home keeps above the router, so a return draws it at once.
#[derive(Clone, Copy)]
struct Caches {
    counts: Signal<Option<CachedCounts>>,
    my_counts: Signal<Option<CachedMyCounts>>,
    play_style: Signal<Option<CachedPlayStyle>>,
    flavor: Signal<Option<FlavorCard>>,
    scope: Signal<CountScope>,
}

impl Caches {
    /// What the caches hold for `session`'s user now, for [`Action::Load`].
    fn snapshot(self, session: Option<&Session>) -> Cached {
        let user = session.map(|s| s.user.id);
        let everybody = self.counts.peek().as_ref().map(|c| CachedFigures {
            figures: Figures::from(&c.metrics),
            age: if c.fetched_at.is_none() {
                Age::Restored
            } else if c.is_stale() {
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
                age: if c.is_stale() { Age::Stale } else { Age::Fresh },
            });
        let play_style = self
            .play_style
            .peek()
            .as_ref()
            .filter(|c| Some(c.user_id) == user)
            .map(|c| c.summary.clone());
        let flavor = self.flavor.peek().as_ref().map(|f| home::FlavorCard {
            card: f.card.clone(),
            expires_at: f.expires_at,
        });
        Cached {
            user,
            scope: scope_of(*self.scope.peek()),
            everybody,
            mine,
            play_style,
            flavor,
            now: Utc::now(),
        }
    }

    /// Whether the user's own counters held are theirs and under a minute
    /// old.
    fn mine_fresh(self, session: Option<&Session>) -> bool {
        let user = session.map(|s| s.user.id);
        self.my_counts
            .peek()
            .as_ref()
            .is_some_and(|c| Some(c.user_id) == user && !c.is_stale())
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
    theme_config: Signal<ThemeConfig>,
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
            Effect::FetchPreferences => self.fetch_preferences(),
            Effect::FetchCounts(ticket) => self.fetch_counts(ticket),
            Effect::FetchMyCounts(ticket) => self.fetch_my_counts(ticket),
            Effect::FetchPlayStyle(ticket) => self.fetch_play_style(ticket),
            Effect::FetchFlavor(ticket) => self.fetch_flavor(ticket),
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::SaveScope(next) => {
                let next = count_scope(next);
                next.infallible_save();
                let mut scope = self.caches.scope;
                scope.set(next);
            }
            Effect::MarkGreeted => GREETED.store(true, Ordering::Relaxed),
            Effect::Navigate(destination) => {
                self.navigator.push(route(destination));
            }
            Effect::Toast(raised) => ui_toast::raise(self.toast, raised),
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
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            let action = match client().fetch_user(&s).await {
                Ok(fresh_user) => {
                    let current = session.peek().clone();
                    if let Some(mut current) = current {
                        current.user = fresh_user.clone();
                        session.set(Some(current));
                    }
                    Action::UserLoaded {
                        ticket,
                        user: Box::new(fresh_user),
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

    /// Applies fresh preferences, so the theme stays current mid-session.
    fn fetch_preferences(self) {
        let Self {
            session,
            client,
            mut theme_config,
            mut pip_style,
            ..
        } = self;
        let Some(s) = session.peek().clone() else {
            return;
        };
        spawn(async move {
            match client().fetch_preferences(&s).await {
                Ok(prefs) => {
                    let theme = components::theme_from_preferences(&prefs);
                    app_icon::follow(&theme);
                    theme_config.set(theme);
                    pip_style.set(prefs.pip_style);
                }
                Err(e) => {
                    tracing::warn!("preferences fetch failed: {e}");
                }
            }
        });
    }

    /// Fetches everybody's counters into the cache and the saved copy.
    fn fetch_counts(self, ticket: Ticket) {
        let client = self.client;
        let mut counts = self.caches.counts;
        spawn(async move {
            match client().public_metrics().await {
                Ok(metrics) => {
                    metrics.infallible_save();
                    let figures = Figures::from(&metrics);
                    counts.set(Some(CachedCounts {
                        metrics,
                        fetched_at: Some(Instant::now()),
                    }));
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
                    fetched_at: Instant::now(),
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

    /// Fetches the featured flavor card: ONE shared server pick per UTC hour
    /// (unauthed endpoint; everyone sees the same card at the same moment),
    /// cached until the top of the next hour and overwritten only on
    /// success, so a failed refetch keeps the last card instead of blanking.
    fn fetch_flavor(self, ticket: Ticket) {
        let client = self.client;
        let mut flavor = self.caches.flavor;
        spawn(async move {
            let action = match client().featured_flavor().await {
                Ok(card) => {
                    let entry = FlavorCard::new(card);
                    let fetched = home::FlavorCard {
                        card: entry.card.clone(),
                        expires_at: entry.expires_at,
                    };
                    flavor.set(Some(entry));
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

/// The app's route for a place Home sends the user.
fn route(destination: Destination) -> Router {
    match destination {
        Destination::Decks => Router::DeckList {},
        Destination::Account => Router::Account {},
        Destination::Config => Router::Config {},
        Destination::Activity => Router::Activity {},
        Destination::Playstyle => Router::SuggestedCommanders {},
    }
}

/// The model's scope for the app's stored one.
fn scope_of(scope: CountScope) -> Scope {
    match scope {
        CountScope::AllUsers => Scope::Everybody,
        CountScope::JustMe => Scope::OnlyMe,
    }
}

/// The app's stored scope for the model's.
fn count_scope(scope: Scope) -> CountScope {
    match scope {
        Scope::Everybody => CountScope::AllUsers,
        Scope::OnlyMe => CountScope::JustMe,
    }
}

/// Home screen with navigation to main app features.
#[component]
pub fn Home() -> Element {
    let session: Signal<Option<Session>> = use_context();
    let caches = Caches {
        counts: use_context(),
        my_counts: use_context(),
        play_style: use_context(),
        flavor: use_context(),
        scope: use_context(),
    };
    // Opening Home takes in what the caches hold, so a return draws it on
    // the first frame, and keeps what the model asks for to perform below.
    let mut opening = Vec::new();
    let model = use_signal(|| {
        let mut state = State::new();
        let cached = caches.snapshot(session.peek().as_ref());
        opening = state.update(Action::Load(Box::new(cached)));
        state
    });
    let driver = Driver {
        model,
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Home),
        session,
        client: use_context(),
        hint_open: use_signal(|| false),
        theme_config: use_context(),
        pip_style: use_context(),
        caches,
    };
    let first_login_hint_open = driver.hint_open;

    // Refreshes the user (then greets with `home::greeting`), the
    // preferences, the counters, the summary and the flavor card as the
    // model asks, and offers the first-login hint, once per mount.
    use_hook(move || {
        for effect in opening {
            driver.perform(effect);
        }
    });

    // Re-checks the flavor card's expiry each time the session refreshes
    // (~60s), not only on mount; the fetch itself does not need the session.
    use_effect(move || {
        let _ = session();
        driver.dispatch(Action::CheckFlavor { now: Utc::now() });
    });

    // The flavor card's Image button previews that card's image.
    let mut preview_card: Signal<Option<(ScryfallData, usize)>> = use_signal(|| None);
    let preview_dismissing: Signal<bool> = use_signal(|| false);
    // The flavor card's Buy button opens its buy sheet (TCGplayer / CK).
    let mut show_buy_sheet = use_signal(|| false);

    // Snapshot the model once per render.
    let state = driver.model.read();
    let figures = state.figures();
    let live = state.live();
    let scope = state.scope();
    let play_style = state.play_style();
    let seed = state.seed();
    let flavor = state.flavor();
    let flavor_data = state.flavor_card().map(|c| c.scryfall_data.clone());
    let buy_links = state.buy_links();
    drop(state);

    // The Buy-card dialog: Close, then the retailers for the flavor card.
    let buy_actions: Vec<DialogAction> = std::iter::once(DialogAction::Dismiss(home::CLOSE.into()))
        .chain(buy_links.into_iter().map(|link| DialogAction::Link {
            label: link.label.into(),
            href: link.url,
        }))
        .collect();

    rsx! {
            div { class: "screen",
                ScreenHeader { title: home::TITLE }
                div { class: "screen-content home-content",
                HomeHeroScoped {
                    figures,
                    live,
                    scope,
                    ontoggle: move |()| {
                        let mine_fresh = caches.mine_fresh(session.peek().as_ref());
                        driver.dispatch(Action::ToggleScope { mine_fresh });
                    },
                    onactivity: move |()| driver.dispatch(Action::Activity),
                }

                HomePlayStyle {
                    play_style,
                    seed,
                    onplaystyle: move |()| driver.dispatch(Action::Playstyle),
                }

                // Display random flavor text
                div { class: "container-sm text-center flex-col home-flavor content-enter-delayed",
                    match flavor {
                        Flavor::Quote(quote) => {
                            rsx! {
                                div { class: "settings-list-header", style: "justify-content: center;",
                                    span { class: "settings-list-title", {home::FEATURED_FLAVOR} }
                                }
                                div { class: "flavor-quote", style: "padding: 1rem;",
                                    "{quote.text} "
                                }
                                div { class: "flavor-buy-row",
                                    span { class: "flavor-name", "{quote.name}" }
                                    if let Some(price) = quote.price.as_ref() {
                                        span { class: "flavor-price", "{price}" }
                                    }
                                    button {
                                        class: "card-action-btn",
                                        onclick: move |_| {
                                            if let Some(sd) = flavor_data.clone() {
                                                preview_card.set(Some((sd, 0)));
                                            }
                                        },
                                        {home::IMAGE}
                                    }
                                    if quote.price.is_some() {
                                        button {
                                            class: "card-action-btn",
                                            onclick: move |_| show_buy_sheet.set(true),
                                            {home::BUY}
                                        }
                                    }
                                }
                            }
                        }
                        Flavor::Blank => rsx! {},
                        Flavor::Loading => rsx! {
                            // Mirror the loaded card: a "Featured flavor" title
                            // bar + divider, the quote lines, then a chip
                            // ghost for each tag and button on the row under
                            // them.
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
                    }
                }
            }
            HintDialog {
                open: first_login_hint_open,
                actions: guide_link::one_time(HINT_FIRST_LOGIN),
                title: "Welcome to Zwipe",
                HintBullets {
                    HintBullet { HintKey { "Decks" } " holds your decks" }
                    HintBullet { HintKey { "Color map" } " grows from your decks' colors. Tap Playstyle for suggested commanders" }
                    HintBullet { HintKey { "Account" } " has your email, password and log out" }
                    HintBullet { HintKey { "Settings" } " sets your theme and which cards show" }
                    HintBullet { HintColored { color: "--color-warning", "Verify your email" } " in Account for full deck and card limits" }
                }
            }

            ImagePreview { card: preview_card, dismissing: preview_dismissing }


            AlertDialog {
                open: show_buy_sheet,
                title: home::BUY_TITLE,
                actions: buy_actions,
                {home::BUY_BODY}
            }
            }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_destination_has_the_app_route() {
        assert_eq!(route(Destination::Decks), Router::DeckList {});
        assert_eq!(route(Destination::Account), Router::Account {});
        assert_eq!(route(Destination::Config), Router::Config {});
        assert_eq!(route(Destination::Activity), Router::Activity {});
        assert_eq!(
            route(Destination::Playstyle),
            Router::SuggestedCommanders {}
        );
    }

    #[test]
    fn the_scope_round_trips_the_apps_stored_one() {
        for scope in [CountScope::AllUsers, CountScope::JustMe] {
            assert_eq!(count_scope(scope_of(scope)), scope);
            assert_eq!(scope_of(scope).label(), scope.label());
        }
    }
}
