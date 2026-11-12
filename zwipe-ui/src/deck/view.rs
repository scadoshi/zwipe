//! The deck view's rules: a phone's two areas ([`DeckArea`]), which sections show, the Tags badge, the Budget section's total, the share link, the clone dialog's prefill, and the toasts its actions raise.

use uuid::Uuid;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckOtherTag, DeckProfile, Format};
use zwipe_core::domain::share_link;
use zwipe_core::http::contracts::deck::HttpSharedDeck;

use crate::deck::command_zone::Slot;
use crate::toast::{Toast, ToastKind};

/// The two areas a deck's page splits into on a phone, under its head: its profile (the stat
/// panels and the page's foot) or its cards (the controls and the groups). A wide window shows
/// both.
#[derive(Debug, Clone, Copy, Default, PartialEq, Eq, Hash)]
pub enum DeckArea {
    /// The stat panels and the foot, which a deck opens on, as the app does.
    #[default]
    Profile,
    /// The controls and the card groups.
    Cards,
}

impl DeckArea {
    /// Both areas, in the switch's order.
    pub const ALL: [Self; 2] = [Self::Profile, Self::Cards];

    /// The switch's label.
    #[must_use]
    pub const fn label(self) -> &'static str {
        match self {
            Self::Profile => "Profile",
            Self::Cards => "Cards",
        }
    }

    /// The page's class while this area shows, which the stylesheet reads on a phone.
    #[must_use]
    pub const fn class(self) -> &'static str {
        match self {
            Self::Profile => "area-profile",
            Self::Cards => "area-cards",
        }
    }
}

/// What the deck's stat panels read from its profile: its tags and its targets. An owner's deck
/// and a shared deck both give one, so the panels draw the same for either.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct StatsProfile {
    /// The deck's own tags, as stored.
    pub tags: Vec<String>,
    /// The deck's Oracle tag slugs.
    pub oracle_tags: Vec<String>,
    /// The deck's other tags.
    pub other_tags: Vec<DeckOtherTag>,
    /// How many lands the owner aims for.
    pub land_target: Option<i32>,
    /// The price the owner aims to stay under, in `price_target_currency`.
    pub price_target: Option<f64>,
    /// The currency `price_target` is in; dollars when none is stored.
    pub price_target_currency: Option<PriceCurrency>,
}

impl From<&DeckProfile> for StatsProfile {
    fn from(profile: &DeckProfile) -> Self {
        Self {
            tags: profile.tags.clone(),
            oracle_tags: profile.oracle_tags.clone(),
            other_tags: profile.other_tags.clone(),
            land_target: profile.land_target,
            price_target: profile.price_target,
            price_target_currency: profile.price_target_currency,
        }
    }
}

impl From<&HttpSharedDeck> for StatsProfile {
    fn from(deck: &HttpSharedDeck) -> Self {
        Self {
            tags: deck.tags.clone(),
            oracle_tags: deck.oracle_tags.clone(),
            other_tags: deck.other_tags.clone(),
            land_target: deck.land_target,
            price_target: deck.price_target,
            price_target_currency: deck.price_target_currency,
        }
    }
}

impl StatsProfile {
    /// Whether the `Budget` panel shows: the deck has a price target, or it has metrics, which
    /// carry the running prices.
    #[must_use]
    pub fn has_budget(&self, has_metrics: bool) -> bool {
        self.price_target.is_some() || has_metrics
    }

    /// The deck, oracle and other tags together; the `Tags` panel shows only when this is above
    /// zero.
    #[must_use]
    pub fn tag_count(&self) -> usize {
        self.tags
            .len()
            .saturating_add(self.oracle_tags.len())
            .saturating_add(self.other_tags.len())
    }

    /// The `Total price` row's value: the running total in `currency`, then ` / ` and the price
    /// target in its own stored currency when the deck has one. Nothing priced reads as the
    /// currency's zero, and the target is never converted.
    #[must_use]
    pub fn total_price_line(&self, total: Option<f64>, currency: PriceCurrency) -> String {
        let running = currency.format_amount(total.unwrap_or(0.0));
        match self.price_target {
            Some(target) => {
                let stored = self.price_target_currency.unwrap_or_default();
                format!("{running} / {}", stored.format_amount(target))
            }
            None => running,
        }
    }
}

/// Whether the deck view shows its `Budget` section: the deck has a price target, or it has metrics, which carry the running prices.
#[must_use]
pub fn has_budget(profile: &DeckProfile, has_metrics: bool) -> bool {
    StatsProfile::from(profile).has_budget(has_metrics)
}

/// The number on the `Tags` section's badge: deck, oracle and other tags together. The section shows only when this is above zero.
#[must_use]
pub fn tag_count(profile: &DeckProfile) -> usize {
    StatsProfile::from(profile).tag_count()
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

/// The public link to the deck `owner` shares as `deck_name` under `token`: `{WEB_BASE}/<owner>/deck/<slug>/<token>`, from [`share_link::shared_deck_url`]. Debug builds point at the local site, release builds at zwipe.net.
#[must_use]
pub fn share_link(owner: &str, deck_name: &str, token: Uuid) -> String {
    share_link::shared_deck_url(owner, deck_name, token)
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
    StatsProfile::from(profile).total_price_line(total, currency)
}

/// A price row's value with ` (N unpriced)` after it when `unpriced` copies have no price in the shown currency, which its sum and average leave out.
#[must_use]
pub fn with_unpriced(line: String, unpriced: usize) -> String {
    if unpriced == 0 {
        line
    } else {
        format!("{line} ({unpriced} unpriced)")
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
    /// Stop sharing disabled the link.
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
    use zwipe_core::domain::site::WEB_BASE;

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
    fn a_price_row_says_how_many_are_unpriced() {
        assert_eq!(with_unpriced("$412.30".to_owned(), 0), "$412.30");
        assert_eq!(
            with_unpriced("$412.30 / $500.00".to_owned(), 11),
            "$412.30 / $500.00 (11 unpriced)"
        );
    }

    #[test]
    fn the_share_link_names_the_owner_and_the_deck_then_the_token() {
        let token = Uuid::nil();
        assert_eq!(
            share_link("scotty", "Elves Go Wide", token),
            format!("{WEB_BASE}/scotty/deck/elves-go-wide/00000000-0000-0000-0000-000000000000")
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

    #[test]
    fn a_phone_opens_a_deck_on_its_profile_then_offers_its_cards() {
        assert_eq!(DeckArea::default(), DeckArea::Profile, "as the app opens a deck");
        assert_eq!(DeckArea::ALL, [DeckArea::Profile, DeckArea::Cards]);
        let labels: Vec<&str> = DeckArea::ALL.iter().map(|a| a.label()).collect();
        assert_eq!(labels, ["Profile", "Cards"]);
        let classes: Vec<&str> = DeckArea::ALL.iter().map(|a| a.class()).collect();
        assert_eq!(classes, ["area-profile", "area-cards"]);
    }
}
