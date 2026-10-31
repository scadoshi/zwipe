//! App-root hint host: a sender/receiver for on-demand "?" help.
//!
//! An [`InfoButton`](crate::inbound::components::info_button::InfoButton) anywhere in the tree posts a
//! [`HintTopic`](crate::inbound::components::hint_host::HintTopic) to a context signal; this single [`HintHost`](crate::inbound::components::hint_host::HintHost), mounted at the app
//! root, hands the topic's dialog to orangui's dialog host. A single `Option`
//! means only one hint shows at a time.

use components::{AlertDialog, DialogAction, HintBullet, HintBullets, HintKey, HintLine, ModeIcon};
use dioxus::prelude::*;
use zwipe_core::domain::auth::password;

use crate::inbound::components::concept_explainers::{DeckTagsExplainer, OracleTagsExplainer};

/// A concept a "?" button can explain. Cheap `Copy` message on the hint channel.
///
/// One per field or row rather than one per screen: a dialog that explains
/// ten fields at once gets skipped, and the reader only wanted the one they
/// tapped. Each body says only what looking at the control does not.
#[derive(Debug, Clone, Copy, PartialEq)]
pub enum HintTopic {
    /// The archetype tags you pick on a deck.
    DeckTags,
    /// The granular functional tags that sharpen suggestions.
    OracleTags,
    /// The deck's name on the create/edit form.
    DeckName,
    /// The deck's format.
    Format,
    /// The commander or oathbreaker field.
    Commander,
    /// The partner commander field.
    Partner,
    /// The background field.
    Background,
    /// The oathbreaker signature spell field.
    SignatureSpell,
    /// The Commander bracket the deck targets.
    PowerLevel,
    /// The deck's land count target.
    LandTarget,
    /// The deck's budget.
    PriceTarget,
    /// Non-gameplay deck labels.
    OtherTags,
    /// A new password's field: the policy it has to meet.
    Password,
    /// Profile: email row.
    Email,
    /// Profile: email verification row.
    Verification,
    /// Profile: theme row, with its dark/light toggle.
    Theme,
    /// Profile: pips row, theme or natural colors.
    Pips,
    /// Profile: app icon row.
    AppIcon,
    /// Profile: Universes Beyond row.
    UniversesBeyond,
    /// Profile: website row.
    Website,
    /// Profile: version row, with the changelog button.
    Version,
    /// Deck view: the Types chart.
    TypeChart,
    /// Deck view: the Roles chart.
    RoleChart,
    /// Deck view: the Colors chart, in the Mana section.
    ColorChart,
    /// Deck view: mana curve chart.
    ManaCurve,
    /// Deck view: per-color mana fulfillment chart.
    ManaFulfillment,
    /// Deck view: draw odds chart.
    DrawOdds,
    /// Home and Playstyle: the color map.
    ColorMap,
    /// Playstyle: the card roles chart.
    SuggestionRoleChart,
    /// Playstyle: the oracle tags table.
    SuggestionTagChart,
    /// Playstyle: the oracle tags table's Decks column.
    SuggestionTagDecks,
    /// Playstyle: the oracle tags table's Cards column.
    SuggestionTagCards,
    /// Playstyle: the oracle tags table's Impact column.
    SuggestionTagImpact,
    /// Playstyle: the suggested commanders list.
    SuggestionList,
    /// Playstyle: the staples list.
    SuggestionStaples,
    /// Activity: the Activity section's heatmap and swipe columns.
    Activity,
    /// Activity: the Card roles section's rate chart.
    RoleRates,
    /// Activity: the Oracle tags section's rate chart.
    TagRates,
    /// Playstyle: the Hard passes group.
    HardPasses,
}

