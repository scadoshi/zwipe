//! The nav's entries, which one a route belongs to, and where a screen's
//! back arrow goes.

use dioxus::prelude::*;

use crate::inbound::router::Route;

/// Whether the main menu is open, shared by the nav over every screen.
/// Provided once at the app root so the panel outlives the screen it was
/// opened on and its close transition plays over the next one.
#[derive(Debug, Clone, Copy)]
pub struct MenuOpen(pub Signal<bool>);

/// Where the header's back arrow goes from a screen.
#[derive(Debug, Clone, PartialEq)]
pub enum Back {
    /// The screen before this one, or Home when the tab has no history.
    Previous,
    /// The screen's fixed parent.
    To(Route),
}

impl Back {
    /// The back arrow's target from `route`, or `None` on Home and the auth
    /// screens, whose left corner is empty. A screen inside a deck goes up
    /// the deck tree, the maybeboard to Playstyle, Privacy policy to About,
    /// and a section goes to the screen before it.
    pub fn of(route: &Route) -> Option<Back> {
        match route {
            Route::Home {} | Route::Login {} | Route::Register {} | Route::ForgotPassword {} => {
                None
            }
            Route::DeckList {}
            | Route::Activity {}
            | Route::Playstyle {}
            | Route::Account {}
            | Route::Config {}
            | Route::About {}
            | Route::NotFound { .. } => Some(Back::Previous),
            Route::CreateDeck {} | Route::ViewDeck { .. } => Some(Back::To(Route::DeckList {})),
            Route::EditDeck { deck_id }
            | Route::ImportDeck { deck_id }
            | Route::ExportDeck { deck_id }
            | Route::AddDeckCard { deck_id }
            | Route::ViewDeckCard { deck_id }
            | Route::RemoveDeckCard { deck_id } => {
                Some(Back::To(Route::ViewDeck { deck_id: *deck_id }))
            }
            Route::CommanderMaybeboard {} => Some(Back::To(Route::Playstyle {})),
            Route::PrivacyPolicy {} => Some(Back::To(Route::About {})),
        }
    }
}

/// The menu's sections, the shape shared with the app; the routes they open
/// are this client's, in [`route_of`] and [`section_of`].
pub use components::MenuSection as Section;

/// The screen `section` opens.
#[must_use]
pub const fn route_of(section: Section) -> Route {
    match section {
        Section::Home => Route::Home {},
        Section::Decks => Route::DeckList {},
        Section::Playstyle => Route::Playstyle {},
        Section::Maybeboard => Route::CommanderMaybeboard {},
        Section::Activity => Route::Activity {},
        Section::Account => Route::Account {},
        Section::Config => Route::Config {},
        Section::About => Route::About {},
    }
}

/// The section `route` sits under, lit in the menu, or `None` for a route
/// outside it (the public screens, not found).
#[must_use]
pub const fn section_of(route: &Route) -> Option<Section> {
    match route {
        Route::Home {} => Some(Section::Home),
        Route::DeckList {}
        | Route::CreateDeck {}
        | Route::EditDeck { .. }
        | Route::ViewDeck { .. }
        | Route::ImportDeck { .. }
        | Route::ExportDeck { .. }
        | Route::AddDeckCard { .. }
        | Route::ViewDeckCard { .. }
        | Route::RemoveDeckCard { .. } => Some(Section::Decks),
        Route::Activity {} => Some(Section::Activity),
        Route::Playstyle {} => Some(Section::Playstyle),
        Route::CommanderMaybeboard {} => Some(Section::Maybeboard),
        Route::Account {} => Some(Section::Account),
        Route::Config {} => Some(Section::Config),
        Route::PrivacyPolicy {} | Route::About {} => Some(Section::About),
        Route::Login {}
        | Route::Register {}
        | Route::ForgotPassword {}
        | Route::NotFound { .. } => None,
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn every_entry_lights_itself_up() {
        for section in Section::ALL {
            assert_eq!(section_of(&route_of(section)), Some(section), "{section:?}");
        }
    }

    #[test]
    fn deck_screens_light_up_decks() {
        let deck_id = Uuid::new_v4();
        for route in [
            Route::ViewDeck { deck_id },
            Route::AddDeckCard { deck_id },
            Route::ExportDeck { deck_id },
            Route::CreateDeck {},
        ] {
            assert_eq!(section_of(&route), Some(Section::Decks), "{route:?}");
        }
    }

    #[test]
    fn home_and_the_auth_screens_have_no_back_arrow() {
        for route in [Route::Home {}, Route::Login {}, Route::Register {}] {
            assert_eq!(Back::of(&route), None, "{route:?}");
        }
    }

    #[test]
    fn a_deck_screen_goes_back_up_the_deck_tree() {
        let deck_id = Uuid::new_v4();
        assert_eq!(
            Back::of(&Route::AddDeckCard { deck_id }),
            Some(Back::To(Route::ViewDeck { deck_id }))
        );
        assert_eq!(
            Back::of(&Route::ViewDeck { deck_id }),
            Some(Back::To(Route::DeckList {}))
        );
        assert_eq!(Back::of(&Route::Activity {}), Some(Back::Previous));
    }

    #[test]
    fn public_screens_light_up_nothing() {
        assert_eq!(section_of(&Route::Login {}), None);
        assert_eq!(section_of(&Route::ForgotPassword {}), None);
    }

    #[test]
    fn the_privacy_policy_lights_up_about_and_goes_back_to_it() {
        assert_eq!(section_of(&Route::PrivacyPolicy {}), Some(Section::About));
        assert_eq!(
            Back::of(&Route::PrivacyPolicy {}),
            Some(Back::To(Route::About {}))
        );
    }
}
