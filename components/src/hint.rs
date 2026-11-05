//! Draws zwipe-ui's typed hint copy with the kit's hint pieces, so every
//! `?` topic and one-time hint reads word for word as the model says it.
//!
//! [`HintBody`] turns a hint's [`Block`]s into `HintLine`s and `HintBullets`,
//! [`KeyGlyph`] draws a [`KeyIcon`] as the button it names draws it, and
//! [`ContentHintDialog`] is a whole [`HintContent`] in a `HintDialog`. The
//! site's guides draw their `[[icon]]` tokens through [`KeyGlyph`] too, so an
//! icon looks the same in the app's hint and on the web.

use dioxus::prelude::*;
use orangui::icons::{
    BoltIcon, CHECK, COG, COPY, CloneIcon, DockIcon, ERASER, EditIcon, EyeIcon, FUNNEL, HOUSE,
    INFO, MINUS, PERSON, PLUS, REFRESH, ShareIcon, UNDO,
};
use orangui::{
    DialogAction, HintBullet, HintBullets, HintDialog, HintKey, HintLine, ModeIcon, PaletteIcon,
};
use zwipe_ui::hint::{Block, HintContent, KeyIcon, Span};

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
/// CSS variable, like `--color-success`.
#[component]
pub fn HintColored(color: String, children: Element) -> Element {
    rsx! {
        span { style: "color: var({color}); font-weight: 600;", {children} }
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
