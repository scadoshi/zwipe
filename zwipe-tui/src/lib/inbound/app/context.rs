//! What a screen's handler gets besides its own state: the shared state, and a way to ask the loop for work, navigation and quitting.

use zwipe_core::domain::user::preferences::UserPreferences;

use super::Shared;
use crate::domain::theme::Palette;
use crate::inbound::screens::{Request, Screen};
use crate::inbound::widgets::toast::ToastKind;
use crate::outbound::theme_file::RememberedTheme;

/// A change to the back stack, applied by the loop once the handler returns.
#[derive(Debug)]
pub(crate) enum Navigation {
    /// Opens a screen over the current one.
    Push(Screen),
    /// Swaps the screen on top for another.
    Replace(Screen),
    /// Closes the screen on top and tells the one under it, or quits from the bottom screen.
    Back,
    /// Drops every screen and starts over from this one.
    Reset(Screen),
}

/// What a handler asked for, in the order it asked.
#[derive(Debug, Default)]
pub(crate) struct Effects {
    pub(crate) requests: Vec<Request>,
    pub(crate) navigation: Vec<Navigation>,
    pub(crate) quit: bool,
}

/// A handler's view of the app: the shared state to read and change, and the effects it queues.
///
/// Handlers never touch the network or the back stack directly; they queue a [`Request`] or a [`Navigation`] here.
#[derive(Debug)]
pub(crate) struct Context<'app> {
    pub(crate) shared: &'app mut Shared,
    effects: Effects,
}

impl<'app> Context<'app> {
    pub(crate) fn new(shared: &'app mut Shared) -> Self {
        Self {
            shared,
            effects: Effects::default(),
        }
    }

    /// Queues background work; its result comes back as an update.
    pub(crate) fn request(&mut self, request: Request) {
        self.effects.requests.push(request);
    }

    pub(crate) fn push(&mut self, screen: Screen) {
        self.effects.navigation.push(Navigation::Push(screen));
    }

    /// Swaps the screen on top for `screen`, so going back skips the one it replaces.
    pub(crate) fn replace(&mut self, screen: Screen) {
        self.effects.navigation.push(Navigation::Replace(screen));
    }

    /// Closes the screen on top; the one under it may refresh itself. From the bottom screen this quits.
    pub(crate) fn back(&mut self) {
        self.effects.navigation.push(Navigation::Back);
    }

    pub(crate) fn reset(&mut self, screen: Screen) {
        self.effects.navigation.push(Navigation::Reset(screen));
    }

    pub(crate) fn quit(&mut self) {
        self.effects.quit = true;
    }

    /// Shows a toast that leaves on its own.
    pub(crate) fn toast(&mut self, kind: ToastKind, message: String) {
        self.shared.toasts.push(kind, message);
    }

    /// Draws in `preferences` from now on and remembers their theme for the next launch.
    pub(crate) fn use_preferences(&mut self, preferences: &UserPreferences) {
        self.shared.palette = Palette::for_preferences(preferences);
        self.request(Request::RememberTheme(RememberedTheme {
            theme: preferences.theme.clone(),
            dark_mode: preferences.dark_mode,
        }));
    }

    pub(crate) fn into_effects(self) -> Effects {
        self.effects
    }
}
