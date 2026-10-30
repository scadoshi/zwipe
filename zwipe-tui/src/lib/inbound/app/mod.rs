//! What the screens draw from: which screen is up and its state.

use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::DeckProfile;

use crate::domain::swipe::Stack;
use crate::domain::theme::Palette;

/// Everything the UI draws from.
#[derive(Debug)]
pub(crate) struct App {
    pub(crate) screen: Screen,
    /// The account's ZWIPE theme, or before sign-in the last account's, or gruvbox dark.
    pub(crate) palette: Palette,
    pub(crate) username: Option<String>,
    /// A one-line message under the deck list and the swipe screen.
    pub(crate) status: String,
}

impl App {
    pub(crate) fn new(palette: Palette) -> Self {
        Self {
            screen: Screen::Login(LoginForm::default()),
            palette,
            username: None,
            status: String::new(),
        }
    }
}

/// Which screen is up.
#[derive(Debug)]
pub(crate) enum Screen {
    Login(LoginForm),
    Decks(DeckList),
    Swipe(Box<SwipeView>),
    /// The server no longer accepts this version. Only quitting is possible.
    UpdateRequired,
}

/// The sign-in form.
#[derive(Debug, Default, PartialEq, Eq)]
pub(crate) struct LoginForm {
    /// Username or email.
    pub(crate) identifier: String,
    pub(crate) password: String,
    pub(crate) focus: LoginField,
    pub(crate) error: Option<String>,
    /// Set while the sign-in request is out.
    pub(crate) busy: bool,
}

impl LoginForm {
    /// The field the cursor is in.
    pub(crate) fn focused_mut(&mut self) -> &mut String {
        match self.focus {
            LoginField::Identifier => &mut self.identifier,
            LoginField::Password => &mut self.password,
        }
    }
}

#[derive(Debug, Default, Clone, Copy, PartialEq, Eq)]
pub(crate) enum LoginField {
    #[default]
    Identifier,
    Password,
}

impl LoginField {
    pub(crate) fn other(self) -> Self {
        match self {
            Self::Identifier => Self::Password,
            Self::Password => Self::Identifier,
        }
    }
}

/// The user's decks.
#[derive(Debug, Default)]
pub(crate) struct DeckList {
    pub(crate) decks: Vec<DeckProfile>,
    pub(crate) selected: usize,
    pub(crate) loading: bool,
    pub(crate) error: Option<String>,
}

impl DeckList {
    pub(crate) fn loading() -> Self {
        Self {
            loading: true,
            ..Self::default()
        }
    }

    pub(crate) fn selected_deck(&self) -> Option<&DeckProfile> {
        self.decks.get(self.selected)
    }

    pub(crate) fn next(&mut self) {
        self.selected = self
            .selected
            .saturating_add(1)
            .min(self.decks.len().saturating_sub(1));
    }

    pub(crate) fn previous(&mut self) {
        self.selected = self.selected.saturating_sub(1);
    }
}

/// Swiping cards into one deck.
#[derive(Debug)]
pub(crate) struct SwipeView {
    pub(crate) deck: DeckProfile,
    /// `None` until the deck has loaded and the query is known.
    pub(crate) stack: Option<Stack>,
    pub(crate) query: Option<CardQuery>,
    /// Set while a page of cards is being fetched.
    pub(crate) fetching: bool,
    /// Copies added to the main board here, less the ones undone.
    pub(crate) added: i64,
    pub(crate) error: Option<String>,
}

impl SwipeView {
    pub(crate) fn opening(deck: DeckProfile) -> Self {
        Self {
            deck,
            stack: None,
            query: None,
            fetching: false,
            added: 0,
            error: None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn login_focus_toggles_between_the_two_fields() {
        assert_eq!(LoginField::Identifier.other(), LoginField::Password);
        assert_eq!(LoginField::Password.other(), LoginField::Identifier);
    }

    #[test]
    fn an_empty_deck_list_keeps_the_selection_at_zero() {
        let mut list = DeckList::default();
        list.next();
        list.previous();
        assert_eq!(list.selected, 0);
        assert!(list.selected_deck().is_none());
    }
}
