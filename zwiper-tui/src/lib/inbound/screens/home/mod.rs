//! The home screen: the logo and the counters, the color map, and the featured flavor card.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]: which figures show and whether they are live, what the Color map and Featured flavor panels draw, the greeting, and which fetches a visit needs. Home's data outlives a visit in [`cache::Caches`] and `shared.counts`: each opening hands what they hold to [`Action::Load`] as [`Cached`], and each reply is written there as well as handed to the model. This view adds the entrance and roll animations, the overlays, and the featured card's image.

pub(crate) mod cache;
pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::cell::RefCell;
use std::time::{Duration, Instant};

use chrono::Utc;
use image::DynamicImage;
use ratatui_image::picker::ProtocolType;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ImageSize;
use zwipe_core::domain::user::User;
use zwipe_core::domain::user::preferences::UserPreferences;
use zwipe_core::http::contracts::suggestions::HttpPlayStyleSummary;
use zwipe_ui::color_map::seed_from_id;
use zwipe_ui::generation::Ticket;
use zwipe_ui::home::{Figures, Scope};
use zwipe_ui::screens::home::{
    Action, Age, Cached, CachedFigures, Destination, Effect, FlavorCard, PlayStyle, State,
};

use self::cache::{CachedMyCounts, CachedPlayStyle, FlavorImage};
use super::Screen;
use super::visit::Visit;
use crate::domain::decode;
use crate::inbound::app::{Context, Shared};
use crate::inbound::nav::Nav;
use crate::inbound::screens::swipe::CardImage;
use crate::inbound::screens::{self, account, activity, config, decks, oracle_tags, playstyle};
use crate::inbound::widgets::alert::AlertDialog;
use crate::inbound::widgets::info::InfoPopup;
use crate::inbound::widgets::{color_map, entrance, figures, hint, logo, stagger};
use crate::outbound::account::Account;
use crate::outbound::home_file::HomeFile;
use crate::outbound::images;

/// A popup over home.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Overlay {
    /// The color map's "?".
    Info(InfoPopup),
    /// The featured card's Buy dialog.
    Buy(AlertDialog),
}

/// The home screen's state: the model plus what moves on screen.
#[derive(Debug)]
pub(crate) struct HomeView {
    pub(crate) model: State,
    pub(crate) visit: Visit,
    /// The ticket of the everybody's counts this visit asked for, which the app-wide fetch answers.
    pub(crate) counts_ticket: Option<Ticket>,
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
    /// When the scope last changed, which drives its label's `hero-scope-in`.
    pub(crate) scope_at: Option<Instant>,
    pub(crate) overlay: Option<Overlay>,
}

impl Default for HomeView {
    /// A settled home on a fresh model: no entrance left to play.
    fn default() -> Self {
        let long_ago = Instant::now()
            .checked_sub(Duration::from_secs(60))
            .unwrap_or_else(Instant::now);
        Self {
            model: State::new(),
            visit: Visit::next(),
            counts_ticket: None,
            logo_started: None,
            figures_opened: long_ago,
            figures_arrived: Some(long_ago),
            map_started: Some(long_ago),
            opened: long_ago,
            scope_at: None,
            overlay: None,
        }
    }
}

