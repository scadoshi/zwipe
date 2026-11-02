//! Which zwipe.net guide goes with each hint, for the "See guide ↗" link a hint dialog offers.
//!
//! A hint answers the quick question; its guide on zwipe.net answers the long one. Every client links the same hint to the same guide through [`HintTopic::guide_slug`] and [`one_time_slug`], and builds the address with [`url`]. A hint with no guide gets no link. Both matches are exhaustive, so a new hint has to say whether it has a guide.
//!
//! Only screen hints link out: the one-time hint a screen opens with and the `?` in a screen's header (the format, deck tag and oracle tag pickers), while a panel's or field's `?` inside a screen shows no link even when its topic has a slug here.

use zwipe_core::domain::user::hints::OneTimeHint;

use super::HintTopic;

/// Where zite serves its guides; a slug goes on the end.
pub const GUIDE_BASE: &str = "https://zwipe.net/guides/";

/// The address of the guide named `slug`.
#[must_use]
pub fn url(slug: &str) -> String {
    format!("{GUIDE_BASE}{slug}")
}

impl HintTopic {
    /// The slug of the guide that goes deeper on this `?` topic, or `None` when no guide covers it.
    #[must_use]
    pub const fn guide_slug(self) -> Option<&'static str> {
        match self {
            Self::DeckTags | Self::OtherTags => Some("deck-tags"),
            Self::OracleTags => Some("oracle-tags"),
            Self::Format
            | Self::Commander
            | Self::Partner
            | Self::Background
            | Self::SignatureSpell => Some("commander-and-formats"),
            Self::LandTarget => Some("land-targets"),
            Self::PriceTarget => Some("budgeting"),
            Self::TypeChart
            | Self::RoleChart
            | Self::ColorChart
            | Self::ManaCurve
            | Self::ManaFulfillment
            | Self::DrawOdds => Some("deck-stats"),
            Self::ColorMap
            | Self::SuggestionRoleChart
            | Self::SuggestionTagChart
            | Self::SuggestionTagDecks
            | Self::SuggestionTagCards
            | Self::SuggestionTagImpact
            | Self::SuggestionList
            | Self::SuggestionStaples => Some("playstyle"),
            Self::Activity | Self::RoleRates | Self::TagRates | Self::HardPasses => {
                Some("activity")
            }
            Self::Password
            | Self::Email
            | Self::Verification
            | Self::Theme
            | Self::Pips
            | Self::AppIcon
            | Self::UniversesBeyond
            | Self::Website
            | Self::Version
            | Self::DeckName
            | Self::PowerLevel => None,
        }
    }
}

