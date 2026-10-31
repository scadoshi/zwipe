//! Hint copy: the `?` topics a user opens beside a row or a field, and the
//! one-time hints a screen shows the first time a user reaches it.
//!
//! The copy is typed spans, not markup, so each client draws it with its own
//! pieces: the app and the web through `HintKey` and `HintColored`, the
//! terminal as styled spans. A [`HintContent`] is a title and its
//! [`Block`]s; a block is a line or a bullet of [`Span`]s; a span is plain
//! text, a key chip or an emphasized word, the last two colored by a
//! [`Role`].

pub mod one_time;
pub mod topics;

pub use self::one_time::SwipeSelectMode;
pub use self::topics::HintTopic;

/// The theme color a key chip or an emphasized word takes.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash, Default)]
pub enum Role {
    /// The theme's first palette swatch.
    Palette1,
    /// The theme's second palette swatch, every key chip's color unless the
    /// copy names another.
    #[default]
    Palette2,
    /// The theme's third palette swatch.
    Palette3,
    /// The theme's fourth palette swatch.
    Palette4,
    /// The theme's fifth palette swatch.
    Palette5,
    /// The theme's sixth palette swatch, the `?` button's color.
    Palette6,
    /// The primary accent.
    Primary,
    /// The secondary accent.
    Secondary,
    /// The tertiary accent.
    Tertiary,
    /// The success color: adds and right swipes.
    Success,
    /// The error color: removals, skips and left swipes.
    Error,
    /// The warning color: the maybeboard, resets and up swipes.
    Warning,
}

impl Role {
    /// The CSS variable the app's and the web's themes define for this role,
    /// such as `--accent-primary`.
    #[must_use]
    pub fn css_var(self) -> &'static str {
        match self {
            Self::Palette1 => "--palette-1",
            Self::Palette2 => "--palette-2",
            Self::Palette3 => "--palette-3",
            Self::Palette4 => "--palette-4",
            Self::Palette5 => "--palette-5",
            Self::Palette6 => "--palette-6",
            Self::Primary => "--accent-primary",
            Self::Secondary => "--accent-secondary",
            Self::Tertiary => "--accent-tertiary",
            Self::Success => "--color-success",
            Self::Error => "--color-error",
            Self::Warning => "--color-warning",
        }
    }
}

/// An icon drawn inside a key chip, before its label and a space.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyIcon {
    /// Light mode's sun.
    Light,
    /// Dark mode's moon.
    Dark,
}

/// One run of hint copy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Span {
    /// Plain text, drawn as is with its spaces.
    Text(String),
    /// A key chip naming a button or a chip on the screen, such as `Apply`.
    Key {
        /// The words on the chip.
        label: String,
        /// The chip's color.
        role: Role,
        /// An icon before the label, if any.
        icon: Option<KeyIcon>,
    },
    /// A word or phrase drawn bold in a role's color, such as a swipe
    /// direction.
    Emph {
        /// The emphasized words.
        text: String,
        /// Their color.
        role: Role,
    },
}

impl Span {
    /// The words the span reads as, without its styling or icon.
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) | Self::Emph { text, .. } => text,
            Self::Key { label, .. } => label,
        }
    }
}

/// One block of a hint's body.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Block {
    /// A line of its own.
    Line(Vec<Span>),
    /// A bullet; consecutive bullets form one list.
    Bullet(Vec<Span>),
}

impl Block {
    /// The block's spans, whether a line or a bullet.
    #[must_use]
    pub fn spans(&self) -> &[Span] {
        match self {
            Self::Line(spans) | Self::Bullet(spans) => spans,
        }
    }

    /// The words the block reads as, its spans joined without styling.
    #[must_use]
    pub fn plain(&self) -> String {
        self.spans().iter().map(Span::text).collect()
    }
}

/// What a hint dialog says: its title and its body. Every hint dialog ends
/// with one dismiss button, [`GOT_IT`].
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct HintContent {
    /// The dialog's title.
    pub title: String,
    /// The dialog's body, top to bottom.
    pub blocks: Vec<Block>,
}

impl HintContent {
    /// The body as it reads, one block per line, styling dropped.
    #[must_use]
    pub fn plain(&self) -> String {
        self.blocks
            .iter()
            .map(Block::plain)
            .collect::<Vec<_>>()
            .join("\n")
    }
}

/// The dismiss button every hint dialog ends with.
pub const GOT_IT: &str = "Got it";

/// Plain text.
fn text(text: &str) -> Span {
    Span::Text(text.to_owned())
}

/// A key chip in the default color.
fn key(label: &str) -> Span {
    key_in(label, Role::default())
}

/// A key chip in `role`'s color.
fn key_in(label: &str, role: Role) -> Span {
    Span::Key {
        label: label.to_owned(),
        role,
        icon: None,
    }
}

/// Words drawn bold in `role`'s color.
fn emph(text: &str, role: Role) -> Span {
    Span::Emph {
        text: text.to_owned(),
        role,
    }
}

/// A line of its own.
fn line(spans: Vec<Span>) -> Block {
    Block::Line(spans)
}

/// A bullet.
fn bullet(spans: Vec<Span>) -> Block {
    Block::Bullet(spans)
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn plain_text_drops_styling_and_keeps_spaces() {
        let block = bullet(vec![
            key("Apply"),
            text(" uses it. "),
            key_in("Reset", Role::Warning),
            text(" then "),
            emph("Apply", Role::Primary),
        ]);
        assert_eq!(block.plain(), "Apply uses it. Reset then Apply");
        assert_eq!(
            Role::default(),
            Role::Palette2,
            "key chips default to palette 2"
        );
    }

    #[test]
    fn roles_name_the_css_variables_the_app_uses() {
        assert_eq!(Role::Palette2.css_var(), "--palette-2");
        assert_eq!(Role::Tertiary.css_var(), "--accent-tertiary");
        assert_eq!(Role::Warning.css_var(), "--color-warning");
    }
}