impl HomeView {
    /// Plays the entrance from now, as every mount of home does. Figures and a map the model already holds land at once.
    fn enter(&mut self) {
        let now = Instant::now();
        self.logo_started = Some(now);
        self.figures_opened = now;
        self.figures_arrived = None;
        self.map_started = None;
        self.opened = now;
        self.scope_at = None;
        self.overlay = None;
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

    /// The hero's figures, `None` while every one is still out.
    pub(crate) fn figures(&self) -> Option<Figures> {
        let figures = self.model.figures();
        figures
            .values()
            .iter()
            .any(Option::is_some)
            .then_some(figures)
    }

    /// Whether something on home is moving: the logo, a rolling figure, the map's sweep or a loading ghost.
    pub(crate) fn animating(&self) -> bool {
        let loading_map = self.model.play_style() == PlayStyle::Loading;
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

    /// Starts the strip's roll and the map's sweep the first time their data is on the model.
    fn settle(&mut self) {
        let now = Instant::now();
        if self.figures_arrived.is_none() && self.figures().is_some() {
            self.figures_arrived = Some(now);
        }
        if self.map_started.is_none() && matches!(self.model.play_style(), PlayStyle::Map(_)) {
            self.map_started = Some(now);
        }
    }

    /// Remounts home the way the app does on every return: the entrance plays again and the model asks for what is missing or old.
    pub(crate) fn refresh(&mut self, cx: &mut Context<'_>) {
        self.enter();
        mount(self, cx);
    }

    /// Queues `job` under this visit.
    fn request(&self, cx: &mut Context<'_>, job: Job) {
        cx.request(
            Request::Visit {
                visit: self.visit,
                job,
            }
            .into(),
        );
    }
}

/// The signed-in user's id.
pub(crate) fn user_id(shared: &Shared) -> Option<Uuid> {
    shared.user.as_ref().map(|u| u.id)
}

/// The color map's seed for the signed-in user, the same one Playstyle draws with.
pub(crate) fn seed(shared: &Shared) -> u64 {
    user_id(shared).map_or(0, |id| seed_from_id(id.as_u128()))
}

/// Whether the signed-in user's own counters held are theirs and under [`COUNTS_FRESH`](zwipe_ui::timing::COUNTS_FRESH) old.
fn mine_fresh(shared: &Shared) -> bool {
    user_id(shared).is_some_and(|user| !shared.caches.my_counts_stale(user, Instant::now()))
}

/// What the caches hold for the signed-in user now, for [`Action::Load`]. Everybody's counts are the copy Login's hero shares, under the same 60 second rule.
fn snapshot(shared: &Shared) -> Cached {
    let user = user_id(shared);
    let now = Instant::now();
    let everybody = shared.counts.metrics.as_ref().map(|metrics| CachedFigures {
        figures: Figures::from(metrics),
        age: if !shared.counts.live() {
            Age::Restored
        } else if shared.counts.stale(now) {
            Age::Stale
        } else {
            Age::Fresh
        },
    });
    let mine = user
        .and_then(|id| {
            shared
                .caches
                .my_figures(Some(id))
                .map(|figures| (id, figures))
        })
        .map(|(id, figures)| CachedFigures {
            figures,
            age: if shared.caches.my_counts_stale(id, now) {
                Age::Stale
            } else {
                Age::Fresh
            },
        });
    Cached {
        user,
        scope: shared.caches.scope,
        everybody,
        mine,
        play_style: shared.caches.play_style_of(user).cloned(),
        flavor: shared.caches.flavor.clone(),
        now: Utc::now(),
    }
}

/// Applies `action` to the model, performs the effects it returns in order, and starts any roll or sweep its data allows.
pub(crate) fn dispatch(view: &mut HomeView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        perform_effect(view, effect, cx);
    }
    view.settle();
}

fn perform_effect(view: &mut HomeView, effect: Effect, cx: &mut Context<'_>) {
    match effect {
        Effect::FetchUser(ticket) => view.request(cx, Job::User { ticket }),
        Effect::FetchPreferences => cx.request(Request::Preferences.into()),
        Effect::FetchCounts(ticket) => {
            view.counts_ticket = Some(ticket);
            cx.request(screens::Request::FetchCounts);
        }
        Effect::FetchMyCounts(ticket) => view.request(cx, Job::MyMetrics { ticket }),
        Effect::FetchPlayStyle(ticket) => view.request(cx, Job::PlayStyle { ticket }),
        Effect::FetchFlavor(ticket) => view.request(cx, Job::Flavor { ticket }),
        Effect::OfferHint(offer) => {
            hint::once(cx, offer.key());
        }
        Effect::SaveScope(scope) => {
            cx.shared.caches.scope = scope;
            cx.request(Request::SaveScope(scope).into());
            let now = Instant::now();
            view.figures_opened = now;
            view.scope_at = Some(now);
            view.figures_arrived = view.figures().map(|_| now);
        }
        Effect::MarkGreeted => cx.shared.caches.greeted = true,
        Effect::Navigate(Destination::Decks) => decks::open(cx),
        Effect::Navigate(Destination::Account) => account::open(cx),
        Effect::Navigate(Destination::Config) => config::open(cx),
        Effect::Navigate(Destination::Activity) => activity::open(cx),
        Effect::Navigate(Destination::Playstyle) => playstyle::open(cx),
        Effect::Toast(toast) => cx.show(toast),
    }
}

/// The home screen's work.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Request {
    /// Reads the scope the last launch saved.
    Restore,
    /// Saves the scope for the next launch.
    SaveScope(Scope),
    /// The preferences, applied to the app as they arrive.
    Preferences,
    /// The user's own counters for the cache alone, for a screen other than home.
    MyMetrics,
    /// The featured card's image from Scryfall's image CDN.
    FlavorImage { url: String },
    /// Home's own fetches, stamped with the visit that asked for them.
    Visit { visit: Visit, job: Job },
}

