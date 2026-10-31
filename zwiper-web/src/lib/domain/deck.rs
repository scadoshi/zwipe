//! Deck rules the app keeps in zwiper that zwipe-ui does not hold yet, as
//! pure functions. Each belongs in zwipe-ui's `deck` module.

use zwipe_core::domain::deck::DeckProfile;

// Belongs in zwipe-ui
/// The deck's command-zone art crops for its list row, in the order
/// commander, partner, background, signature spell, only those it has.
pub fn command_zone_art(profile: &DeckProfile) -> Vec<&str> {
    [
        profile.commander_art_url.as_deref(),
        profile.partner_commander_art_url.as_deref(),
        profile.background_art_url.as_deref(),
        profile.signature_spell_art_url.as_deref(),
    ]
    .into_iter()
    .flatten()
    .collect()
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_fixtures;

    #[test]
    fn art_follows_the_command_zone_order() {
        let mut profile = test_fixtures::deck_profile("Pair");
        profile.background_art_url = Some("bg".to_owned());
        profile.commander_art_url = Some("cmd".to_owned());
        assert_eq!(command_zone_art(&profile), ["cmd", "bg"]);
    }
}
