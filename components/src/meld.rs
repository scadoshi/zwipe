//! The melded card behind a meld half: the toggle a card image and a card
//! details panel share, with the result fetched by the host.
//!
//! Components never call the API. A host that supports meld provides a
//! [`MeldSource`] at its root: the results it has fetched this session,
//! keyed by Scryfall id, and a callback that fetches one. Without a source
//! no meld toggle shows.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
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

/// The host's melded cards: the session's fetches by Scryfall id, the
/// halves whose Meld is on, and the callback that starts a fetch, which
/// writes its outcome into `results`. A half's Meld is one state wherever the
/// half shows (its image, its details, its row, its stack card).
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct MeldSource {
    /// Each melded card asked for this session, by its Scryfall id.
    pub results: Signal<HashMap<Uuid, MeldFetch>>,
    /// The halves showing their melded card, by the half's Scryfall id.
    pub on: Signal<HashSet<Uuid>>,
    /// Fetches the melded card with this Scryfall id.
    pub request: Callback<Uuid>,
}

/// The melded card that shows: only once it is wanted and fetched. A fetch
/// on its way or failed leaves the half showing.
fn shown_result(wanted: bool, fetch: Option<&MeldFetch>) -> Option<&ScryfallData> {
    match fetch {
        Some(MeldFetch::Ready(card)) if wanted => Some(card),
        _ => None,
    }
}

/// One meld half's toggle, from [`use_meld`].
#[derive(Debug, Clone, PartialEq)]
pub struct MeldToggle {
    meld: Option<Meld>,
    result: Option<ScryfallData>,
    /// The half's Scryfall id, its key in [`MeldSource::on`].
    id: Uuid,
    /// Whether the half's Meld is on.
    wanted: bool,
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

    /// `card` as it shows: the melded card, with `card`'s own profile, while
    /// the toggle is on and the melded card is in; otherwise `card` itself.
    #[must_use]
    pub fn apply(&self, card: &Card) -> Card {
        match self.result.as_ref() {
            Some(result) => Card::new(card.card_profile.clone(), result.clone()),
            None => card.clone(),
        }
    }

    /// Whether the toggle is on: the melded card is wanted and its fetch
    /// has not failed.
    #[must_use]
    pub fn on(&self) -> bool {
        self.wanted && !self.failed
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
        let Some(mut source) = self.source else {
            return;
        };
        let next = !source.on.peek().contains(&self.id) || self.failed;
        if next {
            source.on.write().insert(self.id);
        } else {
            source.on.write().remove(&self.id);
            return;
        }
        if let Some(meld) = self.meld.as_ref() {
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

/// The meld toggle for the card `sd`, from the [`MeldSource`] in context:
/// off at first, and one state wherever the card shows.
#[must_use]
pub fn use_meld(sd: &ScryfallData) -> MeldToggle {
    MeldToggle::of(sd, try_use_context::<MeldSource>())
}

impl MeldToggle {
    /// The meld toggle for the card `sd` under `source`, for a caller outside
    /// a component's body (a key handler); [`use_meld`] in one. Without a
    /// source the card has no toggle.
    #[must_use]
    pub fn of(sd: &ScryfallData, source: Option<MeldSource>) -> Self {
        let meld = source.and_then(|_| sd.meld());
        let wanted = source.is_some_and(|s| s.on.read().contains(&sd.id));
        let failed = meld.as_ref().zip(source).is_some_and(|(m, s)| {
            matches!(s.results.read().get(&m.result_id), Some(MeldFetch::Failed))
        });
        let result = meld.as_ref().zip(source).and_then(|(m, s)| {
            let results = s.results.read();
            shown_result(wanted, results.get(&m.result_id)).cloned()
        });
        Self {
            meld,
            result,
            id: sd.id,
            wanted,
            failed,
            source,
        }
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
        assert!(shown_result(true, Some(&ready)).is_some(), "wanted and in");
        assert!(shown_result(false, Some(&ready)).is_none(), "turned off");
        assert!(
            shown_result(true, Some(&MeldFetch::Loading)).is_none(),
            "on its way: the half"
        );
        assert!(
            shown_result(true, Some(&MeldFetch::Failed)).is_none(),
            "failed: the half, quietly"
        );
        assert!(shown_result(true, None).is_none(), "not asked for yet");
    }
}
