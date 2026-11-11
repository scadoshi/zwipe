//! A toast a screen's logic asks its client to raise: its kind, its words
//! and how long it stays up. The client draws it.
//!
//! The three bands are orangui's, from orangui-core, so every client agrees
//! on how long a toast lingers.

use std::time::Duration;

use chrono::{DateTime, TimeDelta, Utc};
pub use orangui_core::toast::{LONG, NORMAL, QUICK};

use crate::confirm;

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
    /// The swipe screens' quiet chip word, when it is not the one
    /// [`chip_word`](crate::confirm::chip_word) gives the text.
    pub chip: Option<&'static str>,
}

impl Toast {
    /// A toast of `kind` saying `text` for `duration`.
    #[must_use]
    pub fn new(kind: ToastKind, text: impl Into<String>, duration: Duration) -> Self {
        Self {
            kind,
            text: text.into(),
            duration,
            chip: None,
        }
    }

    /// The same toast with `word` in the quiet chip in place of its text's.
    #[must_use]
    pub fn with_chip(self, word: &'static str) -> Self {
        Self {
            chip: Some(word),
            ..self
        }
    }

    /// The quiet chip's word for this toast.
    #[must_use]
    pub fn chip_word(&self) -> &str {
        match self.chip {
            Some(word) => word,
            None => confirm::chip_word(&self.text),
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

/// Holds back a toast whose words are already up. A failed load raises its
/// error toast again on every retry (a Retry tap, the connection coming
/// back, a second load beside the first). A message shows once until its
/// toast has gone, and a different message still shows.
#[derive(Debug, Default, Clone, PartialEq, Eq)]
pub struct RepeatGate {
    /// Each message up and when its toast goes.
    up: Vec<(String, DateTime<Utc>)>,
}

impl RepeatGate {
    /// A gate with nothing up.
    #[must_use]
    pub const fn new() -> Self {
        Self { up: Vec::new() }
    }

    /// Whether a toast saying `text` at `now`, up for `lasts`, should show:
    /// `false` while a toast with the same words is still up. A shown one is
    /// remembered until it goes, for good when `lasts` runs past the calendar.
    #[must_use]
    pub fn admit(&mut self, text: &str, now: DateTime<Utc>, lasts: Duration) -> bool {
        self.up.retain(|(_, goes)| *goes > now);
        if self.up.iter().any(|(words, _)| words == text) {
            return false;
        }
        let goes = TimeDelta::from_std(lasts)
            .ok()
            .and_then(|lasts| now.checked_add_signed(lasts))
            .unwrap_or(DateTime::<Utc>::MAX_UTC);
        self.up.push((text.to_owned(), goes));
        true
    }
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

#[cfg(test)]
mod repeat_gate_tests {
    use super::*;

    const OFFLINE: &str = "Couldn't reach Zwipe. Check your connection.";

    fn at(seconds: i64) -> DateTime<Utc> {
        DateTime::from_timestamp(seconds, 0).expect("a small timestamp is in range")
    }

    #[test]
    fn a_repeat_while_the_first_is_up_is_held_back() {
        let mut gate = RepeatGate::new();
        assert!(gate.admit(OFFLINE, at(0), NORMAL));
        assert!(!gate.admit(OFFLINE, at(1), NORMAL), "a retry 1s later");
        assert!(
            !gate.admit(OFFLINE, at(4), NORMAL),
            "the connection coming back 4s later"
        );
    }

    #[test]
    fn the_same_words_show_again_once_the_toast_has_gone() {
        let mut gate = RepeatGate::new();
        assert!(gate.admit(OFFLINE, at(0), NORMAL));
        assert!(
            gate.admit(OFFLINE, at(6), NORMAL),
            "NORMAL is 5s, so it has gone by 6s"
        );
    }

    #[test]
    fn different_words_still_show() {
        let mut gate = RepeatGate::new();
        assert!(gate.admit(OFFLINE, at(0), NORMAL));
        assert!(gate.admit("Something went wrong. Try again.", at(1), NORMAL));
    }

    #[test]
    fn a_held_back_repeat_does_not_extend_the_first() {
        let mut gate = RepeatGate::new();
        assert!(gate.admit(OFFLINE, at(0), NORMAL));
        assert!(!gate.admit(OFFLINE, at(4), NORMAL));
        assert!(
            gate.admit(OFFLINE, at(6), NORMAL),
            "the first toast went at 5s"
        );
    }

    #[test]
    fn a_toast_too_long_to_time_is_held_for_good() {
        let mut gate = RepeatGate::new();
        assert!(gate.admit(OFFLINE, at(0), Duration::MAX));
        assert!(!gate.admit(OFFLINE, at(1), NORMAL), "still up 1s later");
    }
}
