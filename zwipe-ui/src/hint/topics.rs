//! The `?` topics: what a row, a field or a chart is for, opened on demand
//! and never recorded.
//!
//! The copy is the app's, word for word.

use zwipe_core::domain::auth::models::password;
use zwipe_core::domain::deck::MAX_DECK_TAGS;

use super::{Block, HintContent, KeyIcon, Role, Span, bullet, emph, key, key_in, line, text};

/// A `?` beside a row, a field label or a chart.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum HintTopic {
    /// The password field's rules.
    Password,
    /// The email row.
    Email,
    /// The email verification row.
    Verification,
    /// The theme picker.
    Theme,
    /// How mana symbols are colored.
    Pips,
    /// The app icon picker, which only the app has.
    AppIcon,
    /// The Universes Beyond setting.
    UniversesBeyond,
    /// The website link.
    Website,
    /// The version row.
    Version,
    /// A deck's tags.
    DeckTags,
    /// A deck's oracle tags.
    OracleTags,
    /// A deck's name.
    DeckName,
    /// A deck's format.
    Format,
    /// The commander field.
    Commander,
    /// The partner field.
    Partner,
    /// The background field.
    Background,
    /// The signature spell field.
    SignatureSpell,
    /// The power level field.
    PowerLevel,
    /// The land target field.
    LandTarget,
    /// The price target field.
    PriceTarget,
    /// A deck's other tags.
    OtherTags,
    /// The deck view's Types chart.
    TypeChart,
    /// The deck view's Roles chart.
    RoleChart,
    /// The deck view's Colors chart.
    ColorChart,
    /// The deck view's mana curve.
    ManaCurve,
    /// The deck view's mana cost fulfillment.
    ManaFulfillment,
    /// The deck view's draw odds.
    DrawOdds,
    /// Home's color map.
    ColorMap,
    /// Playstyle's Card roles chart.
    SuggestionRoleChart,
    /// Playstyle's Oracle tags chart.
    SuggestionTagChart,
    /// The Decks column of Playstyle's tag table.
    SuggestionTagDecks,
    /// The Cards column of Playstyle's tag table.
    SuggestionTagCards,
    /// The Impact column of Playstyle's tag table.
    SuggestionTagImpact,
    /// Playstyle's suggested commanders.
    SuggestionList,
    /// Playstyle's Staples.
    SuggestionStaples,
    /// Activity's heatmap and totals.
    Activity,
    /// Activity's Card roles chart.
    RoleRates,
    /// Activity's Oracle tags chart.
    TagRates,
    /// Activity's Most skipped list.
    HardPasses,
}

