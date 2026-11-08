//! The card screens: Add cards and Remove cards, the swipe surfaces, and
//! the deck cards view, the editor. They share zwipe-ui's stores through
//! [`CardStores`](crate::inbound::card_stores::CardStores).

pub mod add;
pub(crate) mod editor;
mod featured;
pub(crate) mod quick_add;
pub mod remove;
pub mod view;

use components::{CardDetailsPanel, CardDetailsPanelSkeleton, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts};
use zwipe_core::domain::card::Card;
use zwipe_ui::deck_signals::Level;
use zwipe_ui::stack::NO_CARDS_LEFT;

use crate::inbound::cards::otag::use_describe_tag;
use crate::inbound::cards::swipe_confirm::{SwipeConfirm, Tone};

/// Raises a land or budget signal from zwipe-ui as the app's toast: info or
/// warning, up for the normal band.
fn signal_toast(toast: Toasts, signal: Option<(Level, String)>) {
    let Some((level, message)) = signal else {
        return;
    };
    let options = ToastOptions::default().duration(TOAST_NORMAL);
    match level {
        Level::Info => toast.info(message, options),
        Level::Warning => toast.warning(message, options),
    }
}

/// What one swipe raised, held so it reaches the user as one toast: a land,
/// budget or pile signal over the swipe's own confirmation, the warning over
/// the info, then the first. In chip mode the confirmation still updates the
/// chip.
#[derive(Debug, Default)]
pub(crate) struct OneLine {
    said: Option<(Tone, String)>,
    /// The quiet chip's word for the confirmation, when not its text's.
    chip: Option<&'static str>,
    signals: Vec<(Level, String)>,
}

impl OneLine {
    /// The swipe's own confirmation, in its `tone`.
    pub(crate) fn confirm(&mut self, tone: Tone, text: impl Into<String>) {
        self.said = Some((tone, text.into()));
    }

    /// [`Self::confirm`] with `chip` in the quiet chip in place of the
    /// text's own word.
    pub(crate) fn confirm_as(&mut self, tone: Tone, text: impl Into<String>, chip: &'static str) {
        self.said = Some((tone, text.into()));
        self.chip = Some(chip);
    }

    /// A signal the swipe set off, if any.
    pub(crate) fn signal(&mut self, signal: Option<(Level, String)>) {
        self.signals.extend(signal);
    }

    /// The stack ran out under this swipe: says [`NO_CARDS_LEFT`] in place of
    /// the swipe's own line, whatever the confirmation setting.
    pub(crate) fn ran_out(&mut self) {
        self.signals.push((Level::Info, NO_CARDS_LEFT.to_owned()));
    }

    /// Raises the one line that wins.
    pub(crate) fn raise(self, confirm: SwipeConfirm, toast: Toasts) {
        let warning = self
            .signals
            .iter()
            .position(|(level, _)| matches!(level, Level::Warning));
        let winner = warning
            .or_else(|| (!self.signals.is_empty()).then_some(0))
            .and_then(|at| self.signals.into_iter().nth(at));
        match (winner, self.said) {
            (Some(signal), said) => {
                if let Some((_, text)) = said {
                    confirm.quietly_as(&text, self.chip);
                }
                signal_toast(toast, Some(signal));
            }
            (None, Some((Tone::Success, text))) => match self.chip {
                Some(chip) => confirm.success_as(text, chip),
                None => confirm.success(text),
            },
            (None, Some((Tone::Info, text))) => confirm.info(text),
            (None, None) => {}
        }
    }
}

/// Presses the flip control of the top swipe card, if it has one.
fn flip_top_card() {
    document::eval(
        r#"const flip = document.querySelector('.swipe-stack-card.top [aria-label="Flip card"]'); if (flip) flip.click();"#,
    );
}

/// The top card's details beside it on a wide window, what the phone keeps
/// behind the eye button, or their ghost while the cards load. With
/// `on_printings` the pane offers Printings, as the drawer does.
#[component]
fn SwipeDetails(
    card: Option<Card>,
    #[props(default)] loading: bool,
    #[props(default)] on_printings: Option<EventHandler<()>>,
) -> Element {
    let describe_tag = use_describe_tag();
    rsx! {
        div { class: "web-swipe-details",
            if let Some(card) = card {
                CardDetailsPanel {
                    key: "{card.scryfall_data.id}",
                    card,
                    describe_tag: Some(describe_tag),
                    on_printings,
                }
            } else if loading {
                CardDetailsPanelSkeleton {}
            }
        }
    }
}
