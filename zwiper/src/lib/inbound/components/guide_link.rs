//! The "See guide ↗" action a hint dialog offers when zwipe.net has a guide
//! that goes deeper than the hint.
//!
//! Which guide goes with which hint is `zwipe-ui`'s
//! [`guide`](zwipe_ui::hint::guide) map, shared with the other clients. The
//! action is a [`DialogAction::Link`](components::DialogAction::Link), which the app root's
//! [`components::use_in_app_links`] opens in the in-app browser. A hint with
//! no guide gets no action.

use components::DialogAction;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::hint::{HintTopic as SharedTopic, guide};

use crate::inbound::components::hint_host::HintTopic;

/// The action's label.
pub const SEE_GUIDE: &str = "See guide \u{2197}";

/// The action for a `?` topic, empty when the topic has no guide.
pub fn topic(topic: HintTopic) -> Vec<DialogAction> {
    shared_topic(SharedTopic::from(topic))
}

/// The action for a topic named in `zwipe-ui`'s terms, for the screen
/// hints that explain the same thing as a `?` topic.
pub fn shared_topic(topic: SharedTopic) -> Vec<DialogAction> {
    action(topic.guide_slug())
}

/// The action for a one-time hint, by the key it is recorded under, empty when
/// the hint has no guide.
pub fn one_time(key: &str) -> Vec<DialogAction> {
    action(OneTimeHint::from_key(key).and_then(guide::one_time_slug))
}

fn action(slug: Option<&'static str>) -> Vec<DialogAction> {
    slug.map(|slug| DialogAction::Link {
        label: SEE_GUIDE.to_string(),
        href: guide::url(slug),
    })
    .into_iter()
    .collect()
}

impl From<HintTopic> for SharedTopic {
    fn from(topic: HintTopic) -> Self {
        match topic {
            HintTopic::DeckTags => SharedTopic::DeckTags,
            HintTopic::OracleTags => SharedTopic::OracleTags,
            HintTopic::DeckName => SharedTopic::DeckName,
            HintTopic::Format => SharedTopic::Format,
            HintTopic::Commander => SharedTopic::Commander,
            HintTopic::Partner => SharedTopic::Partner,
            HintTopic::Background => SharedTopic::Background,
            HintTopic::SignatureSpell => SharedTopic::SignatureSpell,
            HintTopic::PowerLevel => SharedTopic::PowerLevel,
            HintTopic::LandTarget => SharedTopic::LandTarget,
            HintTopic::PriceTarget => SharedTopic::PriceTarget,
            HintTopic::OtherTags => SharedTopic::OtherTags,
            HintTopic::Password => SharedTopic::Password,
            HintTopic::Email => SharedTopic::Email,
            HintTopic::Verification => SharedTopic::Verification,
            HintTopic::Theme => SharedTopic::Theme,
            HintTopic::Pips => SharedTopic::Pips,
            HintTopic::AppIcon => SharedTopic::AppIcon,
            HintTopic::UniversesBeyond => SharedTopic::UniversesBeyond,
            HintTopic::Website => SharedTopic::Website,
            HintTopic::Version => SharedTopic::Version,
            HintTopic::TypeChart => SharedTopic::TypeChart,
            HintTopic::RoleChart => SharedTopic::RoleChart,
            HintTopic::ColorChart => SharedTopic::ColorChart,
            HintTopic::ManaCurve => SharedTopic::ManaCurve,
            HintTopic::ManaFulfillment => SharedTopic::ManaFulfillment,
            HintTopic::DrawOdds => SharedTopic::DrawOdds,
            HintTopic::ColorMap => SharedTopic::ColorMap,
            HintTopic::SuggestionRoleChart => SharedTopic::SuggestionRoleChart,
            HintTopic::SuggestionTagChart => SharedTopic::SuggestionTagChart,
            HintTopic::SuggestionTagDecks => SharedTopic::SuggestionTagDecks,
            HintTopic::SuggestionTagCards => SharedTopic::SuggestionTagCards,
            HintTopic::SuggestionTagImpact => SharedTopic::SuggestionTagImpact,
            HintTopic::SuggestionList => SharedTopic::SuggestionList,
            HintTopic::SuggestionStaples => SharedTopic::SuggestionStaples,
            HintTopic::Activity => SharedTopic::Activity,
            HintTopic::RoleRates => SharedTopic::RoleRates,
            HintTopic::TagRates => SharedTopic::TagRates,
            HintTopic::HardPasses => SharedTopic::HardPasses,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_topic_with_a_guide_links_to_it() {
        let actions = topic(HintTopic::LandTarget);
        assert!(
            matches!(
                actions.as_slice(),
                [DialogAction::Link { label, href }]
                    if label == SEE_GUIDE && href == "https://zwipe.net/guides/land-targets"
            ),
            "actions {actions:?}"
        );
    }

    #[test]
    fn a_topic_without_a_guide_has_no_action() {
        assert_eq!(topic(HintTopic::Password), Vec::new());
    }

    #[test]
    fn a_one_time_hint_links_by_its_key() {
        let key = OneTimeHint::Filter.key();
        assert!(
            matches!(
                one_time(key).as_slice(),
                [DialogAction::Link { href, .. }] if href == "https://zwipe.net/guides/filtering"
            ),
            "key {key}"
        );
        assert_eq!(one_time(OneTimeHint::Config.key()), Vec::new());
        assert_eq!(one_time("not-a-hint"), Vec::new());
    }
}
