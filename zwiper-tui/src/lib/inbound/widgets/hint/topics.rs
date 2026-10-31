//! The app's `?` topics: what a row or a field is for, opened on demand and never recorded.
//!
//! The copy is the app's, word for word, in the markup of [`super::markup`].

use zwipe_core::domain::auth::models::password;
use zwipe_core::domain::deck::MAX_DECK_TAGS;

use super::HintContent;

/// A `?` beside a row or a field label.
#[cfg_attr(
    not(test),
    expect(
        dead_code,
        reason = "the deck screens word their field and chart explainers in their own popups, and the Playstyle topics wait for that screen"
    )
)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HintTopic {
    Password,
    Email,
    Verification,
    Theme,
    Pips,
    UniversesBeyond,
    Website,
    Version,
    DeckTags,
    OracleTags,
    DeckName,
    Format,
    Commander,
    Partner,
    Background,
    SignatureSpell,
    PowerLevel,
    LandTarget,
    PriceTarget,
    OtherTags,
    TypeChart,
    RoleChart,
    ColorChart,
    ManaCurve,
    ManaFulfillment,
    DrawOdds,
    ColorMap,
    SuggestionRoleChart,
    SuggestionTagChart,
    SuggestionTagDecks,
    SuggestionTagCards,
    SuggestionTagImpact,
    SuggestionList,
    SuggestionStaples,
    Activity,
    RoleRates,
    TagRates,
    HardPasses,
}

impl HintTopic {
    /// Every topic, in the app's order.
    #[cfg(test)]
    pub(crate) const ALL: [Self; 38] = [
        Self::Password,
        Self::Email,
        Self::Verification,
        Self::Theme,
        Self::Pips,
        Self::UniversesBeyond,
        Self::Website,
        Self::Version,
        Self::DeckTags,
        Self::OracleTags,
        Self::DeckName,
        Self::Format,
        Self::Commander,
        Self::Partner,
        Self::Background,
        Self::SignatureSpell,
        Self::PowerLevel,
        Self::LandTarget,
        Self::PriceTarget,
        Self::OtherTags,
        Self::TypeChart,
        Self::RoleChart,
        Self::ColorChart,
        Self::ManaCurve,
        Self::ManaFulfillment,
        Self::DrawOdds,
        Self::ColorMap,
        Self::SuggestionRoleChart,
        Self::SuggestionTagChart,
        Self::SuggestionTagDecks,
        Self::SuggestionTagCards,
        Self::SuggestionTagImpact,
        Self::SuggestionList,
        Self::SuggestionStaples,
        Self::Activity,
        Self::RoleRates,
        Self::TagRates,
        Self::HardPasses,
    ];

