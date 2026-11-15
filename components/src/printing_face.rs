//! The face every printing in a Printings sheet shows: a two-faced card's front or back, or a
//! meld half melded or not, turned for all the printings at once by the Flip or Meld button in
//! the sheet's action dock ([`PrintingFaceButton`]) or by `v` on the web.
//!
//! The sheet opens on what its row shows: the face the row is on, and melded where the row's
//! half is melded ([`MeldSource::on`]). Turning it in the sheet leaves the row as it was.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

use crate::icons::{DockIcon, FLIP};
use crate::meld::{MeldFetch, MeldSource, MeldToggle};
use crate::{MELD, data_action, drawer_swap};

/// The face of the card whose Printings opened last. A row or a details drawer hands the face it
/// shows here as its Printings opens ([`hand_face_then`]), and a Printings sheet whose host gives
/// no face of its own opens on it ([`use_opening_face`]), setting it back to the front as it
/// closes. Provided once at the root ([`use_printings_face_root`]); without it every such sheet
/// opens on the front.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintingsFace(pub Signal<usize>);

/// Provides the root's [`PrintingsFace`]. Call once, in the root component.
pub fn use_printings_face_root() {
    use_context_provider(|| PrintingsFace(Signal::new_in_scope(0, ScopeId::ROOT)));
}

/// `on_printings` handing `face`, the face shown, to `shown` first, so the sheet it opens starts
/// on that face.
#[must_use]
pub fn hand_face_then(
    shown: Option<PrintingsFace>,
    face: usize,
    on_printings: EventHandler<()>,
) -> EventHandler<()> {
    EventHandler::new(move |()| {
        if let Some(PrintingsFace(mut handed)) = shown {
            handed.set(face);
        }
        on_printings.call(());
    })
}

/// The face a Printings sheet opens on: `face` when its host gives one (a row it knows), else the
/// face handed to the [`PrintingsFace`] in context, else the front. The handed face goes back to
/// the front each time the sheet closes.
#[must_use]
pub fn use_opening_face(face: Option<usize>, open: Signal<bool>) -> usize {
    let shown = try_use_context::<PrintingsFace>();
    use_effect(move || {
        if !open()
            && let Some(PrintingsFace(mut handed)) = shown
            && *handed.peek() != 0
        {
            handed.set(0);
        }
    });
    face.or_else(|| shown.map(|PrintingsFace(handed)| handed()))
        .unwrap_or(0)
}

/// What turning a card's printings does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FaceTurn {
    /// The card has this many faces, each with its own image: Flip shows the next.
    Flip(usize),
    /// The card is a meld half and the host can fetch melded cards: Meld shows them.
    Meld,
    /// Nothing turns, so no button shows.
    None,
}

impl FaceTurn {
    /// What turning `card` does, with `source` the host's melded cards.
    #[must_use]
    pub fn of(card: &Card, source: Option<MeldSource>) -> Self {
        let faces = card.scryfall_data.face_count();
        if faces > 1 {
            Self::Flip(faces)
        } else if source.is_some() && card.scryfall_data.meld().is_some() {
            Self::Meld
        } else {
            Self::None
        }
    }
}

/// The face a Printings sheet's printings show, from [`use_printing_face`]. Copy, so every
/// handler holds its own.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct PrintingFace {
    /// The face shown, from 0 for the front.
    pub face: Signal<usize>,
    /// Whether the printings show their melded cards.
    pub melded: Signal<bool>,
    /// What turning does.
    pub turn: FaceTurn,
    source: Option<MeldSource>,
}

impl PrintingFace {
    /// Shows the next face, or melds or unmelds, every printing together.
    pub fn turn(self) {
        match self.turn {
            FaceTurn::Flip(count) => {
                let mut face = self.face;
                let next = face
                    .peek()
                    .saturating_add(1)
                    .checked_rem(count)
                    .unwrap_or(0);
                face.set(next);
            }
            FaceTurn::Meld => {
                let mut melded = self.melded;
                let next = !*melded.peek();
                melded.set(next);
            }
            FaceTurn::None => {}
        }
    }

    /// `printings` as they show: each one's melded card in its place while melded and once
    /// fetched, with the printing's own profile; otherwise the printing itself.
    #[must_use]
    pub fn shown(&self, printings: &[Card]) -> Vec<Card> {
        if !(self.melded)() {
            return printings.to_vec();
        }
        printings
            .iter()
            .map(|printing| self.melded_or_own(printing))
            .collect()
    }