impl HintTopic {
    /// Every topic, in the app's order.
    pub const ALL: [Self; 39] = [
        Self::Password,
        Self::Email,
        Self::Verification,
        Self::Theme,
        Self::Pips,
        Self::AppIcon,
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
    #[must_use]
    pub fn title(self) -> &'static str {
        match self {
            Self::Password => "Password",
            Self::Email => "Email",
            Self::Verification => "Verification",
            Self::Theme => "Theme",
            Self::Pips => "Pips",
            Self::AppIcon => "App icon",
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

    /// The dialog's body.
    #[must_use]
    #[expect(clippy::too_many_lines, reason = "a lookup table, one arm per topic")]
    pub fn blocks(self) -> Vec<Block> {
        match self {
            Self::Password => password::requirements()
                .iter()
                .map(|r| bullet(vec![text(r)]))
                .collect(),
            Self::Theme => vec![
                bullet(vec![text("The app's palette. Picks preview as you tap")]),
                bullet(vec![text("The last four are color blind modes")]),
                bullet(vec![
                    mode_key("light", KeyIcon::Light),
                    text(" "),
                    mode_key("dark", KeyIcon::Dark),
                    text(" switches between the theme's two palettes"),
                ]),
            ],
            Self::AppIcon => vec![
                bullet(vec![text(
                    "The icon on your home screen, in any theme's colors",
                )]),
                bullet(vec![
                    key("Match theme"),
                    text(" keeps it in your theme as you change it"),
                ]),
            ],
            Self::DeckTags => vec![
                bullet(vec![
                    text("The "),
                    emph("archetypes", Role::Secondary),
                    text(" the deck is built around, like Aristocrats or Voltron"),
                ]),
                bullet(vec![
                    text("Picking one pre-selects the "),
                    emph("oracle tags", Role::Tertiary),
                    text(" that define it"),
                ]),
                bullet(vec![
                    text("Up to "),
                    emph(&MAX_DECK_TAGS.to_string(), Role::Primary),
                ]),
            ],
            Self::Email => vec![line(vec![text(
                "Where password resets and verification go",
            )])],
            Self::Verification => vec![line(vec![
                text("Confirms the email is yours. "),
                key("Resend"),
                text(" if it never came"),
            ])],
            Self::Pips => vec![
                bullet(vec![
                    key("Theme"),
                    text(" draws mana symbols in your theme's colors"),
                ]),
                bullet(vec![
                    key("Natural"),
                    text(" keeps Magic's classic colors on every theme"),
                ]),
            ],
            Self::UniversesBeyond => vec![
                bullet(vec![
                    key("Hide"),
                    text(" keeps crossover cards out of searches and commander picks"),
                ]),
                bullet(vec![text("Exceptions keeps chosen franchises showing")]),
            ],
            Self::Website => vec![line(vec![text(
                "Guides, the changelog and news, in your browser",
            )])],
            Self::Version => vec![line(vec![
                text("The build you are on. "),
                key("Changelog"),
                text(" lists what changed in each release"),
            ])],
            Self::OracleTags => vec![
                bullet(vec![
                    text("The "),
                    emph("specific", Role::Tertiary),
                    text(" things the deck does, like spot removal, ramp or reanimation"),
                ]),
                bullet(vec![
                    text("They "),
                    emph("shape which cards we suggest", Role::Secondary),
                ]),
                bullet(vec![
                    text("Your "),
                    emph("deck tags", Role::Primary),
                    text(" pre-pick a starter set from about 4,500. Fine to leave as is"),
                ]),
            ],
            Self::DeckName => vec![line(vec![text(
                "Shown in your deck list. Change it any time",
            )])],
            Self::Format => vec![
                bullet(vec![text("Sets deck size, card pool and the command zone")]),
                bullet(vec![text(
                    "Changing it clears the commander and signature spell",
                )]),
            ],
            Self::Commander => vec![
                bullet(vec![
                    text("Search only shows cards that can lead this format. "),
                    key_in("Filter", Role::Warning),
                    text(" searches any card"),
                ]),
                bullet(vec![key("Swipe"), text(" picks one by swiping")]),
            ],
            Self::Partner => vec![line(vec![text(
                "Shows when the commander allows a partner",
            )])],
            Self::Background => vec![line(vec![text(
                "Shows when the commander can take a background",
            )])],
            Self::SignatureSpell => vec![line(vec![text(
                "Oathbreaker only: the instant or sorcery that starts in the command zone",
            )])],
            Self::PowerLevel => vec![line(vec![text(
                "The Commander bracket the deck aims for, 1 to 5",
            )])],
            Self::LandTarget => vec![
                bullet(vec![text(
                    "How many lands the deck should run. Fewer warns on the deck",
                )]),
                bullet(vec![text("Not set uses the format's usual count")]),
            ],
            Self::PriceTarget => vec![line(vec![text(
                "A budget for the mainboard. Going over warns on the deck",
            )])],
            Self::OtherTags => vec![line(vec![text(
                "Labels that do not change suggestions, like Budget or Jank",
            )])],
            Self::TypeChart => vec![line(vec![text("Mainboard cards by type")])],
            Self::RoleChart => vec![line(vec![text(
                "Mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags",
            )])],
            Self::ColorChart => vec![line(vec![text("Mainboard cards by color")])],
            Self::ManaCurve => vec![line(vec![text("Nonland mainboard cards by mana value")])],
            Self::ManaFulfillment => vec![
                bullet(vec![text(
                    "Per color, the pips your lands and sources make against the pips your spells need",
                )]),
                bullet(vec![text(
                    "A check means you make at least as much as you need",
                )]),
            ],
            Self::DrawOdds => vec![
                bullet(vec![
                    text(
                        "Chance of at least one card from each group in the opening hand, or by a turn. ",
                    ),
                    key("-"),
                    text(" and "),
                    key("+"),
                    text(" step the turn"),
                ]),
                bullet(vec![text(
                    "Random draw only. Mulligans, tutors and card draw are not counted",
                )]),
            ],
            Self::ColorMap => vec![
                bullet(vec![text(
                    "Each tile stands for a share of the colored cards across your decks",
                )]),
                bullet(vec![text(
                    "Every color grows out from the middle, and the color with the most cards claims the most ground",
                )]),
                bullet(vec![text("The shape is generated per player")]),
                bullet(vec![text("Colorless cards are in the key below")]),
            ],
            Self::SuggestionRoleChart => vec![line(vec![text(
                "How many of your decks have a tag in each role. Tap a role for its tags",
            )])],
            Self::SuggestionTagChart => vec![line(vec![text(
                "Your 10 most impactful tags, highest first. Tap a tag to see what it means",
            )])],
            Self::SuggestionTagDecks => vec![line(vec![text(
                "How many of your decks have this tag. The more decks, the higher its impact",
            )])],
            Self::SuggestionTagCards => vec![line(vec![text(
                "How many cards carry this tag across those decks. The more cards, the higher its impact",
            )])],
            Self::SuggestionTagImpact => vec![line(vec![text(
                "How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100",
            )])],
            Self::SuggestionList => vec![
                bullet(vec![text(
                    "Commanders you haven't built, ranked against all your decks, best fit first",
                )]),
                bullet(vec![
                    text("Tap a row to see why it fits. "),
                    key("Swipe"),
                    text(" goes further down the same ranking"),
                ]),
            ],
            Self::SuggestionStaples => vec![
                bullet(vec![text(
                    "The cards in the most of your decks, with the share of your decks that run each",
                )]),
                bullet(vec![text(
                    "Basic lands are left out. Tap a Type or Color chip to narrow it",
                )]),
            ],
            Self::Activity => vec![
                bullet(vec![text(
                    "Your swipes and searches each day over the last year, a darker square for a busier day",
                )]),
                bullet(vec![text(
                    "A lit square is one of your biggest days. Tap a square to see its counts",
                )]),
                bullet(vec![
                    key("Totals"),
                    text(" is what the cards dealt to your decks came to: "),
                    key("Added"),
                    text(" and "),
                    key("Maybeboard"),
                    text(" you kept, "),
                    key("Skipped"),
                    text(" you passed on, and "),
                    key("Removed"),
                    text(" you later took out"),
                ]),
            ],
            Self::RoleRates => vec![
                bullet(vec![
                    text("Your share for each card role. Pick one under "),
                    key("Show"),
                    text(": "),
                    key("Added"),
                    text(", "),
                    key("Skipped"),
                    text(" and "),
                    key("Maybeboard"),
                    text(" are shares of the cards dealt to you, and "),
                    key("Removed"),
                    text(" is the share of added cards you later took out"),
                ]),
                bullet(vec![text(
                    "Tap a role to see the tags you swiped under it. A grey role has none yet",
                )]),
                bullet(vec![text("From your decks with a commander")]),
            ],
            Self::TagRates => vec![
                bullet(vec![text(
                    "The tags where your share stands out most from other players', with your own share",
                )]),
                bullet(vec![text(
                    "A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means",
                )]),
                bullet(vec![text(
                    "From your decks with a commander, for tags dealt at least 10 times",
                )]),
            ],
            Self::HardPasses => vec![
                bullet(vec![text(
                    "Cards you skip and never add or save, when other players usually keep them",
                )]),
                bullet(vec![text(
                    "A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under",
                )]),
                bullet(vec![text("From your decks with a commander")]),
            ],
        }
    }

    /// The topic as a dialog's content.
    #[must_use]
    pub fn content(self) -> HintContent {
        HintContent {
            title: self.title().to_owned(),
            blocks: self.blocks(),
        }
    }
}

/// A light or dark mode key chip: its icon, then its label.
fn mode_key(label: &str, icon: KeyIcon) -> Span {
    Span::Key {
        label: label.to_owned(),
        role: Role::default(),
        icon: Some(icon),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

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
            blocks.first().map(Block::plain).as_deref(),
            Some("8 to 128 characters")
        );
        assert_eq!(
            blocks.last().map(Block::plain).as_deref(),
            Some("No character more than 3 times in a row")
        );
        assert_eq!(blocks.len(), 8);
        assert!(blocks.iter().all(|b| matches!(b, Block::Bullet(_))));
    }

    #[test]
    fn the_deck_tags_topic_names_the_cap() {
        let last = HintTopic::DeckTags.blocks().pop().unwrap();
        assert_eq!(last.plain(), format!("Up to {MAX_DECK_TAGS}"));
        assert_eq!(
            HintTopic::Version.blocks()[0],
            line(vec![
                text("The build you are on. "),
                key("Changelog"),
                text(" lists what changed in each release"),
            ])
        );
    }

    #[test]
    fn the_rate_topics_speak_of_shares_as_the_app_does() {
        let roles = HintTopic::RoleRates.content().plain();
        assert!(
            roles.starts_with("Your share for each card role."),
            "{roles}"
        );
        assert!(roles.contains("A grey role has none yet"), "{roles}");
        assert!(!roles.contains("dealt at least 10 times"), "{roles}");
        let tags = HintTopic::TagRates.content().plain();
        assert!(tags.contains("with your own share"), "{tags}");
        assert!(!tags.contains("rate"), "{tags}");
    }

    #[test]
    fn the_theme_topic_draws_its_mode_icons_in_the_chips() {
        let blocks = HintTopic::Theme.blocks();
        let spans = blocks[2].spans();
        assert_eq!(spans[0], mode_key("light", KeyIcon::Light));
        assert_eq!(spans[2], mode_key("dark", KeyIcon::Dark));
        assert_eq!(HintTopic::AppIcon.title(), "App icon");
    }
}
