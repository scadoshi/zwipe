//! The back stack, like the app's router: push a screen, go back to the one under it, replace the top, or start over.

use std::iter;

use crate::inbound::screens::Screen;

/// The open screens, bottom first. There is always at least one.
#[derive(Debug)]
pub(crate) struct Nav {
    bottom: Screen,
    /// The screens over the bottom one, the last on top.
    above: Vec<Screen>,
}

impl Nav {
    pub(crate) fn new(bottom: Screen) -> Self {
        Self {
            bottom,
            above: Vec::new(),
        }
    }

    /// The screen that is drawn and takes keys.
    pub(crate) fn top(&self) -> &Screen {
        self.above.last().unwrap_or(&self.bottom)
    }

    pub(crate) fn top_mut(&mut self) -> &mut Screen {
        self.above.last_mut().unwrap_or(&mut self.bottom)
    }

    /// How many screens are open, at least one.
    #[cfg(test)]
    pub(crate) fn depth(&self) -> usize {
        self.above.len().saturating_add(1)
    }

    pub(crate) fn push(&mut self, screen: Screen) {
        self.above.push(screen);
    }

    /// Closes the screen on top and returns it, or `None` when it is the bottom one, which stays.
    pub(crate) fn pop(&mut self) -> Option<Screen> {
        self.above.pop()
    }

    /// Swaps the screen on top for `screen`.
    pub(crate) fn replace(&mut self, screen: Screen) {
        *self.top_mut() = screen;
    }

    /// Drops every screen and makes `screen` the only one.
    pub(crate) fn reset(&mut self, screen: Screen) {
        self.above.clear();
        self.bottom = screen;
    }

    /// Every open screen, bottom first.
    pub(crate) fn screens_mut(&mut self) -> impl Iterator<Item = &mut Screen> {
        iter::once(&mut self.bottom).chain(self.above.iter_mut())
    }

    /// The topmost screen `pick` accepts, searching down from the top.
    pub(crate) fn find_mut<'nav, T: ?Sized>(
        &'nav mut self,
        pick: impl FnMut(&'nav mut Screen) -> Option<&'nav mut T>,
    ) -> Option<&'nav mut T> {
        self.above
            .iter_mut()
            .rev()
            .chain(iter::once(&mut self.bottom))
            .find_map(pick)
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::screens::decks::DeckList;

    fn decks(screen: &mut Screen) -> Option<&mut DeckList> {
        match screen {
            Screen::Decks(list) => Some(list),
            _ => None,
        }
    }

    #[test]
    fn push_and_pop_stop_at_the_bottom_screen() {
        let mut nav = Nav::new(Screen::Login(Box::default()));
        nav.push(Screen::UpdateRequired);
        assert_eq!(nav.depth(), 2);
        assert!(matches!(nav.top(), Screen::UpdateRequired));
        assert!(nav.pop().is_some());
        assert!(nav.pop().is_none(), "the bottom screen stays");
        assert!(matches!(nav.top(), Screen::Login(_)));
        assert_eq!(nav.depth(), 1);
    }

    #[test]
    fn replace_swaps_only_the_top_and_reset_clears_the_rest() {
        let mut nav = Nav::new(Screen::Login(Box::default()));
        nav.replace(Screen::UpdateRequired);
        assert!(matches!(nav.top(), Screen::UpdateRequired));
        assert_eq!(nav.depth(), 1);

        nav.push(Screen::Decks(Box::default()));
        nav.replace(Screen::Login(Box::default()));
        assert_eq!(nav.depth(), 2);
        nav.reset(Screen::Decks(Box::default()));
        assert_eq!(nav.depth(), 1);
        assert!(matches!(nav.top(), Screen::Decks(_)));
    }

    #[test]
    fn find_looks_down_from_the_top() {
        let mut nav = Nav::new(Screen::Decks(Box::default()));
        nav.push(Screen::Decks(Box::new(DeckList::loading())));
        nav.push(Screen::UpdateRequired);
        let found = nav.find_mut(decks).map(|list| list.loading);
        assert_eq!(found, Some(true), "the upper deck list");
        nav.reset(Screen::UpdateRequired);
        assert!(nav.find_mut(decks).is_none());
    }
}
