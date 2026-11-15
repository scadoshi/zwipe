//! The face every printing in a Printings sheet shows: a two-faced card's front or back, or a
//! meld half melded or not, turned for all the printings at once by the row's own Flip or Meld
//! button ([`PrintingFaceButton`]) or by `v` on the web.
//!
//! The sheet opens on what its row shows: the face the row is on, and melded where the row's
//! half is melded ([`MeldSource::on`]). Turning it in the sheet leaves the row as it was.

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;

use crate::card_details::{FlipButton, MeldButton};
use crate::meld::{MeldFetch, MeldSource, MeldToggle};

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

/// The row's own Flip or Meld button for every printing at once: Flip for a two-faced card, the
/// Meld toggle for a meld half, nothing for a card that does not turn.
#[component]
pub fn PrintingFaceButton(printing_face: PrintingFace) -> Element {
    match printing_face.turn {
        FaceTurn::Flip(face_count) => rsx! {
            FlipButton {
                face_idx: printing_face.face,
                current: (printing_face.face)().min(face_count.saturating_sub(1)),
                face_count,
                on_face_change: None,
            }
        },
        FaceTurn::Meld => rsx! {
            MeldButton {
                on: (printing_face.melded)(),
                on_toggle: move |()| printing_face.turn(),
            }
        },
        FaceTurn::None => rsx! {},
    }
}
