//! Everything the UI draws from: the back stack of screens, and the state every screen shares.

mod context;
pub(crate) mod session;

use std::time::Instant;

use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui_image::picker::Picker;
use zwipe_core::domain::user::{User, UserPreferences};

pub(crate) use self::context::{Context, Effects, Navigation};
use super::nav::Nav;
use super::screens::{Screen, update_required};
use super::widgets::alert::AlertDialog;
use super::widgets::entrance::{self, DURATION as ENTRANCE};
use super::widgets::hero::Counts;
use super::widgets::hint::HintDialog;
use super::widgets::splash::Splash;
use super::widgets::toast::Toasts;
use super::widgets::wipe::Wipe;
use crate::domain::changelog::ChangelogCache;
use crate::domain::theme::Palette;

/// Everything the UI draws from.
#[derive(Debug)]
pub(crate) struct App {
    /// The screens, the one on top drawn and taking keys.
    pub(crate) nav: Nav,
    pub(crate) shared: Shared,
    /// When the screen on top took over, for its entrance.
    entered_at: Instant,
    /// The palette last drawn, to notice a theme change.
    drawn_palette: Palette,
    /// The sweep from the last theme to this one, while it runs.
    wipe: Option<Wipe>,
}

/// What every screen reads and may change, kept apart from the back stack so a screen's handler can hold both.
#[derive(Debug)]
pub(crate) struct Shared {
    /// The account's ZWIPE theme, or before sign-in the last account's, or gruvbox dark.
    pub(crate) palette: Palette,
    pub(crate) username: Option<String>,
    /// The signed-in user as last fetched, which the account screen opens on.
    pub(crate) user: Option<User>,
    /// A one-line message under most screens.
    pub(crate) status: String,
    /// How this terminal draws images, or `None` when card images are off.
    pub(crate) images: Option<Picker>,
    /// Toasts up in the bottom-right corner.
    pub(crate) toasts: Toasts,
    /// A hint over the screen, a `?` topic or a one-time hint, which takes every key until it closes.
    pub(crate) hint: Option<HintDialog>,
    /// The Help & feedback dialog, over everything but the toasts.
    pub(crate) help: Option<AlertDialog>,
    /// The server no longer accepts this version: the update notice replaces every screen until it does again.
    pub(crate) update_required: bool,
    /// The changelog, fetched once at launch.
    pub(crate) changelog: ChangelogCache,
    /// The account's preferences as last applied, which Config opens on before it fetches them.
    pub(crate) preferences: Option<UserPreferences>,
    /// Everybody's counts, which the sign-in screens' hero shows.
    pub(crate) counts: Counts,
    /// The opening over the first screen, while it runs.
    pub(crate) splash: Option<Splash>,
    /// When the first screen's entrances may start: as the splash fades, or at once without one. Logo decodes and rolling counters wait for it.
    pub(crate) gate: Instant,
}

impl Shared {
    /// When an entrance mounted at `now` starts: now, or when the splash lets it.
    pub(crate) fn entrance_start(&self, now: Instant) -> Instant {
        now.max(self.gate)
    }

    /// Ends the opening at `now`, so the first screen's entrances start.
    pub(crate) fn skip_splash(&mut self, now: Instant) {
        if self.splash.take().is_some() {
            self.gate = self.gate.min(now);
        }
    }
}

