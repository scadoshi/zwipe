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
