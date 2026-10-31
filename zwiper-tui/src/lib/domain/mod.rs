//! What the screens draw, free of terminal and network code.

/// Orangui's rolling figures and compact numbers.
pub(crate) mod count_up;
/// The logo entrance, ported from orangui.
pub(crate) mod decode;
/// The privacy policy's HTML as text.
pub(crate) mod legal;
/// Orangui's palettes, mapped to the roles the screens draw with.
pub(crate) mod theme;
