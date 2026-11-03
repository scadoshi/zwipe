//! Swipe select's re-deal: the suggested pile dealt again from a new ranking mid-session, keeping what was already swiped in front.

use uuid::Uuid;
use zwipe_core::domain::card::Card;

/// A suggested card's identity across printings: its oracle id, else its scryfall id.
#[must_use]
pub fn card_key(card: &Card) -> Uuid {
    card.scryfall_data
        .oracle_id
        .unwrap_or(card.scryfall_data.id)
}

/// The pile re-dealt from a new ranking mid-session. The first `swiped` of `dealt` stay in front, so the top card's index and the undo history still line up with them; then every item of `fresh` whose key is not among them, in `fresh`'s order. A `swiped` count past the pile keeps the whole pile.
#[must_use]
pub fn redeal<T: Clone, K: PartialEq>(
    dealt: &[T],
    swiped: usize,
    fresh: &[T],
    key: impl Fn(&T) -> K,
) -> Vec<T> {
    let mut pile: Vec<T> = dealt.iter().take(swiped).cloned().collect();
    let kept: Vec<K> = pile.iter().map(&key).collect();
    pile.extend(
        fresh
            .iter()
            .filter(|item| !kept.contains(&key(item)))
            .cloned(),
    );
    pile
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::test_support;

    fn key(name: &&'static str) -> &'static str {
        name
    }

    #[test]
    fn redeal_keeps_the_swiped_in_front_then_the_new_pile_without_them() {
        let dealt = ["Krenko", "Narset", "Zada", "Ghired"];
        let fresh = ["Zada", "Krenko", "Talrand"];
        assert_eq!(
            redeal(&dealt, 2, &fresh, key),
            vec!["Krenko", "Narset", "Zada", "Talrand"],
            "Krenko was swiped, so the new pile deals Zada then Talrand after the two"
        );
    }

    #[test]
    fn redeal_with_nothing_swiped_starts_at_the_top_of_the_new_pile() {
        let dealt = ["Krenko", "Narset"];
        let fresh = ["Talrand", "Narset"];
        assert_eq!(redeal(&dealt, 0, &fresh, key), vec!["Talrand", "Narset"]);
    }

    #[test]
    fn redeal_to_an_empty_pile_keeps_only_the_swiped() {
        let dealt = ["Krenko", "Narset", "Zada"];
        assert_eq!(redeal(&dealt, 1, &[], key), vec!["Krenko"]);
        assert_eq!(
            redeal(&dealt, 9, &["Talrand"], key),
            vec!["Krenko", "Narset", "Zada", "Talrand"],
            "a swiped count past the pile keeps the whole pile"
        );
    }

    #[test]
    fn redeal_after_an_undo_can_deal_the_undone_card_again() {
        // Three swiped and one undone puts the index back at 2, so the third
        // is no longer swiped and the new pile deals it.
        let dealt = ["Krenko", "Narset", "Zada"];
        let fresh = ["Zada", "Talrand"];
        assert_eq!(
            redeal(&dealt, 2, &fresh, key),
            vec!["Krenko", "Narset", "Zada", "Talrand"]
        );
    }

    #[test]
    fn redeal_from_an_empty_deal_is_the_new_pile() {
        let fresh = ["Talrand", "Narset"];
        assert_eq!(redeal(&[], 3, &fresh, key), vec!["Talrand", "Narset"]);
    }

    #[test]
    fn card_key_is_the_oracle_id_across_printings() {
        let card = test_support::card("Krenko");
        let mut reprint = card.clone();
        reprint.scryfall_data.id = Uuid::new_v4();
        assert_eq!(card_key(&card), card_key(&reprint));
        assert_eq!(Some(card_key(&card)), card.scryfall_data.oracle_id);
    }

    #[test]
    fn card_key_falls_back_to_the_scryfall_id() {
        let mut card = test_support::card("Krenko");
        card.scryfall_data.oracle_id = None;
        assert_eq!(card_key(&card), card.scryfall_data.id);
    }

    #[test]
    fn redeal_by_card_key_drops_a_swiped_card_reprinted_in_the_new_pile() {
        let krenko = test_support::card("Krenko");
        let narset = test_support::card("Narset");
        let mut krenko_reprint = krenko.clone();
        krenko_reprint.scryfall_data.id = Uuid::new_v4();
        let pile = redeal(
            &[krenko.clone(), narset.clone()],
            1,
            &[krenko_reprint, narset.clone()],
            card_key,
        );
        let ids: Vec<Uuid> = pile.iter().map(|c| c.scryfall_data.id).collect();
        assert_eq!(ids, vec![krenko.scryfall_data.id, narset.scryfall_data.id]);
    }
}
