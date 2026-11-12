//! Draws zwipe-ui's typed hint copy with the kit's hint pieces, so every
//! `?` topic and one-time hint reads word for word as the model says it.
//!
//! [`HintBody`] turns a hint's [`Block`]s into `HintLine`s and `HintBullets`,
//! [`KeyGlyph`] draws a [`KeyIcon`] as the button it names draws it, and
//! [`ContentHintDialog`] is a whole [`HintContent`] in a `HintDialog`. The
//! site's guides draw their `[[icon]]` tokens through [`KeyGlyph`] too, so an
//! icon looks the same in the app's hint and on the web.
//!
//! [`guide_for_screen`], [`guide_for_one_time`] and [`guide_for_swipe_select`] are the "See guide ↗"
//! action a screen's main hint offers when zwipe.net has a guide that goes
//! deeper: the one-time hint a screen opens with, or its header hint. A
//! panel's or field's `?` answers its own quick question and offers no guide.
//! Which guide goes with which hint is `zwipe-ui`'s
//! [`guide`] map. The action is a
//! [`DialogAction::Link`]: the app opens it in its in-app browser, the web
//! in the same tab.

use dioxus::prelude::*;
use orangui::icons::{
    BoltIcon, CHECK, COG, COPY, CloneIcon, DockIcon, ERASER, EditIcon, EyeIcon, FUNNEL, HOUSE,
    INFO, MINUS, PERSON, PLUS, REFRESH, ShareIcon, UNDO,
};
use orangui::{
    DialogAction, HintBullet, HintBullets, HintDialog, HintKey, HintLine, ModeIcon, PaletteIcon,
};
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::hint::{Block, HintContent, HintTopic, KeyIcon, Span, SwipeSelectMode, guide};

/// A run of blocks as drawn: consecutive bullets share one list.
enum Group<'blocks> {
    Line(&'blocks [Span]),
    Bullets(Vec<&'blocks [Span]>),
}

/// Folds `blocks` into lines and bullet lists, keeping their order.
fn groups(blocks: &[Block]) -> Vec<Group<'_>> {
    let mut out: Vec<Group<'_>> = Vec::new();
    for block in blocks {
        match block {
            Block::Line(spans) => out.push(Group::Line(spans)),
            Block::Bullet(spans) => {
                if let Some(Group::Bullets(list)) = out.last_mut() {
                    list.push(spans);
                } else {
                    out.push(Group::Bullets(vec![spans]));
                }
            }
        }
    }
    out
}

/// A hint dialog's body: each line as a `HintLine`, each run of bullets as
/// one `HintBullets` list.
#[component]
pub fn HintBody(blocks: Vec<Block>) -> Element {
    rsx! {
        for group in groups(&blocks) {
            match group {
                Group::Line(spans) => rsx! { HintLine { Spans { spans: spans.to_vec() } } },
                Group::Bullets(list) => rsx! {
                    HintBullets {
                        for spans in list {
                            HintBullet { Spans { spans: spans.to_vec() } }
                        }
                    }
                },
            }
        }
    }
}

/// One line's spans: plain text, key chips and colored words.
#[component]
fn Spans(spans: Vec<Span>) -> Element {
    rsx! {
        for span in spans {
            match span {
                Span::Text(text) => rsx! { "{text}" },
                Span::Key { label, role, icon } => rsx! {
                    HintKey { color: role.css_var(),
                        if let Some(icon) = icon {
                            KeyGlyph { icon }
                            if !label.is_empty() {
                                " "
                            }
                        }
                        "{label}"
                    }
                },
                Span::Emph { text, role } => rsx! {
                    HintColored { color: role.css_var(), "{text}" }
                },
            }
        }
    }
}

