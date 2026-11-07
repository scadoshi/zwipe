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
        self.say(false, text.into(), TOAST_QUICK);
    }

    /// Confirms an action that changed the deck: added, removed, picked.
    pub fn success(self, text: impl Into<String>) {
        self.say(true, text.into(), TOAST_QUICK);
    }

    /// [`Self::success`], its toast up for `lasting` rather than the quick
    /// band.
    pub fn success_for(self, text: impl Into<String>, lasting: Duration) {
        self.say(true, text.into(), lasting);
    }

    /// Updates the chip with `text` in chip mode, and raises no toast in any
    /// mode: for a swipe whose own toast gives way to another one.
    pub fn quietly(self, text: impl Into<String>) {
        let chip = self
            .session
            .peek()
            .as_ref()
            .is_some_and(|s| s.settings.swipe_confirmations == SwipeConfirmations::Chip);
        if chip {
            let mut said = self.said;
            let count = said.peek().as_ref().map_or(0, |(n, _)| n.wrapping_add(1));
            said.set(Some((count, chip_word(&text.into()).to_owned())));
        }
    }

    fn say(self, success: bool, text: String, lasting: Duration) {
        let setting = self
            .session
            .peek()
            .as_ref()
            .map(|s| s.settings.swipe_confirmations)
            .unwrap_or_default();
        match setting {
            SwipeConfirmations::Chip => {
                let mut said = self.said;
                let count = said.peek().as_ref().map_or(0, |(n, _)| n.wrapping_add(1));
                said.set(Some((count, chip_word(&text).to_owned())));
            }
            SwipeConfirmations::Toasts => {
                let options = ToastOptions::default().duration(lasting);
                if success {
                    self.toast.success(text, options);
                } else {
                    self.toast.info(text, options);
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

/// The swipe screen's top left: its title chip, shown on a wide window, then
/// the last confirmation in the same quiet chip. Neither takes a click.
#[component]
pub fn SwipeScreenChips(#[props(into)] title: String, confirm: SwipeConfirm) -> Element {
    let said = (confirm.said)();
    rsx! {
        div { class: "web-screen-chips",
            span { class: "chip-note web-screen-chip", "{title}" }
            if let Some((count, text)) = said {
                span { key: "{count}", class: "chip-note web-swipe-confirm", "{text}" }
            }
        }
    }
}
