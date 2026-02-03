#[cfg(feature = "zerver")]
use crate::domain::card::models::{card_profile::CardProfile, scryfall_data::ScryfallData, Card};
#[cfg(feature = "zerver")]
use std::collections::HashMap;
#[cfg(feature = "zerver")]
use uuid::Uuid;

#[cfg(feature = "zerver")]
/// Extension trait to combine Scryfall data with card profiles into complete Card objects.
///
/// "Sleeving" matches Scryfall data with corresponding card profiles by UUID,
/// creating complete Card objects. Unmatched items are filtered out.
pub trait SleeveScryfallData {
    /// Combines Scryfall data with card profiles, preserving card profile sort order.
    ///
    /// Matches by comparing `ScryfallData.id` with `CardProfile.scryfall_data_id`.
    /// Returns only successfully matched pairs as `Card` objects.
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl SleeveScryfallData for Vec<ScryfallData> {
    fn sleeve(self, card_profiles: Vec<CardProfile>) -> Vec<Card> {
        let mut data_map: HashMap<Uuid, ScryfallData> = self
            .into_iter()
            .map(|sfd| (sfd.id.to_owned(), sfd))
            .collect();

        card_profiles
            .into_iter()
            .filter_map(|cp| {
                data_map
                    .remove(&cp.scryfall_data_id)
                    .map(|sfd| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}

#[cfg(feature = "zerver")]
/// Extension trait to combine card profiles with Scryfall data into complete Card objects.
///
/// "Sleeving" matches card profiles with corresponding Scryfall data by UUID,
/// creating complete Card objects. Unmatched items are filtered out.
pub trait SleeveCardProfile {
    /// Combines card profiles with Scryfall data, preserving Scryfall data sort order.
    ///
    /// Matches by comparing `CardProfile.scryfall_data_id` with `ScryfallData.id`.
    /// Returns only successfully matched pairs as `Card` objects.
    fn sleeve(self, scryfall_data: Vec<ScryfallData>) -> Vec<Card>;
}

#[cfg(feature = "zerver")]
impl SleeveCardProfile for Vec<CardProfile> {
    fn sleeve(self, scryfall_data: Vec<ScryfallData>) -> Vec<Card> {
        // Build map of card profiles keyed by scryfall_data_id
        let mut profile_map: HashMap<Uuid, CardProfile> = self
            .into_iter()
            .map(|cp| (cp.scryfall_data_id, cp))
            .collect();

        // Iterate over scryfall_data to preserve DB sort order
        scryfall_data
            .into_iter()
            .filter_map(|sfd| {
                profile_map
                    .remove(&sfd.id)
                    .map(|cp| Card::new(cp, sfd))
            })
            .collect::<Vec<Card>>()
    }
}