    /// `printing`'s melded card, with its profile, once fetched; otherwise `printing`.
    fn melded_or_own(&self, printing: &Card) -> Card {
        let fetched = self
            .source
            .zip(printing.scryfall_data.meld())
            .and_then(|(s, m)| match s.results.read().get(&m.result_id) {
                Some(MeldFetch::Ready(result)) => Some((**result).clone()),
                Some(MeldFetch::Loading | MeldFetch::Failed) | None => None,
            });
        match fetched {
            Some(result) => Card::new(printing.card_profile.clone(), result),
            None => printing.clone(),
        }
    }

    /// Asks the host for each melded card of `printings` it has not fetched or started, while
    /// melded.
    fn request_melded(&self, printings: &[Card]) {
        let Some(source) = self.source else {
            return;
        };
        if !*self.melded.peek() {
            return;
        }
        for meld in printings.iter().filter_map(|p| p.scryfall_data.meld()) {
            let known = matches!(
                source.results.peek().get(&meld.result_id),
                Some(MeldFetch::Ready(_) | MeldFetch::Loading)
            );
            if !known {
                source.request.call(meld.result_id);
            }
        }
    }
}

/// The face the printings of `card` show, the [`MeldSource`] in context fetching each printing's
/// melded card once melded. Each time the sheet `open`s it starts on `face` and melded where
/// `card`'s row is, so a sheet that stays mounted between openings follows its row too.
/// `printings` are the fetched printings.
#[must_use]
pub fn use_printing_face(
    card: &Card,
    face: usize,
    open: Signal<bool>,
    printings: &[Card],
) -> PrintingFace {
    // A flip or meld crossfades every printing shown.
    drawer_swap::use_drawer_swap();
    let source = try_use_context::<MeldSource>();
    let row_melded = MeldToggle::of(&card.scryfall_data, source).on();
    let turn = FaceTurn::of(card, source);
    let printing_face = PrintingFace {
        face: use_signal(move || face),
        melded: use_signal(move || row_melded),
        turn,
        source,
    };
    use_effect(use_reactive!(|face, row_melded| {
        if open() {
            let PrintingFace {
                face: mut shown,
                mut melded,
                ..
            } = printing_face;
            shown.set(face);
            melded.set(row_melded);
        }
    }));
    let printings = printings.to_vec();
    use_effect(use_reactive!(|printings| {
        // Read so a turn asks again.
        let _ = (printing_face.melded)();
        printing_face.request_melded(&printings);
    }));
    printing_face
}

/// The Flip or Meld button for every printing at once, as a button of the sheet's action dock
/// beside Select: Flip for a two-faced card, the Meld toggle for a meld half, lit while on as the
/// dock lights a toggle, and nothing for a card that does not turn. With `divided`, a rule follows
/// it, between it and the dock's next button.
#[component]
pub fn PrintingFaceButton(printing_face: PrintingFace, divided: bool) -> Element {
    let (label, action, path, pressed) = match printing_face.turn {
        FaceTurn::Flip(_) => ("Flip", data_action::FLIP, FLIP, None),
        FaceTurn::Meld => (
            "Meld",
            data_action::MELD,
            MELD,
            Some((printing_face.melded)()),
        ),
        FaceTurn::None => return rsx! {},
    };
    rsx! {
        button {
            r#type: "button",
            class: if pressed == Some(true) { "action-dock-btn action-dock-on" } else { "action-dock-btn" },
            "data-action": action,
            "aria-label": label,
            "aria-pressed": pressed.map(|on| on.to_string()),
            title: label,
            onclick: move |_| printing_face.turn(),
            // Enter presses the button and stays off the sheet's pick.
            onkeydown: move |e: Event<KeyboardData>| {
                if e.key() == Key::Enter {
                    e.stop_propagation();
                }
            },
            DockIcon { path }
        }
        if divided {
            span { class: "action-dock-divider" }
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;

    use super::*;

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        for _ in 0..4 {
            dom.process_events();
            dom.render_immediate(&mut NoOpMutations);
        }
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn a_printings_press_hands_the_shown_face_to_the_sheet_first() {
        fn app() -> Element {
            let shown = use_context_provider(|| PrintingsFace(Signal::new(0)));
            let mut seen = use_signal(|| None);
            let open = EventHandler::new(move |()| seen.set(Some(*shown.0.peek())));
            let press = hand_face_then(Some(shown), 1, open);
            use_hook(move || press.call(()));
            rsx! { p { "{seen():?}" } }
        }
        let html = render(app);
        assert!(
            html.contains("Some(1)"),
            "the sheet opens with the face in hand: {html}"
        );
    }
}
