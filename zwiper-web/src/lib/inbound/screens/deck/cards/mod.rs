//! The card screens: Add cards and Remove cards, the swipe surfaces, and
//! the deck cards view, the editor. They share zwipe-ui's stores through
//! [`CardStores`](crate::inbound::card_stores::CardStores).

pub mod add;
pub(crate) mod editor;
pub(crate) mod quick_add;
pub mod remove;

use std::collections::HashMap;

use components::{
    CardDetailsPanel, CardDetailsPanelSkeleton, TOAST_NORMAL, share_face, use_shared_face,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, Toasts};
use uuid::Uuid;
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

/// The top card's details beside it on a wide window, what the phone keeps
/// behind the eye button, or their ghost while the cards load. With
/// `on_printings` the pane offers Printings, as the drawer does. `faces`,
/// when the screen keeps each card's face there, is the face the pane shows
/// and the one its Flip turns.
#[component]
fn SwipeDetails(
    card: Option<Card>,
    #[props(default)] loading: bool,
    #[props(default)] on_printings: Option<EventHandler<()>>,
    #[props(default)] faces: Option<Signal<HashMap<Uuid, usize>>>,
) -> Element {
    let describe_tag = use_describe_tag();
    rsx! {
        div { class: "web-swipe-details",
            if let Some(card) = card {
                SwipeDetailsPanel {
                    key: "{card.scryfall_data.id}",
                    card,
                    on_printings,
                    faces,
                    describe_tag: Some(describe_tag),
                }
            } else if loading {
                CardDetailsPanelSkeleton {}
            }
        }
    }
}

/// One card's details as a swipe screen's pane, keyed by the card so its face is that card's:
/// the face the screen shares in `faces`, which the pane's Flip turns, with `describe_tag`'s Oracle
/// tag descriptions and `body_end` (a commander suggestion's reasons) under the details.
#[component]
pub(crate) fn SwipeDetailsPanel(
    card: Card,
    on_printings: Option<EventHandler<()>>,
    faces: Option<Signal<HashMap<Uuid, usize>>>,
    describe_tag: Option<Callback<String, Option<String>>>,
    #[props(default)] body_end: Option<Element>,
) -> Element {
    let id = card.scryfall_data.id;
    let face = use_shared_face(faces, id);
    rsx! {
        CardDetailsPanel {
            card,
            describe_tag,
            on_printings,
            face,
            on_face_change: share_face(faces, id),
            body_end,
        }
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use dioxus::dioxus_core::NoOpMutations;
    use serde_json::{Value, json};

    use super::*;

    /// A transform card: the fixture with Delver of Secrets' two faces.
    fn delver() -> Card {
        let json = include_str!("../../../../../../../zwipe-ui/src/test_support/card.json");
        let mut card: Value = serde_json::from_str(json).expect("the fixture parses");
        let sd = card.get_mut("scryfall_data").expect("its data");
        sd["oracle_text"] = Value::Null;
        sd["card_faces"] = json!([
            { "object": "card_face", "name": "Delver of Secrets", "mana_cost": "{U}",
              "type_line": "Creature — Human Wizard", "oracle_text": "Upkeep." },
            { "object": "card_face", "name": "Insectile Aberration", "mana_cost": "",
              "type_line": "Creature — Human Insect", "oracle_text": "Flying" }
        ]);
        serde_json::from_value(card).expect("the two-faced card parses")
    }

    #[test]
    fn a_swipe_screen_s_pane_shows_the_face_the_screen_shares_with_its_reasons_under_it() {
        fn app() -> Element {
            let card = delver();
            let faces = use_signal(|| HashMap::from([(card.scryfall_data.id, 1_usize)]));
            rsx! {
                SwipeDetailsPanel {
                    card,
                    on_printings: None,
                    faces: Some(faces),
                    describe_tag: None,
                    body_end: rsx! { p { class: "reasons", "Why this commander" } },
                }
            }
        }
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        let html = dioxus_ssr::render(&dom);
        assert!(
            html.contains("Creature — Human Insect"),
            "the shared back: {html}"
        );
        assert!(html.contains("Why this commander"), "the reasons: {html}");
    }
}
