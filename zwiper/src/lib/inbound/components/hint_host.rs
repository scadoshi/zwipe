//! App-root hint host: a sender/receiver for on-demand "?" help.
//!
//! An [`InfoButton`](super::info_button::InfoButton) anywhere in the tree posts a
//! [`HintTopic`] to a context signal; this single [`HintHost`], mounted at the app
//! root (beside the router), renders the dialog. Because the dialog only ever
//! renders here, outside every screen's `content-enter` / scroll container, its
//! `position: fixed` overlay can't be trapped by an ancestor's containing block.
//! A single `Option` also means only one hint shows at a time.

use dioxus::prelude::*;
use zwipe_components::{HintBullet, HintBullets, HintKey, HintLine};

use crate::inbound::components::{
    alert_dialog::{
        AlertDialogAction, AlertDialogActions, AlertDialogContent, AlertDialogDescription,
        AlertDialogRoot, AlertDialogTitle,
    },
    concept_explainers::{DeckTagsExplainer, OracleTagsExplainer},
};

/// A concept a "?" button can explain. Cheap `Copy` message on the hint channel.
///
/// One per field or row rather than one per screen: a dialog that explains
/// ten fields at once gets skipped, and the reader only wanted the one they
/// tapped. Each body says only what looking at the control does not.
#[derive(Clone, Copy, PartialEq)]
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
    /// The WotC bracket the deck targets.
    PowerLevel,
    /// The deck's land count target.
    LandTarget,
    /// The deck's budget.
    PriceTarget,
    /// Non-gameplay deck labels.
    OtherTags,
    /// Profile: email row.
    Email,
    /// Profile: email verification row.
    Verification,
    /// Profile: theme row.
    Theme,
    /// Profile: dark mode row.
    DarkMode,
    /// Profile: Universes Beyond row.
    UniversesBeyond,
    /// Profile: website row.
    Website,
    /// Profile: version row, with the changelog button.
    Version,
    /// Deck view: type distribution chart.
    TypeChart,
    /// Deck view: role distribution chart.
    RoleChart,
    /// Deck view: color distribution chart.
    ColorChart,
    /// Deck view: mana curve chart.
    ManaCurve,
    /// Deck view: per-color mana fulfillment chart.
    ManaFulfillment,
    /// Deck view: draw odds chart.
    DrawOdds,
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
            Self::Email => "Email",
            Self::Verification => "Verification",
            Self::Theme => "Theme",
            Self::DarkMode => "Dark mode",
            Self::UniversesBeyond => "Universes Beyond",
            Self::Website => "Website",
            Self::Version => "Version",
            Self::TypeChart => "Type distribution",
            Self::RoleChart => "Role distribution",
            Self::ColorChart => "Color distribution",
            Self::ManaCurve => "Mana curve",
            Self::ManaFulfillment => "Mana cost fulfillment",
            Self::DrawOdds => "Draw odds",
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
                    HintBullet { HintKey { color: "--accent-primary", "Swipe" } " picks one by swiping" }
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
            Self::Email => rsx! {
                HintLine { "Where password resets and verification go" }
            },
            Self::Verification => rsx! {
                HintLine { "Confirms the email is yours. " HintKey { color: "--accent-primary", "Resend" } " if it never came" }
            },
            Self::Theme => rsx! {
                HintBullets {
                    HintBullet { "The app's palette. Picks preview as you tap" }
                    HintBullet { "The last four are color blind modes" }
                }
            },
            Self::DarkMode => rsx! {
                HintLine { "Switches the theme between its light and dark palettes" }
            },
            Self::UniversesBeyond => rsx! {
                HintBullets {
                    HintBullet { HintKey { color: "--accent-secondary", "Hide" } " keeps crossover cards out of searches and commander picks" }
                    HintBullet { "Exceptions keeps chosen franchises showing" }
                }
            },
            Self::Website => rsx! {
                HintLine { "Guides, the changelog and news, in your browser" }
            },
            Self::Version => rsx! {
                HintLine { "The build you are on. " HintKey { color: "--accent-primary", "Changelog" } " lists what changed in each release" }
            },
            Self::TypeChart => rsx! {
                HintLine { "Mainboard cards by type" }
            },
            Self::RoleChart => rsx! {
                HintLine { "Mainboard cards by role, like ramp or removal. Cards with no role are left out" }
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
                    HintBullet { "Chance of at least one card from each group in the opening hand, or by a turn. " HintKey { color: "--accent-primary", "-" } " and " HintKey { color: "--accent-primary", "+" } " step the turn" }
                    HintBullet { "Random draw only. Mulligans, tutors and card draw are not counted" }
                }
            },
        }
    }
}

/// The receiver: reads the hint channel and renders the current topic's dialog.
/// Mount once at the app root, beside the router. `spawn_upkeeper` provides the
/// `Signal<Option<HintTopic>>` this reads.
#[component]
pub fn HintHost() -> Element {
    let mut hint: Signal<Option<HintTopic>> = use_context();
    let topic = hint();
    rsx! {
        AlertDialogRoot {
            open: topic.is_some(),
            on_open_change: move |open: bool| {
                if !open {
                    hint.set(None);
                }
            },
            AlertDialogContent {
                if let Some(topic) = topic {
                    AlertDialogTitle { "{topic.title()}" }
                    hr { class: "dialog-rule" }
                    AlertDialogDescription { {topic.body()} }
                    hr { class: "dialog-rule" }
                    AlertDialogActions {
                        AlertDialogAction { on_click: move |_| hint.set(None), "Got it" }
                    }
                }
            }
        }
    }
}
