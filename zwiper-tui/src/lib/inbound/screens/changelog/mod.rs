//! The Changelog screen: the launch's changelog under chip rows that narrow it from `Latest` to a year, a month and a minor version, with a skeleton while it loads and a note when it could not.
//!
//! What the screen knows and decides is zwipe-ui's [`State`]: the branch to draw, the chip rows and what each chip picks, and the releases shown. This view adds the focused chip, the scroll and the cards' entrance. The history is the app's `shared.changelog`, handed to the model on opening and again when the launch fetch lands.

pub(crate) mod draw;
pub(crate) mod keys;
#[cfg(test)]
mod tests;

use std::time::Instant;

use zwipe_ui::screens::changelog::{Action, Destination, Effect, State};

use super::Screen;
use crate::inbound::app::Context;
use crate::inbound::nav::Nav;

/// The Changelog screen's state: the model plus the focused chip, how far the cards are scrolled, and when they entered.
#[derive(Debug)]
pub(crate) struct ChangelogView {
    pub(crate) model: State,
    /// The chip row with the focus: 0 for years, 1 for months, 2 for minor versions.
    pub(crate) row: usize,
    pub(crate) chip: usize,
    pub(crate) scroll: u16,
    /// When the filter last changed, for the cards' entrance.
    pub(crate) filtered_at: Instant,
}

impl Default for ChangelogView {
    fn default() -> Self {
        Self {
            model: State::new(),
            row: 0,
            chip: 0,
            scroll: 0,
            filtered_at: Instant::now(),
        }
    }
}

impl ChangelogView {
    /// Presses the focused chip: the model picks its filter, the cards scroll back to the top and enter again, and the focus stays on a row that still shows.
    pub(crate) fn press(&mut self, cx: &mut Context<'_>) {
        let rows = self.model.chip_rows();
        let Some(chip) = rows.get(self.row).and_then(|r| r.get(self.chip)) else {
            return;
        };
        dispatch(self, Action::Pick(chip.target.clone()), cx);
        self.scroll = 0;
        self.filtered_at = Instant::now();
        let depth = self.model.chip_rows().len();
        if self.row >= depth {
            self.row = depth.saturating_sub(1);
            self.chip = 0;
        }
    }
}

/// Applies `action` to the model and performs the effects it returns.
pub(crate) fn dispatch(view: &mut ChangelogView, action: Action, cx: &mut Context<'_>) {
    for effect in view.model.update(action) {
        match effect {
            Effect::Navigate(Destination::Previous) => cx.back(),
        }
    }
}

fn pick(screen: &mut Screen) -> Option<&mut ChangelogView> {
    match screen {
        Screen::Changelog(view) => Some(view),
        _ => None,
    }
}

/// Opens the Changelog on `Latest` with the history as it stands.
pub(crate) fn open(cx: &mut Context<'_>) {
    let mut view = ChangelogView::default();
    dispatch(&mut view, Action::Load(cx.shared.changelog.clone()), cx);
    cx.push(Screen::Changelog(Box::new(view)));
}

/// Hands an open Changelog the history the launch fetch just left in `shared.changelog`.
pub(crate) fn reload(nav: &mut Nav, cx: &mut Context<'_>) {
    if let Some(view) = nav.find_mut(pick) {
        let cache = cx.shared.changelog.clone();
        dispatch(view, Action::Load(cache), cx);
        view.row = 0;
        view.chip = 0;
        view.scroll = 0;
        view.filtered_at = Instant::now();
    }
}
