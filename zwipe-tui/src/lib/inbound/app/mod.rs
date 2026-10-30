//! Everything the UI draws from: the back stack of screens, and the state every screen shares.

mod context;

use ratatui::Frame;
use ratatui::widgets::Block;
use ratatui_image::picker::Picker;
use zwipe_core::domain::user::User;

pub(crate) use self::context::{Context, Effects, Navigation};
use super::nav::Nav;
use super::screens::Screen;
use super::screens::login::LoginForm;
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
}

impl App {
    /// The sign-in screen in `palette`.
    pub(crate) fn new(palette: Palette) -> Self {
        Self {
            nav: Nav::new(Screen::Login(LoginForm::default())),
            shared: Shared {
                palette,
                username: None,
                user: None,
                status: String::new(),
                images: None,
            },
        }
    }

    /// Whether something on screen is moving, so the loop should redraw on a timer.
    pub(crate) fn animating(&self) -> bool {
        self.nav.top().animating()
    }

    /// Draws the screen on top over the theme's background.
    pub(crate) fn draw(&self, f: &mut Frame<'_>) {
        f.render_widget(Block::new().style(self.shared.palette.base), f.area());
        self.nav.top().draw(f, &self.shared);
    }
}
