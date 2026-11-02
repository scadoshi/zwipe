//! The deck view's rules: which sections show, the Tags badge, the Budget section's total, the share link, the clone dialog's prefill, and the toasts its actions raise.

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckProfile, Format};
use zwipe_core::domain::site::WEB_BASE;

use crate::deck::command_zone::Slot;
use crate::toast::{Toast, ToastKind};

/// Whether the deck view shows its `Budget` section: the deck has a price target, or it has metrics, which carry the running prices.
#[must_use]
pub fn has_budget(profile: &DeckProfile, has_metrics: bool) -> bool {
    profile.price_target.is_some() || has_metrics
}

/// The number on the `Tags` section's badge: deck, oracle and other tags together. The section shows only when this is above zero.
#[must_use]
pub fn tag_count(profile: &DeckProfile) -> usize {
    profile
        .tags
        .len()
        .saturating_add(profile.oracle_tags.len())
        .saturating_add(profile.other_tags.len())
}

/// Whether the deck can have stats once it loads: a card in it, or any command-zone card. While the deck loads, a skeleton stands in for the stats only when this holds.
#[must_use]
pub fn metrics_possible(profile: &DeckProfile) -> bool {
    profile.card_count > 0
        || profile.commander_id.is_some()
        || profile.partner_commander_id.is_some()
        || profile.background_id.is_some()
        || profile.signature_spell_id.is_some()
}

/// The public link to a shared deck: `{WEB_BASE}/deck/{token}`. Debug builds point at the local site, release builds at zwipe.net.
#[must_use]
pub fn share_link(token: Uuid) -> String {
    format!("{WEB_BASE}/deck/{token}")
}

/// The `Total price` row's value: the running total in `currency`, then ` / ` and the price target in its own stored currency when the deck has one.
///
/// A deck with nothing priced reads as the currency's zero, since the total is a sum. The target is never converted: each currency reads its own stored figure, and a target with no stored currency reads as dollars.
#[must_use]
pub fn total_price_line(
    profile: &DeckProfile,
    total: Option<f64>,
    currency: PriceCurrency,
) -> String {
    let running = currency.format_amount(total.unwrap_or(0.0));
    match profile.price_target {
        Some(target) => {
            let stored = profile.price_target_currency.unwrap_or_default();
            format!("{running} / {}", stored.format_amount(target))
        }
        None => running,
    }
}

/// The clone dialog's prefilled name: `{name} (clone)`.
#[must_use]
pub fn clone_name(deck_name: &str) -> String {
    format!("{deck_name} (clone)")
}

/// An action on the deck view that went through, for its toast. Failures are not here: each client maps the error to its message.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Done {
    /// The deck was cloned under this name, as the user typed it, trimmed.
    Cloned(String),
    /// A share link was copied to the clipboard, after Share or Copy link.
    LinkCopied,
    /// Stop share disabled the link.
    LinkDisabled,
    /// Clear skips went through.
    SkipsCleared,
    /// A warning's fix set a card's quantity to this many.
    QuantitySet(i32),
    /// A warning's fix cleared the commander of a deck in this format: "Oathbreaker" where the format has a signature spell.
    CommanderCleared(Option<Format>),
    /// A warning's fix took the card out of the deck.
    CardRemoved,
}

impl Done {
    /// The toast to raise. Every one is info, up for [`QUICK`](crate::toast::QUICK).
    #[must_use]
    pub fn toast(&self) -> Toast {
        let text = match self {
            Self::Cloned(name) => format!("Cloned as \"{name}\""),
            Self::LinkCopied => "Share link copied".to_owned(),
            Self::LinkDisabled => "Link disabled".to_owned(),
            Self::SkipsCleared => "Skips cleared".to_owned(),
            Self::QuantitySet(quantity) => format!("Quantity set to {quantity}"),
            Self::CommanderCleared(format) => {
                format!("{} cleared", Slot::Commander.label(*format))
            }
            Self::CardRemoved => "Card removed".to_owned(),
        };
        Toast::quick(ToastKind::Info, text)
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::DeckOtherTag;

    use super::*;
    use crate::test_support::deck_profile;
    use crate::toast::QUICK;

    #[test]
    fn budget_shows_for_a_target_or_metrics() {
        let mut profile = deck_profile("Budget");
        assert!(!has_budget(&profile, false));
        assert!(has_budget(&profile, true));
        profile.price_target = Some(50.0);
        assert!(has_budget(&profile, false));
    }

    #[test]
    fn the_tag_badge_counts_all_three_lists() {
        let mut profile = deck_profile("Tags");
        assert_eq!(tag_count(&profile), 0);
        profile.tags = vec!["burn".to_owned(), "aggro".to_owned()];
        profile.oracle_tags = vec!["ramp".to_owned()];
        profile.other_tags = vec![DeckOtherTag::Jank];
        assert_eq!(tag_count(&profile), 4);
    }

    #[test]
    fn metrics_are_possible_with_a_card_or_any_command_zone_card() {
        let profile = deck_profile("Empty");
        assert!(!metrics_possible(&profile));

        let mut cards = profile.clone();
        cards.card_count = 1;
        assert!(metrics_possible(&cards));

        let id = Some(Uuid::new_v4());
        let mut commander = profile.clone();
        commander.commander_id = id;
        let mut partner = profile.clone();
        partner.partner_commander_id = id;
        let mut background = profile.clone();
        background.background_id = id;
        let mut spell = profile;
        spell.signature_spell_id = id;
        for slot in [commander, partner, background, spell] {
            assert!(metrics_possible(&slot), "{slot:?}");
        }
    }

    #[test]
    fn the_target_keeps_its_own_currency() {
        let mut profile = deck_profile("Budget");
        assert_eq!(
            total_price_line(&profile, None, PriceCurrency::Eur),
            "\u{20ac}0.00"
        );
        profile.price_target = Some(100.0);
        assert_eq!(
            total_price_line(&profile, Some(12.5), PriceCurrency::Eur),
            "\u{20ac}12.50 / $100.00"
        );
        profile.price_target_currency = Some(PriceCurrency::Tix);
        assert_eq!(
            total_price_line(&profile, Some(3.0), PriceCurrency::Usd),
            "$3.00 / 100.00 TIX"
        );
    }

    #[test]
    fn the_share_link_ends_in_the_token() {
        let token = Uuid::nil();
        assert_eq!(
            share_link(token),
            format!("{WEB_BASE}/deck/00000000-0000-0000-0000-000000000000")
        );
    }

    #[test]
    fn the_clone_prefill_appends_clone() {
        assert_eq!(clone_name("Elves"), "Elves (clone)");
    }

    #[test]
    fn each_toast_has_the_apps_words_kind_and_band() {
        let cases = [
            (
                Done::Cloned("Elves (clone)".to_owned()),
                "Cloned as \"Elves (clone)\"",
            ),
            (Done::LinkCopied, "Share link copied"),
            (Done::LinkDisabled, "Link disabled"),
            (Done::SkipsCleared, "Skips cleared"),
            (Done::QuantitySet(1), "Quantity set to 1"),
            (
                Done::CommanderCleared(Some(Format::Commander)),
                "Commander cleared",
            ),
            (Done::CommanderCleared(None), "Commander cleared"),
            (
                Done::CommanderCleared(Some(Format::Oathbreaker)),
                "Oathbreaker cleared",
            ),
            (Done::CardRemoved, "Card removed"),
        ];
        for (done, text) in cases {
            assert_eq!(
                done.toast(),
                Toast::new(ToastKind::Info, text, QUICK),
                "{done:?}"
            );
        }
    }
}
