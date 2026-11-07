//! A toast a screen's logic asks its client to raise: its kind, its words
//! and how long it stays up. The client draws it.
//!
//! The three bands are orangui's, from orangui-core, so every client agrees
//! on how long a toast lingers.

use std::time::Duration;

pub use orangui_core::toast::{LONG, NORMAL, QUICK};

/// How a toast is colored and announced.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Hash)]
pub enum ToastKind {
    /// Neutral news.
    Info,
    /// Something worked.
    Success,
    /// Something was refused or needs a look.
    Warning,
    /// Something failed.
    Error,
}

/// One toast to raise.
#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct Toast {
    /// Its kind.
    pub kind: ToastKind,
    /// Its words, verbatim.
    pub text: String,
    /// How long it stays up: [`QUICK`], [`NORMAL`] or [`LONG`].
    pub duration: Duration,
}

impl Toast {
    /// A toast of `kind` saying `text` for `duration`.
    #[must_use]
    pub fn new(kind: ToastKind, text: impl Into<String>, duration: Duration) -> Self {
        Self {
            kind,
            text: text.into(),
            duration,
        }
    }

    /// A routine toast, up for [`QUICK`].
    #[must_use]
    pub fn quick(kind: ToastKind, text: impl Into<String>) -> Self {
        Self::new(kind, text, QUICK)
    }

    /// A toast worth reading, up for [`NORMAL`].
    #[must_use]
    pub fn normal(kind: ToastKind, text: impl Into<String>) -> Self {
        Self::new(kind, text, NORMAL)
    }
}

/// What one screen effect raises, for [`keep_one`].
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Line<'a> {
    /// A toast.
    Toast(&'a Toast),
    /// A swipe's confirmation, which the user's setting may also raise as a toast.
    Confirm(&'a Toast),
}

impl Line<'_> {
    /// Which line wins: a toast over a confirmation, then the more severe.
    fn rank(self) -> (u8, u8) {
        let (toast, inner) = match self {
            Self::Toast(toast) => (1, toast),
            Self::Confirm(toast) => (0, toast),
        };
        let severity = match inner.kind {
            ToastKind::Success => 0,
            ToastKind::Info => 1,
            ToastKind::Warning => 2,
            ToastKind::Error => 3,
        };
        (toast, severity)
    }
}

/// Keeps one of the lines an action raised and drops the rest, so one action never shows two
/// toasts: a toast over a swipe's confirmation (it says more, and the swipe went through), then the
/// more severe, then the first. Every other effect stays, in order.
pub fn keep_one<E>(effects: &mut Vec<E>, line: impl for<'e> Fn(&'e E) -> Option<Line<'e>>) {
    let mut best: Option<(usize, (u8, u8))> = None;
    for (index, effect) in effects.iter().enumerate() {
        if let Some(found) = line(effect) {
            let rank = found.rank();
            if best.is_none_or(|(_, kept)| rank > kept) {
                best = Some((index, rank));
            }
        }
    }
    let Some((keep, _)) = best else {
        return;
    };
    let mut index = 0_usize;
    effects.retain(|effect| {
        let stays = index == keep || line(effect).is_none();
        index = index.saturating_add(1);
        stays
    });
}

#[cfg(test)]
mod keep_one_tests {
    use super::*;

    #[derive(Debug, Clone, PartialEq)]
    enum Effect {
        Toast(Toast),
        Confirm(Toast),
        Fetch,
    }

    fn line(effect: &Effect) -> Option<Line<'_>> {
        match effect {
            Effect::Toast(toast) => Some(Line::Toast(toast)),
            Effect::Confirm(toast) => Some(Line::Confirm(toast)),
            Effect::Fetch => None,
        }
    }

    #[test]
    fn a_toast_wins_over_the_swipes_confirmation() {
        let mut effects = vec![
            Effect::Confirm(Toast::quick(ToastKind::Success, "Removed from deck")),
            Effect::Fetch,
            Effect::Toast(Toast::quick(ToastKind::Info, "No cards left")),
        ];
        keep_one(&mut effects, line);
        assert_eq!(
            effects,
            [
                Effect::Fetch,
                Effect::Toast(Toast::quick(ToastKind::Info, "No cards left"))
            ]
        );
    }

    #[test]
    fn the_more_severe_toast_wins_then_the_first() {
        let mut effects = vec![
            Effect::Toast(Toast::normal(ToastKind::Info, "Land target reached")),
            Effect::Toast(Toast::normal(ToastKind::Warning, "Over budget")),
            Effect::Toast(Toast::normal(ToastKind::Warning, "Another warning")),
        ];
        keep_one(&mut effects, line);
        assert_eq!(
            effects,
            [Effect::Toast(Toast::normal(
                ToastKind::Warning,
                "Over budget"
            ))]
        );
    }

    #[test]
    fn one_line_or_none_is_left_alone() {
        let mut effects = vec![
            Effect::Fetch,
            Effect::Confirm(Toast::quick(ToastKind::Success, "Added")),
        ];
        let before = effects.clone();
        keep_one(&mut effects, line);
        assert_eq!(effects, before);
        let mut none = vec![Effect::Fetch];
        keep_one(&mut none, line);
        assert_eq!(none, [Effect::Fetch]);
    }
}
