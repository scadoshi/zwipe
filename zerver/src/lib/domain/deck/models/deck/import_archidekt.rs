//! Archidekt deck import operation: server-side types.
//!
//! The Archidekt outbound adapter reduces a fetched deck to an
//! [`ArchidektDeck`](crate::domain::deck::models::deck::import_archidekt::ArchidektDeck)
//! holding its card list, which the deck service imports into an existing deck
//! exactly like a plain-text import (same boards, same add/replace modes, same
//! result shape), plus the deck's format and its commanders, which fill the
//! deck's profile where it has none. Card *data*
//! is never taken from Archidekt; only identity (Scryfall printing id),
//! quantity, and name.

use uuid::Uuid;
use zwipe_core::domain::deck::Format;
use zwipe_core::domain::deck::deck_profile::DeckProfile;

/// A single card entry from an Archidekt deck.
#[derive(Debug, Clone)]
pub struct ArchidektCard {
    /// Scryfall printing id (`card.uid`), resolved against `scryfall_data.id`.
    /// `Uuid::nil()` if Archidekt gave an unparseable id, which surfaces as unresolved.
    pub scryfall_id: Uuid,
    /// Card name (Archidekt's oracle name), used for the name fallback and the
    /// unresolved report.
    pub name: String,
    /// Number of copies.
    pub quantity: i32,
}

/// An Archidekt deck as the import reads it.
#[derive(Debug, Clone, Default)]
pub struct ArchidektDeck {
    /// The cards on the deck's included categories.
    pub cards: Vec<ArchidektCard>,
    /// The deck's format, `None` when Archidekt's has no match here.
    pub format: Option<Format>,
    /// The cards in the deck's commander category, in the order Archidekt
    /// lists them.
    pub commanders: Vec<ArchidektCard>,
}

/// A commander card from the import, found in the card database.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ResolvedCommander {
    /// The printing the profile points at.
    pub scryfall_id: Uuid,
    /// The card's oracle id, so its copy in the card list is skipped.
    pub oracle_id: Uuid,
    /// The card's name, for the import's report.
    pub name: String,
    /// Whether the card is a Background enchantment, which takes the
    /// background slot rather than the partner's.
    pub is_background: bool,
}

/// Which of the profile's slots the import may fill: the ones still empty.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct OpenSlots {
    /// The profile's format, when it has one.
    pub format: Option<Format>,
    /// Whether the commander slot is empty.
    pub commander: bool,
    /// Whether the partner slot is empty.
    pub partner: bool,
    /// Whether the background slot is empty.
    pub background: bool,
}

impl From<&DeckProfile> for OpenSlots {
    fn from(profile: &DeckProfile) -> Self {
        Self {
            format: profile.format,
            commander: profile.commander_id.is_none(),
            partner: profile.partner_commander_id.is_none(),
            background: profile.background_id.is_none(),
        }
    }
}

/// What an Archidekt import fills on the deck's profile.
#[derive(Debug, Clone, Default, PartialEq, Eq)]
pub struct ProfileFill {
    /// The format, set when the profile has none.
    pub format: Option<Format>,
    /// The commander, set when the profile has none.
    pub commander: Option<ResolvedCommander>,
    /// The partner commander, set alongside a new commander.
    pub partner: Option<ResolvedCommander>,
    /// The background, set alongside a new commander.
    pub background: Option<ResolvedCommander>,
}

impl ProfileFill {
    /// What to fill from Archidekt's `format` and `commanders` into the
    /// `open` slots.
    ///
    /// The format fills an empty format. The commanders fill only an empty
    /// commander slot, and only under a format that has a commander (or no
    /// format at all): the first card that isn't a Background is the
    /// commander, and the next card is the background when it is one and
    /// the format takes one, otherwise the partner when the format takes
    /// one. Each fills only its own empty slot, and a deck's existing
    /// commander never gets a partner or background from the import.
    #[must_use]
    pub fn plan(open: OpenSlots, format: Option<Format>, commanders: &[ResolvedCommander]) -> Self {
        let format_set = open.format.is_none().then_some(format).flatten();
        let effective = open.format.or(format_set);
        let mut fill = Self {
            format: format_set,
            ..Self::default()
        };
        if !open.commander || !effective.is_none_or(|f| f.has_commander()) {
            return fill;
        }
        let Some(lead) = commanders.iter().position(|c| !c.is_background) else {
            return fill;
        };
        let second = commanders
            .iter()
            .enumerate()
            .find(|(i, _)| *i != lead)
            .map(|(_, c)| c.clone());
        fill.commander = commanders.get(lead).cloned();
        if let Some(second) = second {
            if second.is_background {
                if open.background && effective.is_none_or(|f| f.supports_background()) {
                    fill.background = Some(second);
                }
            } else if open.partner && effective.is_none_or(|f| f.supports_partner()) {
                fill.partner = Some(second);
            }
        }
        fill
    }

