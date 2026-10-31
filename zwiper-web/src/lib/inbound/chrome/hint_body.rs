//! Draws zwipe-ui's typed hint copy with the kit's hint pieces, so every
//! `?` topic and one-time hint reads word for word as the app's.

use components::{HintBullet, HintBullets, HintDialog, HintKey, HintLine, ModeIcon};
use dioxus::prelude::*;
use zwipe_ui::hint::{Block, HintContent, KeyIcon, Span};

/// A run of blocks as drawn: consecutive bullets share one list.
enum Group<'a> {
    Line(&'a [Span]),
    Bullets(Vec<&'a [Span]>),
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
                            ModeIcon { dark: icon == KeyIcon::Dark }
                            " "
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

/// A colored word inside a hint, such as a swipe direction. `color` names a
/// CSS variable, like `--color-success`.
#[component]
pub fn HintColored(color: String, children: Element) -> Element {
    rsx! {
        span { style: "color: var({color}); font-weight: 600;", {children} }
    }
}

/// A one-time hint's dialog drawn from its `content`.
#[component]
pub fn ContentHintDialog(open: Signal<bool>, content: HintContent) -> Element {
    rsx! {
        HintDialog { open, title: content.title.clone(), HintBody { blocks: content.blocks } }
    }
}
