//! What a swipe screen says after a swipe, as the user's swipe
//! confirmations setting has it: the last action's short word in a quiet
//! chip at the screen's top left, a toast for each in full, or nothing.

use std::time::Duration;

use components::TOAST_QUICK;
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts, use_toast};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::settings::SwipeConfirmations;
use zwipe_ui::confirm::chip_word;
use zwipe_ui::hint::Role;

/// What a confirmation says of its swipe, which sets its toast's color.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    /// A neutral action: skipped, kept, moved, undone.
    Info,
    /// An action that changed the deck: added, removed, picked.
    Success,
}

/// A swipe screen's confirmations. `Copy`; from [`use_swipe_confirm`].
#[derive(Clone, Copy)]
pub struct SwipeConfirm {
    session: Signal<Option<Session>>,
    toast: Toasts,
    /// The chip's words, with a count that changes on every confirmation so
    /// the same words said twice ease in again.
    said: Signal<Option<(u32, String)>>,
}

impl std::fmt::Debug for SwipeConfirm {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("SwipeConfirm")
            .field("said", &*self.said.peek())
            .finish_non_exhaustive()
    }
}

impl PartialEq for SwipeConfirm {
    fn eq(&self, other: &Self) -> bool {
        self.said == other.said
    }
}

impl SwipeConfirm {
    /// Confirms a neutral action: skipped, kept, moved, undone.
    pub fn info(self, text: impl Into<String>) {
        self.say(Tone::Info, text.into(), None, TOAST_QUICK);
    }

    /// Confirms an action that changed the deck: added, removed, picked.
    pub fn success(self, text: impl Into<String>) {
        self.say(Tone::Success, text.into(), None, TOAST_QUICK);
    }

    /// [`Self::success`] with `chip` in the quiet chip in place of the text's
    /// own word.
    pub fn success_as(self, text: impl Into<String>, chip: &'static str) {
        self.say(Tone::Success, text.into(), Some(chip), TOAST_QUICK);
    }

    /// [`Self::success`], its toast up for `lasting` rather than the quick
    /// band.
    pub fn success_for(self, text: impl Into<String>, lasting: Duration) {
        self.say(Tone::Success, text.into(), None, lasting);
    }

    /// Updates the chip with `text` in chip mode, and raises no toast in any
    /// mode: for a swipe whose own toast gives way to another one.
    pub fn quietly(self, text: impl Into<String>) {
        self.quietly_as(&text.into(), None);
    }

    /// [`Self::quietly`], with `chip` in place of the text's own word when
    /// given.
    pub fn quietly_as(self, text: &str, chip: Option<&'static str>) {
        let shows_chip = self
            .session
            .peek()
            .as_ref()
            .is_some_and(|s| s.settings.swipe_confirmations == SwipeConfirmations::Chip);
        if shows_chip {
            self.set_chip(text, chip);
        }
    }

    /// Shows `chip`, or `text`'s chip word, easing in again even when it
    /// repeats.
    fn set_chip(self, text: &str, chip: Option<&'static str>) {
        let mut said = self.said;
        let count = said.peek().as_ref().map_or(0, |(n, _)| n.wrapping_add(1));
        let word = chip.unwrap_or_else(|| chip_word(text));
        said.set(Some((count, word.to_owned())));
    }

    fn say(self, tone: Tone, text: String, chip: Option<&'static str>, lasting: Duration) {
        let setting = self
            .session
            .peek()
            .as_ref()
            .map(|s| s.settings.swipe_confirmations)
            .unwrap_or_default();
        match setting {
            SwipeConfirmations::Chip => self.set_chip(&text, chip),
            SwipeConfirmations::Toasts => {
                let options = ToastOptions::default().duration(lasting);
                match tone {
                    Tone::Success => self.toast.success(text, options),
                    Tone::Info => self.toast.info(text, options),
                }
            }
            SwipeConfirmations::Off => {}
        }
    }
}

/// The screen's confirmations, starting with nothing said.
pub fn use_swipe_confirm() -> SwipeConfirm {
    SwipeConfirm {
        session: use_context(),
        toast: use_toast(),
        said: use_signal(|| None),
    }
}

/// The swipe screen's top left: its title chip, shown on a wide window and
/// tinted in `tone`, the screen's own action color, then the last
/// confirmation in the quiet chip. Neither takes a click. The quiet chip
/// draws nothing while `at_end`: the pile has run out, and the end's toast
/// replaces the last swipe's line, so the chip would only repeat the swipe
/// before it.
#[component]
pub fn SwipeScreenChips(
    #[props(into)] title: String,
    tone: Role,
    confirm: SwipeConfirm,
    #[props(default)] at_end: bool,
) -> Element {
    let said = (confirm.said)();
    let var = tone.css_var();
    let name = var.strip_prefix("--").unwrap_or(var);
    // The label tags' own tint and outline in the screen's color, both mixed
    // over the page surface so the tag stays opaque over the backdrop.
    let paint = format!(
        "--tone: var({var}); --tone-ink: var(--ink-{name}, var({var})); --tone-base: var(--bg-primary);"
    );
    rsx! {
        div { class: "web-screen-chips",
            span { class: "stat-chip chip-row-label web-screen-chip", style: "{paint}", "{title}" }
            if let (false, Some((count, text))) = (at_end, said) {
                span { key: "{count}", class: "chip-note web-swipe-confirm", "{text}" }
            }
        }
    }
}