    /// The dialog's title.
    pub(crate) fn title(self) -> &'static str {
        match self {
            Self::Password => "Password",
            Self::Email => "Email",
            Self::Verification => "Verification",
            Self::Theme => "Theme",
            Self::Pips => "Pips",
            Self::UniversesBeyond => "Universes Beyond",
            Self::Website => "Website",
            Self::Version => "Version",
            Self::DeckTags => "Deck tags",
            Self::OracleTags | Self::SuggestionTagChart | Self::TagRates => "Oracle tags",
            Self::DeckName => "Deck name",
            Self::Format => "Format",
            Self::Commander => "Commander",
            Self::Partner => "Partner",
            Self::Background => "Background",
            Self::SignatureSpell => "Signature spell",
            Self::PowerLevel => "Power level",
            Self::LandTarget => "Land target",
            Self::PriceTarget => "Price target",
            Self::OtherTags => "Other tags",
            Self::TypeChart => "Types",
            Self::RoleChart => "Roles",
            Self::ColorChart => "Colors",
            Self::ManaCurve => "Mana curve",
            Self::ManaFulfillment => "Mana cost fulfillment",
            Self::DrawOdds => "Draw odds",
            Self::ColorMap => "Color map",
            Self::SuggestionRoleChart | Self::RoleRates => "Card roles",
            Self::SuggestionTagDecks => "Decks",
            Self::SuggestionTagCards => "Cards",
            Self::SuggestionTagImpact => "Impact",
            Self::SuggestionList => "Suggested commanders",
            Self::SuggestionStaples => "Staples",
            Self::Activity => "Activity",
            Self::HardPasses => "Most skipped",
        }
    }

    /// The dialog's blocks, in hint markup.
    #[expect(clippy::too_many_lines, reason = "a lookup table, one arm per topic")]
    pub(crate) fn blocks(self) -> Vec<String> {
        let owned = |blocks: &[&str]| blocks.iter().map(|&b| b.to_owned()).collect();
        match self {
            Self::Password => password::requirements()
                .iter()
                .map(|r| format!("• {r}"))
                .collect(),
            Self::Email => owned(&["Where password resets and verification go"]),
            Self::Verification => {
                owned(&["Confirms the email is yours. [Resend] if it never came"])
            }
            Self::Theme => owned(&[
                "• The app's palette. Picks preview as you tap",
                "• The last four are color blind modes",
                "• [☀ light] [☾ dark] switches between the theme's two palettes",
            ]),
            Self::Pips => owned(&[
                "• [Theme] draws mana symbols in your theme's colors",
                "• [Natural] keeps Magic's classic colors on every theme",
            ]),
            Self::UniversesBeyond => owned(&[
                "• [Hide] keeps crossover cards out of searches and commander picks",
                "• Exceptions keeps chosen franchises showing",
            ]),
            Self::Website => owned(&["Guides, the changelog and news, in your browser"]),
            Self::Version => {
                owned(&["The build you are on. [Changelog] lists what changed in each release"])
            }
            Self::DeckTags => vec![
                "• The {archetypes|secondary} the deck is built around, like Aristocrats or Voltron"
                    .to_owned(),
                "• Picking one pre-selects the {oracle tags|tertiary} that define it".to_owned(),
                format!("• Up to {{{MAX_DECK_TAGS}|primary}}"),
            ],
            Self::OracleTags => owned(&[
                "• The {specific|tertiary} things the deck does, like spot removal, ramp or reanimation",
                "• They {shape which cards we suggest|secondary}",
                "• Your {deck tags|primary} pre-pick a starter set from about 4,500. Fine to leave as is",
            ]),
            Self::DeckName => owned(&["Shown in your deck list. Change it any time"]),
            Self::Format => owned(&[
                "• Sets deck size, card pool and the command zone",
                "• Changing it clears the commander and signature spell",
            ]),
            Self::Commander => owned(&[
                "• Search only shows cards that can lead this format. [Filter|warning] searches any card",
                "• [Swipe] picks one by swiping",
            ]),
            Self::Partner => owned(&["Shows when the commander allows a partner"]),
            Self::Background => owned(&["Shows when the commander can take a background"]),
            Self::SignatureSpell => {
                owned(&["Oathbreaker only: the instant or sorcery that starts in the command zone"])
            }
            Self::PowerLevel => owned(&["The Commander bracket the deck aims for, 1 to 5"]),
            Self::LandTarget => owned(&[
                "• How many lands the deck should run. Fewer warns on the deck",
                "• Not set uses the format's usual count",
            ]),
            Self::PriceTarget => {
                owned(&["A budget for the mainboard. Going over warns on the deck"])
            }
            Self::OtherTags => {
                owned(&["Labels that do not change suggestions, like Budget or Jank"])
            }
            Self::TypeChart => owned(&["Mainboard cards by type"]),
            Self::RoleChart => owned(&[
                "Mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags",
            ]),
            Self::ColorChart => owned(&["Mainboard cards by color"]),
            Self::ManaCurve => owned(&["Nonland mainboard cards by mana value"]),
            Self::ManaFulfillment => owned(&[
                "• Per color, the pips your lands and sources make against the pips your spells need",
                "• A check means you make at least as much as you need",
            ]),
            Self::DrawOdds => owned(&[
                "• Chance of at least one card from each group in the opening hand, or by a turn. [-] and [+] step the turn",
                "• Random draw only. Mulligans, tutors and card draw are not counted",
            ]),
            Self::ColorMap => owned(&[
                "• Each tile stands for a share of the colored cards across your decks",
                "• Every color grows out from the middle, and the color with the most cards claims the most ground",
                "• The shape is generated per player",
                "• Colorless cards are in the key below",
            ]),
            Self::SuggestionRoleChart => {
                owned(&["How many of your decks have a tag in each role. Tap a role for its tags"])
            }
            Self::SuggestionTagChart => owned(&[
                "Your 10 most impactful tags, highest first. Tap a tag to see what it means",
            ]),
            Self::SuggestionTagDecks => owned(&[
                "How many of your decks have this tag. The more decks, the higher its impact",
            ]),
            Self::SuggestionTagCards => owned(&[
                "How many cards carry this tag across those decks. The more cards, the higher its impact",
            ]),
            Self::SuggestionTagImpact => owned(&[
                "How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100",
            ]),
            Self::SuggestionList => owned(&[
                "• Commanders you haven't built, ranked against all your decks, best fit first",
                "• Tap a row to see why it fits. [Swipe] goes further down the same ranking",
            ]),
            Self::SuggestionStaples => owned(&[
                "• The cards in the most of your decks, with the share of your decks that run each",
                "• Basic lands are left out. Tap a Type or Color chip to narrow it",
            ]),
            Self::Activity => owned(&[
                "• Your swipes and searches each day over the last year, a darker square for a busier day",
                "• A lit square is one of your biggest days. Tap a square to see its counts",
                "• [Totals] is what the cards dealt to your decks came to: [Added] and [Maybeboard] you kept, [Skipped] you passed on, and [Removed] you later took out",
            ]),
            Self::RoleRates => owned(&[
                "• Your share for each card role. Pick one under [Show]: [Added], [Skipped] and [Maybeboard] are shares of the cards dealt to you, and [Removed] is the share of added cards you later took out",
                "• Tap a role to see the tags you swiped under it. A grey role has none yet",
                "• From your decks with a commander",
            ]),
            Self::TagRates => owned(&[
                "• The tags where your share stands out most from other players', with your own share",
                "• A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means",
                "• From your decks with a commander, for tags dealt at least 10 times",
            ]),
            Self::HardPasses => owned(&[
                "• Cards you skip and never add or save, when other players usually keep them",
                "• A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under",
                "• From your decks with a commander",
            ]),
        }
    }

    /// The topic as a dialog's content.
    pub(crate) fn content(self) -> HintContent {
        HintContent {
            title: self.title().to_owned(),
            blocks: self.blocks(),
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::inbound::screens::activity::{Section, passes_topic};
    use crate::inbound::widgets::hint::markup::plain;
    use crate::inbound::widgets::info::InfoPopup;

    #[test]
    fn every_topic_has_a_title_and_copy() {
        for topic in HintTopic::ALL {
            assert!(!topic.title().is_empty(), "{topic:?}");
            assert!(!topic.blocks().is_empty(), "{topic:?}");
        }
    }

    #[test]
    fn the_password_topic_lists_the_core_requirements() {
        let blocks = HintTopic::Password.blocks();
        assert_eq!(
            blocks.first().map(String::as_str),
            Some("• 8 to 128 characters")
        );
        assert_eq!(
            blocks.last().map(String::as_str),
            Some("• No character more than 3 times in a row")
        );
        assert_eq!(blocks.len(), 8);
    }

    #[test]
    fn the_rate_topics_read_as_activity_shows_them() {
        let words = |text: &str| text.split_whitespace().collect::<String>();
        let topic = |topic: HintTopic| {
            topic
                .blocks()
                .iter()
                .map(|b| words(plain(b).trim_start_matches("• ")))
                .collect::<Vec<_>>()
        };
        let shown = |popup: InfoPopup| popup.bullets.iter().map(|b| words(b)).collect::<Vec<_>>();
        assert_eq!(topic(HintTopic::Activity), shown(Section::Activity.topic()));
        assert_eq!(topic(HintTopic::RoleRates), shown(Section::Roles.topic()));
        assert_eq!(topic(HintTopic::TagRates), shown(Section::Tags.topic()));
        assert_eq!(topic(HintTopic::HardPasses), shown(passes_topic()));
    }

    #[test]
    fn the_deck_tags_topic_names_the_cap() {
        let last = HintTopic::DeckTags.blocks().pop().unwrap();
        assert_eq!(plain(&last), format!("• Up to {MAX_DECK_TAGS}"));
        assert_eq!(
            plain(&HintTopic::Version.blocks()[0]),
            "The build you are on.  Changelog  lists what changed in each release"
        );
    }
}
