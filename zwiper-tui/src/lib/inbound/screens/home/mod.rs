//! The home screen: the logo and the counters, the color map, and the featured flavor card, each drawn from the caches kept above the screens.

pub(crate) mod cache;
pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::{Duration, Instant};

use chrono::Utc;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::models::hints::HINT_FIRST_LOGIN;
use zwipe_core::domain::user::preferences::UserPreferences;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;

use self::cache::{CachedMyCounts, CachedPlayStyle, FlavorCard};
use super::Screen;
use crate::domain::color_map::seed_from_id;
use crate::domain::decode;
use crate::domain::home::{self, Figures, Scope};
use crate::inbound::app::{Context, Shared};
use crate::inbound::nav::Nav;
use crate::inbound::screens::{self, oracle_tags};
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::toast::ToastKind;
use crate::inbound::widgets::{color_map, entrance, figures, hint, logo, stagger};
use crate::outbound::account::Account;
use crate::outbound::home_file::HomeFile;

/// A popup over home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Overlay {
    /// The color map's "?".
    Info(InfoPopup),
    /// The featured card's Buy dialog.
    Buy(AlertDialog),
}

/// The home screen's state. The figures, map and card live in [`cache::Caches`]; this holds what moves on screen.
#[derive(Debug)]
pub(crate) struct HomeView {
    /// When the logo started decoding in. `None` draws it settled.
    pub(crate) logo_started: Option<Instant>,
    /// When the strip appeared, which drives the waiting roll.
    pub(crate) figures_opened: Instant,
    /// When the strip's figures arrived, which drives the landing roll.
    pub(crate) figures_arrived: Option<Instant>,
    /// When the color map first drew, which drives its sweep.
    pub(crate) map_started: Option<Instant>,
    /// When the screen opened, which drives the loading ghosts.
    pub(crate) opened: Instant,
    /// The color map failed to load this visit.
    pub(crate) play_style_failed: bool,
    /// A color map summary is on screen, cached or fetched.
    pub(crate) play_style_seen: bool,
    /// When the scope last changed, which drives its label's `hero-scope-in`.
    pub(crate) scope_at: Option<Instant>,
    pub(crate) overlay: Option<Overlay>,
}

impl Default for HomeView {
    /// A settled home: no entrance left to play.
    fn default() -> Self {
        let long_ago = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .unwrap_or_else(Instant::now);
        Self {
            logo_started: None,
            figures_opened: long_ago,
            figures_arrived: Some(long_ago),
            map_started: Some(long_ago),
            opened: long_ago,
            play_style_failed: false,
            play_style_seen: true,
            scope_at: None,
            overlay: None,
        }
    }
}

impl HomeView {
    /// A home screen whose entrance plays from now, with figures already cached rolling to land.
    pub(crate) fn opening(shared: &Shared) -> Self {
        let now = Instant::now();
        let mut view = Self {
            logo_started: Some(now),
            figures_opened: now,
            figures_arrived: None,
            map_started: None,
            opened: now,
            play_style_seen: false,
            ..Self::default()
        };
        if figures_for(shared).is_some() {
            view.figures_arrived = Some(now);
        }
        if play_style_for(shared).is_some() {
            view.play_style_seen = true;
            view.map_started = Some(now);
        }
        view
    }

    /// How far the logo's entrance has run; settled when it never started.
    pub(crate) fn logo_elapsed(&self) -> Duration {
        self.logo_started.map_or(decode::DURATION, |t| t.elapsed())
    }

    /// The time since the figures arrived, zero while they are still out.
    pub(crate) fn since_arrival(&self) -> Duration {
        self.figures_arrived.map_or(Duration::ZERO, |t| t.elapsed())
    }

    /// The time since the map first drew.
    pub(crate) fn map_elapsed(&self) -> Duration {
        self.map_started.map_or(Duration::ZERO, |t| t.elapsed())
    }

    /// Whether something on home is moving: the logo, a rolling figure, the map's sweep or a loading ghost.
    pub(crate) fn animating(&self) -> bool {
        let loading_map = !self.play_style_seen && !self.play_style_failed;
        logo::animating(self.logo_elapsed())
            || self.figures_arrived.is_none()
            || self.since_arrival() < figures::ROLL
            || self
                .map_started
                .is_some_and(|_| self.map_elapsed() < color_map::entrance())
            || loading_map
            || stagger::entering(self.opened.elapsed(), stagger::DELAY, entrance::DURATION)
            || self
                .scope_at
                .is_some_and(|t| stagger::entering(t.elapsed(), Duration::ZERO, stagger::SCOPE_IN))
    }

