//! Everything the UI draws from: the back stack of screens, and the state every screen shares.

mod context;

use std::time::Instant;

use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui_image::picker::Picker;
use zwipe_core::domain::user::User;

pub(crate) use self::context::{Context, Effects, Navigation};
use super::nav::Nav;
use super::screens::Screen;
use super::screens::login::LoginForm;
use super::widgets::alert::AlertDialog;
use super::widgets::hint::HintDialog;
use super::widgets::toast::Toasts;
use crate::domain::theme::Palette;

/// Everything the UI draws from.
#[derive(Debug)]
pub(crate) struct App {
    /// The screens, the one on top drawn and taking keys.
    pub(crate) nav: Nav,
    pub(crate) shared: Shared,
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
}

impl App {
    /// The sign-in screen in `palette`.
    #[expect(
        clippy::large_types_passed_by_value,
        reason = "the palette is copied once, at start"
    )]
    pub(crate) fn new(palette: Palette) -> Self {
        Self {
            nav: Nav::new(Screen::Login(LoginForm::default())),
            shared: Shared {
                palette,
                username: None,
                user: None,
                status: String::new(),
                images: None,
                toasts: Toasts::default(),
                hint: None,
                help: None,
            },
        }
    }

    /// Whether something on screen is moving, so the loop should redraw on a timer: the screen on top, or a toast waiting to leave.
    pub(crate) fn animating(&self) -> bool {
        self.nav.top().animating() || !self.shared.toasts.is_empty()
    }

    /// Drops what has timed out by `now`.
    pub(crate) fn tick(&mut self, now: Instant) {
        self.shared.toasts.prune(now);
    }

    /// Draws the screen on top over the theme's background, then the hint, the help dialog and the toasts over it.
    pub(crate) fn draw(&self, f: &mut Frame<'_>) {
        let palette = &self.shared.palette;
        f.render_widget(Block::new().style(palette.base), f.area());
        self.nav.top().draw(f, &self.shared);
        if let Some(hint) = &self.shared.hint {
            hint.draw(f, palette);
        }
        if let Some(help) = &self.shared.help {
            help.draw(f, palette);
        }
        self.shared.toasts.draw(f, palette, f.area());
    }
}