/// A key's icon, drawn as the button it names draws it: the dock icons from
/// their paths, the eye, pencil, clone and share from their components, the
/// palette from the theme button's own icon, and the sun or moon from the
/// kit's mode icon.
#[component]
pub fn KeyGlyph(icon: KeyIcon) -> Element {
    let path = match icon {
        KeyIcon::Plus => PLUS,
        KeyIcon::Minus => MINUS,
        KeyIcon::Funnel => FUNNEL,
        KeyIcon::Cart => crate::CART,
        KeyIcon::CardArt => crate::CARD_ART,
        KeyIcon::Flip => orangui::icons::FLIP,
        KeyIcon::Meld => crate::MELD,
        KeyIcon::Star => crate::STAR,
        KeyIcon::Undo => UNDO,
        KeyIcon::Eraser => ERASER,
        KeyIcon::Refresh => REFRESH,
        KeyIcon::Check => CHECK,
        KeyIcon::Copy => COPY,
        KeyIcon::Bolt => return rsx! { BoltIcon {} },
        KeyIcon::Person => PERSON,
        KeyIcon::Cog => COG,
        KeyIcon::Info => INFO,
        KeyIcon::Home => HOUSE,
        KeyIcon::Palette => return rsx! { PaletteIcon {} },
        KeyIcon::Eye => return rsx! { EyeIcon {} },
        KeyIcon::Edit => return rsx! { EditIcon {} },
        KeyIcon::Clone => return rsx! { CloneIcon {} },
        KeyIcon::Share => return rsx! { ShareIcon {} },
        KeyIcon::Light => return rsx! { ModeIcon { dark: false } },
        KeyIcon::Dark => return rsx! { ModeIcon { dark: true } },
    };
    rsx! { DockIcon { path } }
}

/// A colored word inside a hint, such as a swipe direction. `color` names a
/// CSS variable, like `--color-success`; the word draws in that variable's
/// ink, so it reads in every theme.
#[component]
pub fn HintColored(color: String, children: Element) -> Element {
    let ink = orangui::ink_color(&color);
    rsx! {
        span { style: "color: {ink}; font-weight: 600;", {children} }
    }
}

/// A hint's dialog drawn from its `content`, with `actions` (a guide link,
/// as a rule) before its dismiss button.
#[component]
pub fn ContentHintDialog(
    open: Signal<bool>,
    content: HintContent,
    #[props(default)] actions: Vec<DialogAction>,
) -> Element {
    rsx! {
        HintDialog { open, title: content.title.clone(), actions,
            HintBody { blocks: content.blocks }
        }
    }
}

/// The guide action's label.
pub(crate) const SEE_GUIDE: &str = "See guide \u{2197}";

/// The guide action for a screen's header hint, by the `?` topic that
/// explains the same thing, empty when the topic has no guide.
#[must_use]
pub fn guide_for_screen(topic: HintTopic) -> Vec<DialogAction> {
    guide_action(topic.guide_slug())
}

/// The guide action for a one-time `hint`, empty when it has no guide.
#[must_use]
pub fn guide_for_one_time(hint: OneTimeHint) -> Vec<DialogAction> {
    guide_action(guide::one_time_slug(hint))
}

/// The guide action for the Swipe select hint, whose guide changes with
/// its overlay `mode`.
#[must_use]
pub fn guide_for_swipe_select(mode: SwipeSelectMode) -> Vec<DialogAction> {
    guide_action(Some(guide::swipe_select_slug(mode)))
}

/// A link to the guide named `slug`, or nothing without one.
fn guide_action(slug: Option<&'static str>) -> Vec<DialogAction> {
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
        let actions = guide_for_screen(HintTopic::LandTarget);
        assert!(
            matches!(
                actions.as_slice(),
                [DialogAction::Link { label, href }]
                    if label == SEE_GUIDE && href == "https://zwipe.net/guides/land-targets/"
            ),
            "actions {actions:?}"
        );
    }

    #[test]
    fn a_topic_without_a_guide_has_no_action() {
        assert_eq!(guide_for_screen(HintTopic::Password), Vec::new());
    }

    #[test]
    fn a_one_time_hint_links_to_its_guide() {
        assert!(matches!(
            guide_for_one_time(OneTimeHint::Filter).as_slice(),
            [DialogAction::Link { href, .. }] if href == "https://zwipe.net/guides/filtering/"
        ));
        assert_eq!(guide_for_one_time(OneTimeHint::Config), Vec::new());
    }

    #[test]
    fn swipe_select_links_by_its_overlay() {
        assert!(matches!(
            guide_for_swipe_select(SwipeSelectMode::Suggested { has_filter: false }).as_slice(),
            [DialogAction::Link { href, .. }] if href == "https://zwipe.net/guides/playstyle/"
        ));
    }
}