    /// Marks the figures arrived now if they just turned up.
    fn note_figures(&mut self, shared: &Shared) {
        if self.figures_arrived.is_none() && figures_for(shared).is_some() {
            self.figures_arrived = Some(Instant::now());
        }
    }

    /// Remounts home the way the app does on every return: the entrance plays again and every panel refreshes.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        *self = Self::opening(cx.shared);
        mount(cx);
    }

    /// Switches whose counters show, remembers the choice, and rolls the strip again.
    pub(crate) fn toggle_scope(&mut self, cx: &mut Context<'_>) {
        let scope = cx.shared.caches.scope.toggled();
        cx.shared.caches.scope = scope;
        cx.request(Request::SaveScope(scope).into());
        self.figures_opened = Instant::now();
        self.scope_at = Some(Instant::now());
        self.figures_arrived = figures_for(cx.shared).map(|_| Instant::now());
        fetch_my_counts_if_stale(cx);
    }
}

/// The signed-in user's id.
pub(crate) fn user_id(shared: &Shared) -> Option<Uuid> {
    shared.user.as_ref().map(|u| u.id)
}

/// The figures for the scope on screen: everybody's, or the signed-in user's own.
pub(crate) fn figures_for(shared: &Shared) -> Option<Figures> {
    match shared.caches.scope {
        Scope::Everybody => shared.counts.metrics.as_ref().map(Figures::from),
        Scope::OnlyMe => shared.caches.my_figures(user_id(shared)),
    }
}

/// Whether the figures on screen count as live: everybody's fetched this launch, or any copy of the user's own.
pub(crate) fn live(shared: &Shared) -> bool {
    match shared.caches.scope {
        Scope::Everybody => shared.counts.live(),
        Scope::OnlyMe => shared.caches.my_figures(user_id(shared)).is_some(),
    }
}

/// The signed-in user's color map summary, if cached.
pub(crate) fn play_style_for(shared: &Shared) -> Option<&HttpPlayStyleSummary> {
    shared.caches.play_style_of(user_id(shared))
}

/// The color map's seed for the signed-in user, the same one Playstyle draws with.
pub(crate) fn seed(shared: &Shared) -> u64 {
    user_id(shared).map_or(0, |id| seed_from_id(id.as_u128()))
}

/// The home screen's work.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum Request {
    /// Reads the scope the last launch saved.
    Restore,
    /// Saves the scope for the next launch.
    SaveScope(Scope),
    /// The user as the server has it now.
    User,
    Preferences,
    /// The user's own counters.
    MyMetrics,
    /// The color map.
    PlayStyle,
    /// This hour's featured flavor card.
    Flavor,
}

/// The home screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Restored(Scope),
    User(Box<User>),
    UserFailed,
    Preferences(Box<UserPreferences>),
    MyMetrics {
        user_id: Uuid,
        figures: Figures,
    },
    PlayStyle {
        user_id: Uuid,
        summary: Box<HttpPlayStyleSummary>,
    },
    PlayStyleFailed,
    Flavor(Box<FlavorCard>),
}

impl From<Request> for screens::Request {
    fn from(request: Request) -> Self {
        Self::Home(request)
    }
}

impl From<Update> for screens::Update {
    fn from(update: Update) -> Self {
        Self::Home(update)
    }
}

fn pick(screen: &mut Screen) -> Option<&mut HomeView> {
    match screen {
        Screen::Home(view) => Some(view),
        _ => None,
    }
}

/// Starts over from home: its entrance plays, its panels refresh, and the welcome hint shows the first time.
pub(crate) fn open(cx: &mut Context<'_>) {
    cx.reset(Screen::Home(Box::new(HomeView::opening(cx.shared))));
    mount(cx);
}

/// What the app does on every mount of home.
fn mount(cx: &mut Context<'_>) {
    let restore = !cx.shared.caches.restore_asked;
    cx.shared.caches.restore_asked = true;
    let counts_stale = cx.shared.counts.stale(Instant::now());
    let flavor_expired = cx.shared.caches.flavor_expired(Utc::now());
    if restore {
        cx.request(Request::Restore.into());
    }
    cx.request(Request::User.into());
    cx.request(Request::Preferences.into());
    if counts_stale {
        cx.request(screens::Request::FetchCounts);
    }
    fetch_my_counts_if_stale(cx);
    cx.request(Request::PlayStyle.into());
    if flavor_expired {
        cx.request(Request::Flavor.into());
    }
    oracle_tags::ensure_catalog(cx);
    hint::once(cx, HINT_FIRST_LOGIN);
}

/// Asks for the user's own counters when home shows them and the cached copy is missing, another user's or a minute old.
fn fetch_my_counts_if_stale(cx: &mut Context<'_>) {
    let Some(user) = user_id(cx.shared) else {
        return;
    };
    if cx.shared.caches.scope == Scope::OnlyMe
        && cx.shared.caches.my_counts_stale(user, Instant::now())
    {
        cx.request(Request::MyMetrics.into());
    }
}

