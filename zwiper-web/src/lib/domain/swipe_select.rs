//! Swipe select's re-deal: the suggested pile dealt again from a new
//! ranking mid-session, keeping what was already swiped in front.

// Belongs in zwipe-ui: the app's `swipe_select.rs` keeps `redeal` and
// `card_key` beside its Dioxus overlay, and zwipe-ui has no copy yet.

use uuid::Uuid;
use zwipe_core::domain::card::Card;

/// A suggested card's identity across printings: its oracle id, else its
/// scryfall id.
pub fn card_key(card: &Card) -> Uuid {
    card.scryfall_data
        .oracle_id
        .unwrap_or(card.scryfall_data.id)
}

/// The pile re-dealt from a new ranking mid-session. The first `swiped` of
/// `dealt` stay in front, so the top card's index and the undo history still
/// line up with them; then every item of `fresh` whose key is not among them,
/// in `fresh`'s order.
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
        let dealt = ["Krenko", "Narset", "Zada"];
        let fresh = ["Zada", "Talrand"];
        assert_eq!(
            redeal(&dealt, 2, &fresh, key),
            vec!["Krenko", "Narset", "Zada", "Talrand"]
        );
    }
}
