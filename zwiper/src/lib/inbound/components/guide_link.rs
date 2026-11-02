//! The "See guide ↗" action a screen's main hint offers when zwipe.net has a
//! guide that goes deeper than the hint.
//!
//! Only the hint that explains a whole screen links out: the one-time hint a
//! screen opens with, or its header hint. A panel's or field's `?` answers
//! its own quick question and offers no guide.
//!
//! Which guide goes with which hint is `zwipe-ui`'s
//! [`guide`](zwipe_ui::hint::guide) map, shared with the other clients. The
//! action is a [`DialogAction::Link`](components::DialogAction::Link), which the app root's
//! [`components::use_in_app_links`] opens in the in-app browser. A hint with
//! no guide gets no action.

use components::DialogAction;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::hint::{HintTopic, guide};

/// The action's label.
pub const SEE_GUIDE: &str = "See guide \u{2197}";

/// The action for a screen's header hint, by the `?` topic that explains the
/// same thing, empty when the topic has no guide.
pub fn screen(topic: HintTopic) -> Vec<DialogAction> {
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

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_topic_with_a_guide_links_to_it() {
        let actions = screen(HintTopic::LandTarget);
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
        assert_eq!(screen(HintTopic::Password), Vec::new());
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