pub(crate) async fn perform(account: &Account, request: Request) -> Option<screens::Update> {
    let client = &account.client;
    let update = match request {
        Request::Restore => Update::Restored(HomeFile::in_config_dir().ok()?.load_scope()),
        Request::SaveScope(scope) => {
            // A failed write only means the next launch opens on everybody.
            HomeFile::in_config_dir()
                .and_then(|f| f.save_scope(scope))
                .ok();
            return None;
        }
        Request::User => match async { client.fetch_user(&account.fresh().await?).await }.await {
            Ok(user) => {
                account.set_user(user.clone()).await;
                Update::User(Box::new(user))
            }
            Err(e) => return Some(screens::failure(&e, |_| Update::UserFailed.into())),
        },
        Request::Preferences => {
            let preferences = async { client.fetch_preferences(&account.fresh().await?).await }
                .await
                .ok()?;
            Update::Preferences(Box::new(preferences))
        }
        Request::MyMetrics => {
            let session = account.fresh().await.ok()?;
            let counters = client.my_metrics(&session).await.ok()?;
            Update::MyMetrics {
                user_id: session.user.id,
                figures: Figures::from(&counters),
            }
        }
        Request::PlayStyle => {
            let result = async {
                let session = account.fresh().await?;
                let summary = client.play_style_summary(&session).await?;
                Ok((session.user.id, summary))
            }
            .await;
            match result {
                Ok((user_id, summary)) => Update::PlayStyle {
                    user_id,
                    summary: Box::new(summary),
                },
                Err(e) => return Some(screens::failure(&e, |_| Update::PlayStyleFailed.into())),
            }
        }
        Request::Flavor => {
            let card: Card = client.featured_flavor().await.ok()?;
            Update::Flavor(Box::new(FlavorCard {
                card,
                expires_at: home::next_hour(Utc::now()),
            }))
        }
    };
    Some(update.into())
}

/// Says hello once per launch.
fn greet(cx: &mut Context<'_>, username: &str) {
    if !cx.shared.caches.greeted {
        cx.shared.caches.greeted = true;
        cx.toast(ToastKind::Info, format!("Hello, {username}!"));
    }
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::Restored(scope) => {
            if cx.shared.caches.scope != scope {
                cx.shared.caches.scope = scope;
                if let Some(view) = nav.find_mut(pick) {
                    view.figures_opened = Instant::now();
                    view.scope_at = Some(Instant::now());
                    view.figures_arrived = None;
                }
                fetch_my_counts_if_stale(cx);
            }
        }
        Update::User(user) => {
            let username = user.username.to_string();
            let verified = user.email_verified_at.is_some();
            cx.shared.username = Some(username.clone());
            cx.shared.user = Some(*user);
            if verified {
                greet(cx, &username);
            } else {
                cx.toast(ToastKind::Error, "Verify your email!".to_owned());
            }
        }
        Update::UserFailed => {
            let cached = cx
                .shared
                .user
                .as_ref()
                .filter(|u| u.email_verified_at.is_some())
                .map(|u| u.username.to_string());
            if let Some(username) = cached {
                greet(cx, &username);
            }
        }
        Update::Preferences(preferences) => cx.use_preferences(&preferences),
        Update::MyMetrics { user_id, figures } => {
            cx.shared.caches.my_counts = Some(CachedMyCounts {
                user_id,
                figures,
                fetched_at: Instant::now(),
            });
        }
        Update::PlayStyle { user_id, summary } => {
            cx.shared.caches.play_style = Some(CachedPlayStyle {
                user_id,
                summary: *summary,
            });
            if let Some(view) = nav.find_mut(pick) {
                view.play_style_failed = false;
                view.play_style_seen = true;
                view.map_started.get_or_insert_with(Instant::now);
            }
        }
        Update::PlayStyleFailed => {
            if let Some(view) = nav.find_mut(pick) {
                view.play_style_failed = true;
            }
        }
        Update::Flavor(flavor) => cx.shared.caches.flavor = Some(*flavor),
    }
    if let Some(view) = nav.find_mut(pick) {
        view.note_figures(cx.shared);
    }
}

/// Lands everybody's counts on home when they arrive, if home is open.
pub(crate) fn counts_arrived(nav: &mut Nav, shared: &Shared) {
    if let Some(view) = nav.find_mut(pick) {
        view.note_figures(shared);
    }
}

/// How long the strip has waited for its figures, for drawing.
pub(crate) fn since_open(view: &HomeView) -> Duration {
    view.figures_opened.elapsed()
}