/// The slug of the guide that goes deeper on a one-time hint, or `None` when no guide covers it.
#[must_use]
pub const fn one_time_slug(hint: OneTimeHint) -> Option<&'static str> {
    match hint {
        OneTimeHint::FirstLogin => Some("getting-started"),
        OneTimeHint::DeckList | OneTimeHint::DeckCards => Some("organize-and-browse"),
        OneTimeHint::CreateDeck | OneTimeHint::EditDeck => Some("commander-and-formats"),
        OneTimeHint::FirstDeck => Some("deck-stats"),
        OneTimeHint::AddDeckCards | OneTimeHint::SwipeSelect => Some("swipe-to-build"),
        OneTimeHint::RemoveDeckCards => Some("remove-cards"),
        OneTimeHint::Filter => Some("filtering"),
        OneTimeHint::Import | OneTimeHint::Export => Some("import-export"),
        OneTimeHint::OracleTagDictionary | OneTimeHint::OtagExamples => {
            Some("oracle-tag-dictionary")
        }
        OneTimeHint::CommanderMaybeboard => Some("commander-maybeboard"),
        OneTimeHint::SuggestedCommanders => Some("playstyle"),
        OneTimeHint::Activity => Some("activity"),
        OneTimeHint::Profile | OneTimeHint::Config | OneTimeHint::UniversesBeyondExceptions => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// zite's guide pages, read from its source so a renamed or dropped guide fails here.
    const ZITE_GUIDES: &str = include_str!("../../../zite/src/pages/guides/content.rs");

    fn zite_slugs() -> Vec<&'static str> {
        ZITE_GUIDES
            .split("slug: \"")
            .skip(1)
            .filter_map(|rest| rest.split('"').next())
            .collect()
    }

    const TOPICS: [HintTopic; 39] = [
        HintTopic::Password,
        HintTopic::Email,
        HintTopic::Verification,
        HintTopic::Theme,
        HintTopic::Pips,
        HintTopic::AppIcon,
        HintTopic::UniversesBeyond,
        HintTopic::Website,
        HintTopic::Version,
        HintTopic::DeckTags,
        HintTopic::OracleTags,
        HintTopic::DeckName,
        HintTopic::Format,
        HintTopic::Commander,
        HintTopic::Partner,
        HintTopic::Background,
        HintTopic::SignatureSpell,
        HintTopic::PowerLevel,
        HintTopic::LandTarget,
        HintTopic::PriceTarget,
        HintTopic::OtherTags,
        HintTopic::TypeChart,
        HintTopic::RoleChart,
        HintTopic::ColorChart,
        HintTopic::ManaCurve,
        HintTopic::ManaFulfillment,
        HintTopic::DrawOdds,
        HintTopic::ColorMap,
        HintTopic::SuggestionRoleChart,
        HintTopic::SuggestionTagChart,
        HintTopic::SuggestionTagDecks,
        HintTopic::SuggestionTagCards,
        HintTopic::SuggestionTagImpact,
        HintTopic::SuggestionList,
        HintTopic::SuggestionStaples,
        HintTopic::Activity,
        HintTopic::RoleRates,
        HintTopic::TagRates,
        HintTopic::HardPasses,
    ];

    const ONE_TIME: [OneTimeHint; 20] = [
        OneTimeHint::FirstLogin,
        OneTimeHint::Profile,
        OneTimeHint::Config,
        OneTimeHint::UniversesBeyondExceptions,
        OneTimeHint::DeckList,
        OneTimeHint::CreateDeck,
        OneTimeHint::EditDeck,
        OneTimeHint::FirstDeck,
        OneTimeHint::DeckCards,
        OneTimeHint::AddDeckCards,
        OneTimeHint::RemoveDeckCards,
        OneTimeHint::Filter,
        OneTimeHint::Import,
        OneTimeHint::Export,
        OneTimeHint::OracleTagDictionary,
        OneTimeHint::OtagExamples,
        OneTimeHint::CommanderMaybeboard,
        OneTimeHint::SuggestedCommanders,
        OneTimeHint::Activity,
        OneTimeHint::SwipeSelect,
    ];

    #[test]
    fn zite_lists_its_guides() {
        let slugs = zite_slugs();
        assert_eq!(slugs.len(), 22, "zite's guide count: {slugs:?}");
        assert!(slugs.contains(&"getting-started"));
    }

    #[test]
    fn every_mapped_slug_is_a_zite_guide() {
        let slugs = zite_slugs();
        let mapped = TOPICS
            .iter()
            .filter_map(|t| t.guide_slug())
            .chain(ONE_TIME.iter().filter_map(|h| one_time_slug(*h)));
        for slug in mapped {
            assert!(slugs.contains(&slug), "{slug} is not one of zite's guides");
        }
    }

    #[test]
    fn hints_without_a_guide_get_no_link() {
        for topic in [
            HintTopic::Password,
            HintTopic::Theme,
            HintTopic::Version,
            HintTopic::PowerLevel,
        ] {
            assert_eq!(topic.guide_slug(), None, "{topic:?}");
        }
        for hint in [
            OneTimeHint::Profile,
            OneTimeHint::Config,
            OneTimeHint::UniversesBeyondExceptions,
        ] {
            assert_eq!(one_time_slug(hint), None, "{hint:?}");
        }
    }

    #[test]
    fn related_hints_share_a_guide() {
        assert_eq!(HintTopic::OtherTags.guide_slug(), Some("deck-tags"));
        assert_eq!(
            HintTopic::Partner.guide_slug(),
            Some("commander-and-formats")
        );
        assert_eq!(HintTopic::DrawOdds.guide_slug(), Some("deck-stats"));
        assert_eq!(HintTopic::SuggestionStaples.guide_slug(), Some("playstyle"));
        assert_eq!(one_time_slug(OneTimeHint::Export), Some("import-export"));
    }

    #[test]
    fn the_url_is_the_base_and_the_slug() {
        assert_eq!(url("deck-stats"), "https://zwipe.net/guides/deck-stats");
    }
}