impl App {
    /// The sign-in screen in `palette`.
    #[expect(
        clippy::large_types_passed_by_value,
        reason = "the palette is copied once, at start"
    )]
    pub(crate) fn new(palette: Palette) -> Self {
        let now = Instant::now();
        Self {
            // The first screen makes no entrance of its own: the splash is its opening.
            entered_at: now.checked_sub(ENTRANCE).unwrap_or(now),
            drawn_palette: palette,
            wipe: None,
            nav: Nav::new(Screen::Login(Box::default())),
            shared: Shared {
                palette,
                username: None,
                user: None,
                status: String::new(),
                images: None,
                toasts: Toasts::default(),
                hint: None,
                help: None,
                update_required: false,
                changelog: ChangelogCache::default(),
                preferences: None,
                counts: Counts::default(),
                splash: None,
                gate: now,
            },
        }
    }

    /// Whether something on screen is moving, so the loop should redraw on a timer: the screen on top, a toast waiting to leave, the splash or a theme sweep.
    pub(crate) fn animating(&self) -> bool {
        self.nav.top().animating()
            || !self.shared.toasts.is_empty()
            || self.shared.splash.is_some()
            || self.wipe.is_some()
    }

    /// Whether the screen on top is still making its entrance at `now`, so the loop should redraw until it is in.
    pub(crate) fn entering(&self, now: Instant) -> bool {
        entrance::animating(now.saturating_duration_since(self.entered_at))
    }

    /// Notes that a new screen took over at `now`, so it makes its entrance.
    pub(crate) fn screen_changed(&mut self, now: Instant) {
        self.entered_at = now;
    }

    /// Drops what has timed out by `now`, ends the splash once it is gone, and starts a sweep when the theme changed.
    pub(crate) fn tick(&mut self, now: Instant) {
        self.shared.toasts.prune(now);
        if self.shared.splash.is_some_and(|s| s.done(now)) {
            self.shared.splash = None;
        }
        if self.shared.palette != self.drawn_palette {
            self.wipe = Some(Wipe::new(&self.drawn_palette, &self.shared.palette, now));
            self.drawn_palette = self.shared.palette;
        }
        if self.wipe.is_some_and(|w| w.done(now)) {
            self.wipe = None;
        }
    }

    /// Draws the app as of now.
    pub(crate) fn draw(&self, f: &mut Frame<'_>) {
        self.draw_at(f, Instant::now());
    }

    /// Draws the screen on top over the theme's background, then the hint, the help dialog and the toasts over it, and the splash over everything; or, while this version is no longer accepted, only the update notice. A theme sweep recolors what it has not reached yet.
    pub(crate) fn draw_at(&self, f: &mut Frame<'_>, now: Instant) {
        let palette = &self.shared.palette;
        f.render_widget(Block::new().style(palette.base), f.area());
        if self.shared.update_required {
            update_required::draw::draw(f, &self.shared);
            self.sweep(f, now);
            return;
        }
        self.nav.top().draw(f, &self.shared);
        let area = f.area();
        entrance::draw(f, area, now.saturating_duration_since(self.entered_at));
        if let Some(hint) = &self.shared.hint {
            hint.draw(f, palette);
        }
        if let Some(help) = &self.shared.help {
            help.draw(f, palette);
        }
        self.shared.toasts.draw(f, palette, f.area());
        if let Some(splash) = &self.shared.splash {
            splash.draw(f, palette, now);
        }
        self.sweep(f, now);
    }

    /// Recolors the rows a running theme sweep has not reached.
    fn sweep(&self, f: &mut Frame<'_>, now: Instant) {
        if let Some(wipe) = &self.wipe {
            let area = f.area();
            wipe.apply(f.buffer_mut(), area, &self.shared.palette, now);
        }
    }
}

#[cfg(test)]
mod tests {
    use std::time::Duration;

    use super::*;
    use crate::test_support::render;

    #[test]
    fn entrances_wait_for_the_splash_and_skipping_it_opens_the_gate() {
        let mut app = App::new(Palette::terminal());
        let now = Instant::now();
        assert!(
            !app.entering(now),
            "the first screen has the splash instead"
        );
        let splash = Splash::new(now);
        app.shared.gate = splash.gate();
        app.shared.splash = Some(splash);
        assert_eq!(app.shared.entrance_start(now), splash.gate());
        assert!(app.animating());
        render(&app, 30, 8);
        app.shared.skip_splash(now);
        assert!(app.shared.splash.is_none());
        assert_eq!(app.shared.entrance_start(now), now);
        app.screen_changed(now);
        assert!(app.entering(now));
    }

    #[test]
    fn a_theme_change_starts_a_sweep_that_ends() {
        let mut app = App::new(Palette::for_account("gruvbox", true));
        let now = Instant::now();
        app.shared.palette = Palette::for_account("nord", true);
        app.tick(now);
        assert!(app.animating(), "the sweep runs");
        render(&app, 30, 8);
        app.tick(now + Duration::from_secs(1));
        assert!(!app.animating());
    }
}
