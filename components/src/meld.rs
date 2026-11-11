//! The melded card behind a meld half: the toggle a card image and a card
//! details panel share, with the result fetched by the host.
//!
//! Components never call the API. A host that supports meld provides a
//! [`MeldSource`] at its root: the results it has fetched this session,
//! keyed by Scryfall id, and a callback that fetches one. Without a source
//! no meld toggle shows.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::card::scryfall_data::all_parts::Meld;

/// How far a melded card's fetch has come.
#[derive(Debug, Clone, PartialEq)]
pub enum MeldFetch {
    /// On its way.
    Loading,
    /// Fetched.
    Ready(Box<ScryfallData>),
    /// The fetch failed; a later tap tries again.
    Failed,
}

/// The host's melded cards: the session's fetches by Scryfall id, and the
/// callback that starts one, which writes its outcome into `results`.
#[derive(Debug, Clone, Copy)]
pub struct MeldSource {
    /// Each melded card asked for this session, by its Scryfall id.
    pub results: Signal<HashMap<Uuid, MeldFetch>>,
    /// Fetches the melded card with this Scryfall id.
    pub request: Callback<Uuid>,
}

/// Whether the melded card shows: only once it is wanted and fetched. A
/// fetch on its way or failed leaves the half showing.
fn shows_result(wanted: bool, fetch: Option<&MeldFetch>) -> bool {
    wanted && matches!(fetch, Some(MeldFetch::Ready(_)))
}

/// One meld half's toggle, from [`use_meld`].
#[derive(Debug, Clone)]
pub struct MeldToggle {
    meld: Option<Meld>,
    result: Option<ScryfallData>,
    wanted: Signal<bool>,
    /// The melded card's fetch failed; the toggle reads as off.
    failed: bool,
    source: Option<MeldSource>,
}

impl MeldToggle {
    /// Whether the card melds and the host can fetch its result: the toggle
    /// shows.
    #[must_use]
    pub fn available(&self) -> bool {
        self.meld.is_some() && self.source.is_some()
    }

    /// The melded card, while it shows.
    #[must_use]
    pub fn shown(&self) -> Option<&ScryfallData> {
        self.result.as_ref()
    }

    /// Whether the toggle is on: the melded card is wanted and its fetch
    /// has not failed.
    #[must_use]
    pub fn on(&self) -> bool {
        *self.wanted.read() && !self.failed
    }

    /// The other half's name, when known.
    #[must_use]
    pub fn partner(&self) -> Option<&str> {
        self.meld.as_ref().and_then(|m| m.partner_name.as_deref())
    }

    /// Turns the melded card on or off. Turning it on asks the host for it
    /// unless it is fetched or on its way, so after a failed fetch one tap
    /// tries again.
    pub fn toggle(&self) {
        let mut wanted = self.wanted;
        let next = !*wanted.peek() || self.failed;
        wanted.set(next);
        if !next {
            return;
        }
        if let (Some(meld), Some(source)) = (self.meld.as_ref(), self.source) {
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

/// The meld toggle for the card `sd`: off at first, each card its own.
#[must_use]
pub fn use_meld(sd: &ScryfallData) -> MeldToggle {
    let source = try_use_context::<MeldSource>();
    let wanted = use_signal(|| false);
    let meld = source.and_then(|_| sd.meld());
    let failed = meld.as_ref().zip(source).is_some_and(|(m, s)| {
        matches!(s.results.read().get(&m.result_id), Some(MeldFetch::Failed))
    });
    let result = meld.as_ref().zip(source).and_then(|(m, s)| {
        let results = s.results.read();
        let fetch = results.get(&m.result_id);
        if shows_result(*wanted.read(), fetch) {
            match fetch {
                Some(MeldFetch::Ready(card)) => Some((**card).clone()),
                _ => None,
            }
        } else {
            None
        }
    });
    MeldToggle {
        meld,
        result,
        wanted,
        failed,
        source,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_melded_card_shows_only_once_wanted_and_fetched() {
        let card: serde_json::Value =
            serde_json::from_str(include_str!("../../zwipe-ui/src/test_support/card.json"))
                .unwrap();
        let ready = MeldFetch::Ready(Box::new(
            serde_json::from_value(card["scryfall_data"].clone()).unwrap(),
        ));
        assert!(shows_result(true, Some(&ready)), "wanted and in");
        assert!(!shows_result(false, Some(&ready)), "turned off");
        assert!(
            !shows_result(true, Some(&MeldFetch::Loading)),
            "on its way: the half"
        );
        assert!(
            !shows_result(true, Some(&MeldFetch::Failed)),
            "failed: the half, quietly"
        );
        assert!(!shows_result(true, None), "not asked for yet");
    }
}