/// One of home's fetches, each carrying its ticket back to the model.
#[derive(Debug, Clone, PartialEq, Eq)]
pub(crate) enum Job {
    /// The user as the server has it now.
    User { ticket: Ticket },
    /// The user's own counters.
    MyMetrics { ticket: Ticket },
    /// The color map.
    PlayStyle { ticket: Ticket },
    /// This hour's featured flavor card.
    Flavor { ticket: Ticket },
}

impl Request {
    /// The action an error from this request is reported under, fixed so a renamed variant keeps its reports.
    pub(crate) fn action(&self) -> &'static str {
        match self {
            Self::Restore => "restore",
            Self::SaveScope(..) => "save_scope",
            Self::Preferences => "preferences",
            Self::MyMetrics
            | Self::Visit {
                job: Job::MyMetrics { .. },
                ..
            } => "my_metrics",
            Self::FlavorImage { .. } => "flavor_image",
            Self::Visit {
                job: Job::User { .. },
                ..
            } => "user",
            Self::Visit {
                job: Job::PlayStyle { .. },
                ..
            } => "play_style",
            Self::Visit {
                job: Job::Flavor { .. },
                ..
            } => "flavor",
        }
    }
}

/// The home screen's results.
#[derive(Debug)]
pub(crate) enum Update {
    Restored(Scope),
    Preferences(Box<UserPreferences>),
    /// The user's own counters, for the cache alone.
    MyMetrics {
        user_id: Uuid,
        figures: Figures,
    },
    FlavorImage {
        url: String,
        image: Option<Box<DynamicImage>>,
    },
    /// A reply to one of home's fetches, stamped with the visit that asked.
    Visit {
        visit: Visit,
        reply: Reply,
    },
}

/// A reply to one of home's fetches.
#[derive(Debug)]
pub(crate) enum Reply {
    User {
        ticket: Ticket,
        user: Box<User>,
    },
    UserFailed {
        ticket: Ticket,
    },
    MyMetrics {
        ticket: Ticket,
        user_id: Uuid,
        figures: Figures,
    },
    PlayStyle {
        ticket: Ticket,
        user_id: Uuid,
        summary: Box<HttpPlayStyleSummary>,
    },
    PlayStyleFailed {
        ticket: Ticket,
    },
    Flavor {
        ticket: Ticket,
        flavor: Box<FlavorCard>,
    },
    FlavorFailed {
        ticket: Ticket,
    },
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
    let mut view = HomeView::default();
    view.enter();
    mount(&mut view, cx);
    cx.reset(Screen::Home(Box::new(view)));
}

/// What the app does on every mount of home: the model takes what the caches hold and asks for the rest, and the client reads the saved scope once, follows the featured card's image and makes sure of the oracle tag catalog.
fn mount(view: &mut HomeView, cx: &mut Context<'_>) {
    if !cx.shared.caches.restore_asked {
        cx.shared.caches.restore_asked = true;
        cx.request(Request::Restore.into());
    }
    view.counts_ticket = None;
    dispatch(view, Action::Load(Box::new(snapshot(cx.shared))), cx);
    sync_flavor_image(cx);
    oracle_tags::ensure_catalog(cx);
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
        Request::FlavorImage { url } => Update::FlavorImage {
            image: images::fetch(&client.client, &url).await.ok().map(Box::new),
            url,
        },
        Request::Visit { visit, job } => {
            let reply = match fetch(account, visit, job).await {
                Ok(Some(reply)) => reply,
                Ok(None) => return None,
                Err(update) => return Some(update),
            };
            Update::Visit { visit, reply }
        }
    };
    Some(update.into())
}

