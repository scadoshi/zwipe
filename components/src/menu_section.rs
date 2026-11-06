//! The main menu's sections, the shape both clients' signed-in menus share.
//!
//! The list holds the deck sections as text rows and the footer holds the
//! rest as icons. Which screen each section opens is the client's: its
//! router maps a section to a route and a route back to the section it lights.

use crate::icons::{COG, HOUSE, INFO, PERSON};

/// A screen the signed-in menu opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum MenuSection {
    /// Home.
    Home,
    /// The deck list, and every screen inside a deck.
    Decks,
    /// Playstyle, with its suggested commanders.
    Playstyle,
    /// The commander maybeboard.
    Maybeboard,
    /// Activity.
    Activity,
    /// Account.
    Account,
    /// Settings (the `config` screen).
    Config,
    /// About, and the privacy policy it links to.
    About,
}

impl MenuSection {
    /// Every section, in menu order: the listed ones, then the footer's.
    pub const ALL: [Self; 8] = [
        Self::Decks,
        Self::Playstyle,
        Self::Activity,
        Self::Maybeboard,
        Self::Home,
        Self::Account,
        Self::Config,
        Self::About,
    ];

    /// The deck screens, the menu's list, in order.
    pub const LISTED: [Self; 4] = [
        Self::Decks,
        Self::Playstyle,
        Self::Activity,
        Self::Maybeboard,
    ];

    /// The sections the footer holds as icons, left to right.
    pub const FOOTER: [Self; 4] = [Self::Home, Self::Account, Self::Config, Self::About];

    /// The entry's label, the title of the screen it opens.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Home => "Home",
            Self::Decks => "Decks",
            Self::Playstyle => "Playstyle",
            Self::Maybeboard => "Commander maybeboard",
            Self::Activity => "Activity",
            Self::Account => "Account",
            Self::Config => "Settings",
            Self::About => "About",
        }
    }

    /// The footer icon's path, for the sections in [`Self::FOOTER`]; the
    /// listed sections draw their label instead.
    #[must_use]
    pub const fn icon(self) -> Option<&'static str> {
        match self {
            Self::Home => Some(HOUSE),
            Self::Account => Some(PERSON),
            Self::Config => Some(COG),
            Self::About => Some(INFO),
            Self::Decks | Self::Playstyle | Self::Maybeboard | Self::Activity => None,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_listed_and_footer_sections_partition_all() {
        let mut split: Vec<MenuSection> = MenuSection::LISTED
            .into_iter()
            .chain(MenuSection::FOOTER)
            .collect();
        let mut all: Vec<MenuSection> = MenuSection::ALL.to_vec();
        split.sort_by_key(|section| section.label());
        all.sort_by_key(|section| section.label());
        assert_eq!(split, all);
        for section in MenuSection::FOOTER {
            assert!(section.icon().is_some(), "{section:?} has no icon");
        }
        for section in MenuSection::LISTED {
            assert_eq!(section.icon(), None, "{section:?} draws its label");
        }
    }
}
