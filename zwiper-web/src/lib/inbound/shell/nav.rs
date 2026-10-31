//! The sidebar's entries and which one a route belongs to.

use crate::inbound::router::Route;

/// One sidebar entry.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Section {
    /// Home.
    Home,
    /// The deck list and every screen reached from a deck.
    Decks,
    /// Activity.
    Activity,
    /// Playstyle, with the commander maybeboard it leads to.
    Playstyle,
    /// Account, with the privacy policy and changelog it links to.
    Account,
    /// Config.
    Config,
}

impl Section {
    /// Every entry, top to bottom.
    pub const ALL: [Section; 6] = [
        Section::Home,
        Section::Decks,
        Section::Activity,
        Section::Playstyle,
        Section::Account,
        Section::Config,
    ];

    /// The entry's text.
    pub fn label(self) -> &'static str {
        match self {
            Section::Home => "Home",
            Section::Decks => "Decks",
            Section::Activity => "Activity",
            Section::Playstyle => "Playstyle",
            Section::Account => "Account",
            Section::Config => "Config",
        }
    }

    /// Where the entry goes.
    pub fn route(self) -> Route {
        match self {
            Section::Home => Route::Home {},
            Section::Decks => Route::DeckList {},
            Section::Activity => Route::Activity {},
            Section::Playstyle => Route::Playstyle {},
            Section::Account => Route::Account {},
            Section::Config => Route::Config {},
        }
    }

    /// The entry `route` lights up, or `None` for a route outside the
    /// sidebar (the public screens, not found).
    pub fn of(route: &Route) -> Option<Section> {
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
            Route::Playstyle {} | Route::CommanderMaybeboard {} => Some(Section::Playstyle),
            Route::Account {} | Route::PrivacyPolicy {} | Route::Changelog {} => {
                Some(Section::Account)
            }
            Route::Config {} => Some(Section::Config),
            Route::Login {}
            | Route::Register {}
            | Route::ForgotPassword {}
            | Route::NotFound { .. } => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn every_entry_lights_itself_up() {
        for section in Section::ALL {
            assert_eq!(Section::of(&section.route()), Some(section), "{section:?}");
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
            assert_eq!(Section::of(&route), Some(Section::Decks), "{route:?}");
        }
    }

    #[test]
    fn public_screens_light_up_nothing() {
        assert_eq!(Section::of(&Route::Login {}), None);
        assert_eq!(Section::of(&Route::ForgotPassword {}), None);
    }
}