/// Runs one of home's fetches. `Ok(None)` when a failure says nothing, `Err` when the session ended.
async fn fetch(
    account: &Account,
    visit: Visit,
    job: Job,
) -> Result<Option<Reply>, screens::Update> {
    let client = &account.client;
    let ended = |reply: Reply| move |_| Update::Visit { visit, reply }.into();
    Ok(Some(match job {
        Job::User { ticket } => {
            match async { client.fetch_user(&account.fresh().await?).await }.await {
                Ok(user) => {
                    account.set_user(user.clone()).await;
                    Reply::User {
                        ticket,
                        user: Box::new(user),
                    }
                }
                Err(e) => return Err(screens::failure(&e, ended(Reply::UserFailed { ticket }))),
            }
        }
        Job::MyMetrics { ticket } => {
            let Ok(session) = account.fresh().await else {
                return Ok(None);
            };
            let Ok(counters) = client.my_metrics(&session).await else {
                return Ok(None);
            };
            Reply::MyMetrics {
                ticket,
                user_id: session.user.id,
                figures: Figures::from(&counters),
            }
        }
        Job::PlayStyle { ticket } => {
            let result = async {
                let session = account.fresh().await?;
                let summary = client.play_style_summary(&session).await?;
                Ok((session.user.id, summary))
            }
            .await;
            match result {
                Ok((user_id, summary)) => Reply::PlayStyle {
                    ticket,
                    user_id,
                    summary: Box::new(summary),
                },
                Err(e) => {
                    return Err(screens::failure(
                        &e,
                        ended(Reply::PlayStyleFailed { ticket }),
                    ));
                }
            }
        }
        Job::Flavor { ticket } => match client.featured_flavor().await {
            Ok(card) => Reply::Flavor {
                ticket,
                flavor: Box::new(FlavorCard::fetched(card, Utc::now())),
            },
            Err(_) => Reply::FlavorFailed { ticket },
        },
    }))
}

/// Whether this terminal draws real images (kitty, iTerm2 or sixel), which the featured card's image needs; half blocks are too coarse to tell a card by.
pub(crate) fn draws_images(shared: &Shared) -> bool {
    shared
        .images
        .as_ref()
        .is_some_and(|picker| picker.protocol_type() != ProtocolType::Halfblocks)
}

/// Keeps the featured card's image in step with the card: asks for a new card's image, and drops the old one, or every image when the terminal draws none. A card with no image has none to ask for.
fn sync_flavor_image(cx: &mut Context<'_>) {
    let wanted = cx
        .shared
        .caches
        .flavor
        .as_ref()
        .filter(|_| draws_images(cx.shared))
        .and_then(|flavor| {
            flavor
                .card
                .scryfall_data
                .primary_image_url(ImageSize::Normal)
        })
        .map(str::to_owned);
    let held = cx
        .shared
        .caches
        .flavor_image
        .as_ref()
        .map(|i| i.url.as_str());
    if held == wanted.as_deref() {
        return;
    }
    cx.shared.caches.flavor_image = wanted.clone().map(|url| FlavorImage {
        url,
        image: CardImage::Loading,
    });
    if let Some(url) = wanted {
        cx.request(Request::FlavorImage { url }.into());
    }
}

