//! Hint copy: the `?` topics a user opens beside a row or a field, and the
//! one-time hints a screen shows the first time a user reaches it.
//!
//! The copy is typed spans, not markup, so each client draws it with its own
//! pieces: the app and the web through `HintKey` and `HintColored`, the
//! terminal as styled spans. A [`HintContent`] is a title and its
//! [`Block`]s; a block is a line or a bullet of [`Span`]s; a span is plain
//! text, a key chip or an emphasized word, the last two colored by a
//! [`Role`].
//!
//! A key chip names a button the way the button renders: a text button by
//! its words, an icon button by its icon alone, as a [`KeyIcon`] the
//! renderer draws inside the chip. Copy never describes an icon in words.

pub mod guide;
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

/// An icon drawn inside a key chip: alone, for an icon button, or before a
/// label and a space. The dock icons are the app's action dock set; each
/// renderer maps them to its own paths.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum KeyIcon {
    /// The add button's plus.
    Plus,
    /// The remove button's minus.
    Minus,
    /// The filter button's funnel.
    Funnel,
    /// The undo and reset button's arrow turning back.
    Undo,
    /// A sheet's reset, wiping the form to its defaults.
    Eraser,
    /// The refresh button's arrow circling round.
    Refresh,
    /// The save and apply button's checkmark.
    Check,
    /// The copy button's two stacked sheets.
    Copy,
    /// The synergy button's lightning bolt.
    Bolt,
    /// The card details button's eye.
    Eye,
    /// The edit button's pencil.
    Edit,
    /// The clone button's two overlapping squares.
    Clone,
    /// The share button's box with an arrow rising out of it.
    Share,
    /// The Account button's person.
    Person,
    /// The Settings button's cog.
    Cog,
    /// The About button's circled i.
    Info,
    /// The Theme button's palette.
    Palette,
    /// The Home button's house.
    Home,
    /// Light mode's sun.
    Light,
    /// Dark mode's moon.
    Dark,
}

impl KeyIcon {
    /// Every icon, in [`KeyIcon::name`] order.
    pub const ALL: [Self; 20] = [
        Self::Plus,
        Self::Minus,
        Self::Funnel,
        Self::Undo,
        Self::Eraser,
        Self::Refresh,
        Self::Check,
        Self::Copy,
        Self::Bolt,
        Self::Eye,
        Self::Edit,
        Self::Clone,
        Self::Share,
        Self::Person,
        Self::Cog,
        Self::Info,
        Self::Palette,
        Self::Home,
        Self::Light,
        Self::Dark,
    ];

    /// The icon whose [`KeyIcon::name`] is `name`, or `None` for a name no
    /// icon has.
    #[must_use]
    pub fn from_name(name: &str) -> Option<Self> {
        Self::ALL.into_iter().find(|icon| icon.name() == name)
    }

    /// What the icon reads as in the plain-text form of a hint, such as
    /// `plus` or `pencil`.
    #[must_use]
    pub fn name(self) -> &'static str {
        match self {
            Self::Plus => "plus",
            Self::Minus => "minus",
            Self::Funnel => "funnel",
            Self::Undo => "undo",
            Self::Eraser => "eraser",
            Self::Refresh => "refresh",
            Self::Check => "check",
            Self::Copy => "copy",
            Self::Bolt => "bolt",
            Self::Eye => "eye",
            Self::Edit => "pencil",
            Self::Clone => "clone",
            Self::Share => "share",
            Self::Person => "person",
            Self::Cog => "cog",
            Self::Info => "info",
            Self::Palette => "palette",
            Self::Home => "home",
            Self::Light => "light",
            Self::Dark => "dark",
        }
    }
}

/// One run of hint copy.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub enum Span {
    /// Plain text, drawn as is with its spaces.
    Text(String),
    /// A key chip naming a button or a chip on the screen, such as `Apply`.
    /// An icon button's chip is its icon alone: an empty label with an
    /// icon.
    Key {
        /// The words on the chip; empty for an icon-only chip.
        label: String,
        /// The chip's color.
        role: Role,
        /// An icon before the label, or the chip's whole content when the
        /// label is empty.
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
    /// The words the span reads as, without its styling. An icon-only key
    /// reads as its icon's [`KeyIcon::name`].
    #[must_use]
    pub fn text(&self) -> &str {
        match self {
            Self::Text(text) | Self::Emph { text, .. } => text,
            Self::Key {
                label,
                icon: Some(icon),
                ..
            } if label.is_empty() => icon.name(),
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

/// An icon button's key chip, the icon alone, in the default color.
fn icon_key(icon: KeyIcon) -> Span {
    icon_key_in(icon, Role::default())
}

/// An icon button's key chip, the icon alone, in `role`'s color.
fn icon_key_in(icon: KeyIcon, role: Role) -> Span {
    Span::Key {
        label: String::new(),
        role,
        icon: Some(icon),
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
    fn an_icon_key_reads_as_its_icon_name() {
        let block = line(vec![
            text("The "),
            icon_key(KeyIcon::Plus),
            text(" adds and the "),
            icon_key_in(KeyIcon::Undo, Role::Warning),
            text(" resets"),
        ]);
        assert_eq!(block.plain(), "The plus adds and the undo resets");
        assert_eq!(
            icon_key(KeyIcon::Edit),
            Span::Key {
                label: String::new(),
                role: Role::Palette2,
                icon: Some(KeyIcon::Edit),
            }
        );
        assert_eq!(
            icon_key_in(KeyIcon::Check, Role::Success),
            Span::Key {
                label: String::new(),
                role: Role::Success,
                icon: Some(KeyIcon::Check),
            }
        );
        let labeled = Span::Key {
            label: "light".to_owned(),
            role: Role::default(),
            icon: Some(KeyIcon::Light),
        };
        assert_eq!(labeled.text(), "light", "a labeled key keeps its words");
        assert_eq!(KeyIcon::Edit.name(), "pencil");
        assert_eq!(KeyIcon::Clone.name(), "clone");
    }

    #[test]
    fn every_icon_round_trips_through_its_name() {
        for icon in KeyIcon::ALL {
            assert_eq!(KeyIcon::from_name(icon.name()), Some(icon), "{icon:?}");
        }
        assert_eq!(KeyIcon::from_name("pencil"), Some(KeyIcon::Edit));
        assert_eq!(KeyIcon::from_name("funel"), None);
        assert_eq!(KeyIcon::from_name(""), None);
    }

    #[test]
    fn roles_name_the_css_variables_the_app_uses() {
        assert_eq!(Role::Palette2.css_var(), "--palette-2");
        assert_eq!(Role::Tertiary.css_var(), "--accent-tertiary");
        assert_eq!(Role::Warning.css_var(), "--color-warning");
    }
}