impl HintTopic {
    /// Dialog title for this topic (mirrors the in-app labels).
    pub fn title(&self) -> &'static str {
        match self {
            Self::DeckTags => "Deck tags",
            Self::OracleTags => "Oracle tags",
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
            Self::Password => "Password",
            Self::Email => "Email",
            Self::Verification => "Verification",
            Self::Theme => "Theme",
            Self::Pips => "Pips",
            Self::AppIcon => "App icon",
            Self::UniversesBeyond => "Universes Beyond",
            Self::Website => "Website",
            Self::Version => "Version",
            Self::TypeChart => "Types",
            Self::RoleChart => "Roles",
            Self::ColorChart => "Colors",
            Self::ManaCurve => "Mana curve",
            Self::ManaFulfillment => "Mana cost fulfillment",
            Self::DrawOdds => "Draw odds",
            Self::ColorMap => "Color map",
            Self::SuggestionRoleChart => "Card roles",
            Self::SuggestionTagChart => "Oracle tags",
            Self::SuggestionTagDecks => "Decks",
            Self::SuggestionTagCards => "Cards",
            Self::SuggestionTagImpact => "Impact",
            Self::SuggestionList => "Suggested commanders",
            Self::SuggestionStaples => "Staples",
            Self::Activity => "Activity",
            Self::RoleRates => "Card roles",
            Self::TagRates => "Oracle tags",
            Self::HardPasses => "Most skipped",
        }
    }

    /// The dialog body. Only what the control itself does not show.
    fn body(self) -> Element {
        match self {
            Self::DeckTags => rsx! { DeckTagsExplainer {} },
            Self::OracleTags => rsx! { OracleTagsExplainer {} },
            Self::DeckName => rsx! {
                HintLine { "Shown in your deck list. Change it any time" }
            },
            Self::Format => rsx! {
                HintBullets {
                    HintBullet { "Sets deck size, card pool and the command zone" }
                    HintBullet { "Changing it clears the commander and signature spell" }
                }
            },
            Self::Commander => rsx! {
                HintBullets {
                    HintBullet { "Search only shows cards that can lead this format. " HintKey { color: "--color-warning", "Filter" } " searches any card" }
                    HintBullet { HintKey { "Swipe" } " picks one by swiping" }
                }
            },
            Self::Partner => rsx! {
                HintLine { "Shows when the commander allows a partner" }
            },
            Self::Background => rsx! {
                HintLine { "Shows when the commander can take a background" }
            },
            Self::SignatureSpell => rsx! {
                HintLine { "Oathbreaker only: the instant or sorcery that starts in the command zone" }
            },
            Self::PowerLevel => rsx! {
                HintLine { "The Commander bracket the deck aims for, 1 to 5" }
            },
            Self::LandTarget => rsx! {
                HintBullets {
                    HintBullet { "How many lands the deck should run. Fewer warns on the deck" }
                    HintBullet { "Not set uses the format's usual count" }
                }
            },
            Self::PriceTarget => rsx! {
                HintLine { "A budget for the mainboard. Going over warns on the deck" }
            },
            Self::OtherTags => rsx! {
                HintLine { "Labels that do not change suggestions, like Budget or Jank" }
            },
            Self::Password => rsx! {
                HintBullets {
                    for line in password::requirements() {
                        HintBullet { "{line}" }
                    }
                }
            },
            Self::Email => rsx! {
                HintLine { "Where password resets and verification go" }
            },
            Self::Verification => rsx! {
                HintLine { "Confirms the email is yours. " HintKey { "Resend" } " if it never came" }
            },
            Self::Theme => rsx! {
                HintBullets {
                    HintBullet { "The app's palette. Picks preview as you tap" }
                    HintBullet { "The last four are color blind modes" }
                    HintBullet { HintKey { ModeIcon { dark: false } " light" } " " HintKey { ModeIcon { dark: true } " dark" } " switches between the theme's two palettes" }
                }
            },
            Self::Pips => rsx! {
                HintBullets {
                    HintBullet { HintKey { "Theme" } " draws mana symbols in your theme's colors" }
                    HintBullet { HintKey { "Natural" } " keeps Magic's classic colors on every theme" }
                }
            },
            Self::AppIcon => rsx! {
                HintBullets {
                    HintBullet { "The icon on your home screen, in any theme's colors" }
                    HintBullet { HintKey { "Match theme" } " keeps it in your theme as you change it" }
                }
            },
            Self::UniversesBeyond => rsx! {
                HintBullets {
                    HintBullet { HintKey { "Hide" } " keeps crossover cards out of searches and commander picks" }
                    HintBullet { "Exceptions keeps chosen franchises showing" }
                }
            },
            Self::Website => rsx! {
                HintLine { "Guides, the changelog and news, in your browser" }
            },
            Self::Version => rsx! {
                HintLine { "The build you are on. " HintKey { "Changelog" } " lists what changed in each release" }
            },
            Self::TypeChart => rsx! {
                HintLine { "Mainboard cards by type" }
            },
            Self::RoleChart => rsx! {
                HintLine { "Mainboard cards by role, like ramp or removal. Cards with no role are left out. Tap a role to see its tags" }
            },
            Self::ColorChart => rsx! {
                HintLine { "Mainboard cards by color" }
            },
            Self::ManaCurve => rsx! {
                HintLine { "Nonland mainboard cards by mana value" }
            },
            Self::ManaFulfillment => rsx! {
                HintBullets {
                    HintBullet { "Per color, the pips your lands and sources make against the pips your spells need" }
                    HintBullet { "A check means you make at least as much as you need" }
                }
            },
            Self::DrawOdds => rsx! {
                HintBullets {
                    HintBullet { "Chance of at least one card from each group in the opening hand, or by a turn. " HintKey { "-" } " and " HintKey { "+" } " step the turn" }
                    HintBullet { "Random draw only. Mulligans, tutors and card draw are not counted" }
                }
            },
            Self::ColorMap => rsx! {
                HintBullets {
                    HintBullet { "Each tile stands for a share of the colored cards across your decks" }
                    HintBullet { "Every color grows out from the middle, and the color with the most cards claims the most ground" }
                    HintBullet { "The shape is generated per player" }
                    HintBullet { "Colorless cards are in the key below" }
                }
            },
            Self::SuggestionRoleChart => rsx! {
                HintLine { "How many of your decks have a tag in each role. Tap a role for its tags" }
            },
            Self::SuggestionTagChart => rsx! {
                HintLine { "Your 10 most impactful tags, highest first. Tap a tag to see what it means" }
            },
            Self::SuggestionTagDecks => rsx! {
                HintLine { "How many of your decks have this tag. The more decks, the higher its impact" }
            },
            Self::SuggestionTagCards => rsx! {
                HintLine { "How many cards carry this tag across those decks. The more cards, the higher its impact" }
            },
            Self::SuggestionTagImpact => rsx! {
                HintLine { "How much more your decks lean on this tag than Magic cards do overall. Your top tag scores 100" }
            },
            Self::SuggestionList => rsx! {
                HintBullets {
                    HintBullet { "Commanders you haven't built, ranked against all your decks, best fit first" }
                    HintBullet { "Tap a row to see why it fits. " HintKey { "Swipe" } " goes further down the same ranking" }
                }
            },
            Self::SuggestionStaples => rsx! {
                HintBullets {
                    HintBullet { "The cards in the most of your decks, with the share of your decks that run each" }
                    HintBullet { "Basic lands are left out. Tap a Type or Color chip to narrow it" }
                }
            },
            Self::Activity => rsx! {
                HintBullets {
                    HintBullet { "Your swipes and searches each day over the last year, a darker square for a busier day" }
                    HintBullet { "A lit square is one of your biggest days. Tap a square to see its counts" }
                    HintBullet { HintKey { "Totals" } " is what the cards dealt to your decks came to: " HintKey { "Added" } " and " HintKey { "Maybeboard" } " you kept, " HintKey { "Skipped" } " you passed on, and " HintKey { "Removed" } " you later took out" }
                }
            },
            Self::RoleRates => rsx! {
                HintBullets {
                    HintBullet { "Your share for each card role. Pick one under " HintKey { "Show" } ": " HintKey { "Added" } ", " HintKey { "Skipped" } " and " HintKey { "Maybeboard" } " are shares of the cards dealt to you, and " HintKey { "Removed" } " is the share of added cards you later took out" }
                    HintBullet { "Tap a role to see the tags you swiped under it. A grey role has none yet" }
                    HintBullet { "From your decks with a commander" }
                }
            },
            Self::TagRates => rsx! {
                HintBullets {
                    HintBullet { "The tags where your share stands out most from other players', with your own share" }
                    HintBullet { "A tag everyone passes on shows only after a lot of skips. Tap a tag to see what it means" }
                    HintBullet { "From your decks with a commander, for tags dealt at least 10 times" }
                }
            },
            Self::HardPasses => rsx! {
                HintBullets {
                    HintBullet { "Cards you skip and never add or save, when other players usually keep them" }
                    HintBullet { "A card everyone passes on shows only after a lot of skips. The tag counts yours, and an open row names the commanders you skipped it under" }
                    HintBullet { "From your decks with a commander" }
                }
            },
        }
    }
}

/// The receiver: reads the hint channel and describes the current topic's dialog
/// to the app root's dialog host. Mount once at the app root, beside the router.
/// `spawn_upkeeper` provides the `Signal<Option<HintTopic>>` this reads.
#[component]
pub fn HintHost() -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let mut open = use_signal(|| false);
    // A posted topic opens the dialog; closing it clears the channel.
    use_effect(move || {
        if hint().is_some() {
            open.set(true);
        }
    });
    use_effect(move || {
        if !open() {
            hint.set(None);
        }
    });
    let topic = hint();
    rsx! {
        AlertDialog {
            open,
            title: topic.map(|t| t.title()).unwrap_or_default().to_string(),
            actions: vec![DialogAction::Dismiss("Got it".into())],
            if let Some(topic) = topic {
                {topic.body()}
            }
        }
    }
}