/// Writes a reply to the caches and turns it into the model's action. The caches take it whichever visit asked; counters or a map fetched for another user than the one signed in now reach neither.
fn take(reply: Reply, cx: &mut Context<'_>) -> Option<Action> {
    let greeted = cx.shared.caches.greeted;
    let signed_in = user_id(cx.shared);
    Some(match reply {
        Reply::User { ticket, user } => {
            cx.shared.username = Some(user.username.to_string());
            cx.shared.user = Some((*user).clone());
            Action::UserLoaded {
                ticket,
                user,
                greeted,
            }
        }
        Reply::UserFailed { ticket } => Action::UserFailed {
            ticket,
            cached: cx.shared.user.clone().map(Box::new),
            greeted,
        },
        Reply::MyMetrics {
            ticket,
            user_id,
            figures,
        } => {
            if signed_in != Some(user_id) {
                return None;
            }
            store_my_counts(cx, user_id, figures);
            Action::MyCountsLoaded { ticket, figures }
        }
        Reply::PlayStyle {
            ticket,
            user_id,
            summary,
        } => {
            if signed_in != Some(user_id) {
                return None;
            }
            cx.shared.caches.play_style = Some(CachedPlayStyle {
                user_id,
                summary: (*summary).clone(),
            });
            Action::PlayStyleLoaded {
                ticket,
                summary: *summary,
            }
        }
        Reply::PlayStyleFailed { ticket } => Action::PlayStyleFailed { ticket },
        Reply::Flavor { ticket, flavor } => {
            cx.shared.caches.flavor = Some((*flavor).clone());
            sync_flavor_image(cx);
            Action::FlavorLoaded { ticket, flavor }
        }
        Reply::FlavorFailed { ticket } => Action::FlavorFailed { ticket },
    })
}

/// Keeps `user_id`'s own counters, fetched now.
fn store_my_counts(cx: &mut Context<'_>, user_id: Uuid, figures: Figures) {
    cx.shared.caches.my_counts = Some(CachedMyCounts {
        user_id,
        figures,
        fetched_at: Instant::now(),
    });
}

pub(crate) fn apply(nav: &mut Nav, update: Update, cx: &mut Context<'_>) {
    match update {
        Update::Restored(scope) => {
            if cx.shared.caches.scope == scope {
                return;
            }
            match nav.find_mut(pick) {
                // The model has one way to change the scope; it saves the restored one again, which changes nothing on disk.
                Some(view) if view.model.scope() != scope => {
                    let mine_fresh = mine_fresh(cx.shared);
                    dispatch(view, Action::ToggleScope { mine_fresh }, cx);
                }
                _ => cx.shared.caches.scope = scope,
            }
        }
        Update::Preferences(preferences) => cx.use_preferences(&preferences),
        Update::MyMetrics { user_id, figures } => store_my_counts(cx, user_id, figures),
        Update::FlavorImage { url, image } => {
            if let Some(picker) = &cx.shared.images
                && let Some(slot) = cx.shared.caches.flavor_image.as_mut()
                && slot.url == url
                && matches!(slot.image, CardImage::Loading)
            {
                slot.image = match image {
                    Some(image) => {
                        CardImage::Ready(Box::new(RefCell::new(picker.new_resize_protocol(*image))))
                    }
                    None => CardImage::Failed,
                };
            }
        }
        Update::Visit { visit, reply } => {
            if let Some(action) = take(reply, cx)
                && let Some(view) = nav.find_mut(pick)
                && view.visit == visit
            {
                dispatch(view, action, cx);
            }
        }
    }
}

/// Hands everybody's counts to an open home that asked for them, as they land in `shared.counts`.
pub(crate) fn counts_arrived(nav: &mut Nav, cx: &mut Context<'_>) {
    let Some(figures) = cx.shared.counts.metrics.as_ref().map(Figures::from) else {
        return;
    };
    if let Some(view) = nav.find_mut(pick)
        && let Some(ticket) = view.counts_ticket
    {
        dispatch(view, Action::CountsLoaded { ticket, figures }, cx);
    }
}

/// Asks an open home to fetch the featured card again if the server has picked a new one, as the app does each time the session refreshes.
pub(crate) fn check_flavor(nav: &mut Nav, cx: &mut Context<'_>) {
    if let Some(view) = nav.find_mut(pick) {
        dispatch(view, Action::CheckFlavor { now: Utc::now() }, cx);
    }
}

/// Switches whose counters show.
pub(crate) fn toggle_scope(view: &mut HomeView, cx: &mut Context<'_>) {
    let mine_fresh = mine_fresh(cx.shared);
    dispatch(view, Action::ToggleScope { mine_fresh }, cx);
}

/// How long the strip has waited for its figures, for drawing.
pub(crate) fn since_open(view: &HomeView) -> Duration {
    view.figures_opened.elapsed()
}