    /// Whether the import fills nothing.
    #[must_use]
    pub fn is_empty(&self) -> bool {
        self.format.is_none()
            && self.commander.is_none()
            && self.partner.is_none()
            && self.background.is_none()
    }

    /// The cards it sets, commander first, by name.
    #[must_use]
    pub fn card_names(&self) -> Vec<String> {
        [&self.commander, &self.partner, &self.background]
            .into_iter()
            .flatten()
            .map(|c| c.name.clone())
            .collect()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(name: &str, is_background: bool) -> ResolvedCommander {
        ResolvedCommander {
            scryfall_id: Uuid::new_v4(),
            oracle_id: Uuid::new_v4(),
            name: name.to_owned(),
            is_background,
        }
    }

    fn empty() -> OpenSlots {
        OpenSlots {
            format: None,
            commander: true,
            partner: true,
            background: true,
        }
    }

    #[test]
    fn an_empty_profile_takes_the_format_and_commander() {
        let atraxa = card("Atraxa, Praetors' Voice", false);
        let fill = ProfileFill::plan(
            empty(),
            Some(Format::Commander),
            std::slice::from_ref(&atraxa),
        );
        assert_eq!(fill.format, Some(Format::Commander));
        assert_eq!(fill.commander, Some(atraxa));
        assert_eq!(fill.partner, None);
    }

    #[test]
    fn two_partners_fill_commander_and_partner() {
        let (tymna, thrasios) = (card("Tymna the Weaver", false), card("Thrasios", false));
        let fill = ProfileFill::plan(
            empty(),
            Some(Format::Commander),
            &[tymna.clone(), thrasios.clone()],
        );
        assert_eq!(fill.commander, Some(tymna));
        assert_eq!(fill.partner, Some(thrasios));
        assert_eq!(fill.card_names().len(), 2);
    }

    #[test]
    fn a_background_takes_the_background_slot_whichever_comes_first() {
        let (bg, wyll) = (card("Raised by Giants", true), card("Wyll", false));
        let fill = ProfileFill::plan(
            empty(),
            Some(Format::Commander),
            &[bg.clone(), wyll.clone()],
        );
        assert_eq!(fill.commander, Some(wyll));
        assert_eq!(fill.background, Some(bg));
        assert_eq!(fill.partner, None);
    }

    #[test]
    fn filled_slots_stay_as_they_are() {
        let open = OpenSlots {
            format: Some(Format::Modern),
            commander: false,
            partner: true,
            background: true,
        };
        let fill = ProfileFill::plan(
            open,
            Some(Format::Commander),
            &[card("Tymna the Weaver", false), card("Thrasios", false)],
        );
        assert!(fill.is_empty(), "the format and commander were set already");
    }

    #[test]
    fn a_format_with_no_commander_sets_none() {
        let open = OpenSlots {
            format: Some(Format::Modern),
            ..empty()
        };
        let fill = ProfileFill::plan(open, None, &[card("Atraxa, Praetors' Voice", false)]);
        assert!(fill.is_empty());
    }

    #[test]
    fn an_unmapped_format_still_takes_the_commander() {
        let atraxa = card("Atraxa, Praetors' Voice", false);
        let fill = ProfileFill::plan(empty(), None, std::slice::from_ref(&atraxa));
        assert_eq!(fill.format, None);
        assert_eq!(fill.commander, Some(atraxa));
    }

    #[test]
    fn oathbreaker_takes_no_second_card() {
        let fill = ProfileFill::plan(
            empty(),
            Some(Format::Oathbreaker),
            &[
                card("Wrenn and Seven", false),
                card("Kodama's Reach", false),
            ],
        );
        assert!(fill.commander.is_some());
        assert_eq!((fill.partner, fill.background), (None, None));
    }
}
