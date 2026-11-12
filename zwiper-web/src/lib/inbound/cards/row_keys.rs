//! The card keys of a screen of card rows with no editing (Playstyle's
//! lists, Activity's Most skipped, the Commander maybeboard): `p` opens the card's Printings, `i` its
//! image, `v` flips it (or toggles a meld half's Meld) and `d` opens or
//! closes its details, each as its own button or a click on it does. The
//! card is the one under the cursor, else the one whose details are open.
//! Shift+H lists them ([`ShortcutsScreen::CardRows`]).

use components::{MeldSource, MeldToggle};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;

use crate::inbound::cards::card_row::CardOverlays;
use crate::inbound::cards::stack_view::{RowFaces, StackDetails};
use crate::inbound::chrome::page_keys::use_page_keys_through;
use crate::inbound::chrome::shortcuts::{self, CardKey, STACK_DETAILS_CLASS, ShortcutsScreen};
use crate::inbound::chrome::shortcuts_sheet::use_offer_shortcuts;

/// Every key [`row_key`] answers, as `KeyboardEvent.key`: the keys that still reach the screen
/// while a stack card's details drawer is open.
pub(crate) const ROW_KEYS: [&str; 8] = ["p", "P", "i", "I", "v", "V", "d", "D"];

/// A key a card row screen answers for the card under the cursor.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum RowKey {
    /// `p`: the card's Printings.
    Printings,
    /// `i`: the card's image.
    Image,
    /// `v`: the card's other face, or a meld half's Meld.
    Flip,
    /// `d`: open or close its details.
    Details,
}

/// The row key `key` (a `KeyboardEvent.key`) stands for; `v` and `d` as every screen reads them
/// ([`shortcuts::card_key`]).
#[must_use]
pub(crate) fn row_key(key: &str) -> Option<RowKey> {
    match shortcuts::card_key(key) {
        Some(CardKey::Flip) => return Some(RowKey::Flip),
        Some(CardKey::Details) => return Some(RowKey::Details),
        Some(CardKey::Filter) | None => {}
    }
    match key {
        "p" | "P" => Some(RowKey::Printings),
        "i" | "I" => Some(RowKey::Image),
        _ => None,
    }
}

/// Sends the card under the cursor, else the stack card whose drawer is open, as `stack:<id>`
/// or `row:<id>`, else an empty string.
const KEY_CARD_JS: &str = "const under = document.querySelectorAll('[data-row]:hover'); const open = document.querySelector('.bottom-sheet.show.__DRAWER__') && document.querySelector('.card-stack-slot.open[data-row]'); const card = under.length ? under[under.length - 1] : open; dioxus.send(card ? (card.classList.contains('card-stack-slot') ? 'stack:' : 'row:') + card.getAttribute('data-row') : '');";

/// Answers [`ROW_KEYS`] on a screen listing `cards` as rows (expanding `expanded_card`), as
/// stacks, or both in different sections, and offers the screen's shortcuts on Shift+H. A key
/// acts on the card under the cursor, a stack card or a row, else on the open stack card, else
/// on the expanded row. The image and
/// Printings open in the screen's [`CardOverlays`], faces turn in its [`RowFaces`] and the
/// stacks' details move in its [`StackDetails`], each where the screen provides it.
pub(crate) fn use_card_row_keys(cards: Memo<Vec<Card>>, expanded_card: Signal<Option<Uuid>>) {
    use_card_row_keys_and(cards, expanded_card, None);
}

/// [`use_card_row_keys`] on a screen with keys of its own: a key that is not a card key goes
/// to `screen_keys`, since only the newest page key listener hears the keys.
pub(crate) fn use_card_row_keys_and(
    cards: Memo<Vec<Card>>,
    expanded_card: Signal<Option<Uuid>>,
    screen_keys: Option<Callback<String>>,
) {
    let overlays = try_use_context::<CardOverlays>();
    let details = try_use_context::<StackDetails>();
    let faces = try_use_context::<RowFaces>();
    let melds = try_use_context::<MeldSource>();
    use_offer_shortcuts(ShortcutsScreen::CardRows);
    use_page_keys_through(
        STACK_DETAILS_CLASS,
        &ROW_KEYS,
        Callback::new(move |key: String| {
            let Some(pressed) = row_key(&key) else {
                if let Some(screen_keys) = screen_keys {
                    screen_keys.call(key);
                }
                return;
            };
            spawn(async move {
                let mut found =
                    document::eval(&KEY_CARD_JS.replace("__DRAWER__", STACK_DETAILS_CLASS));
                let hovered = found.recv::<String>().await.ok().and_then(|found| {
                    let (kind, id) = found.split_once(':')?;
                    Some((kind == "stack", Uuid::try_parse(id).ok()?))
                });
                let open_stack = details
                    .and_then(|d| d.reveal.peek().open())
                    .map(|id| (true, id));
                let open_row = (*expanded_card.peek()).map(|id| (false, id));
                let Some((stacks, id)) = hovered.or(open_stack).or(open_row) else {
                    return;
                };
                let Some(card) = cards
                    .peek()
                    .iter()
                    .find(|c| c.scryfall_data.id == id)
                    .cloned()
                else {
                    return;
                };
                let face = faces.map_or(0, |f| f.of(id));
                match pressed {
                    RowKey::Printings => {
                        if let Some(CardOverlays { mut printing, .. }) = overlays {
                            printing.set(Some(card));
                        }
                    }
                    RowKey::Image => {
                        if let Some(CardOverlays { mut preview, .. }) = overlays {
                            preview.set(Some((card.scryfall_data, face)));
                        }
                    }
                    RowKey::Flip => {
                        let count = components::card_face_count(&card);
                        if count > 1 {
                            if let Some(faces) = faces {
                                let next = face.saturating_add(1).checked_rem(count).unwrap_or(0);
                                faces.set(id, next);
                            }
                        } else if card.scryfall_data.meld().is_some() {
                            MeldToggle::of(&card.scryfall_data, melds).toggle();
                        }
                    }
                    RowKey::Details => match details {
                        Some(details) if stacks => details.click(id),
                        _ => {
                            let mut expanded = expanded_card;
                            let next = (*expanded.peek() != Some(id)).then_some(id);
                            expanded.set(next);
                        }
                    },
                }
            });
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn every_row_key_is_listed_for_the_drawer() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                row_key(&key).is_some(),
                ROW_KEYS.contains(&key.as_str()),
                "{key}"
            );
        }
    }

    #[test]
    fn v_and_d_read_as_on_every_screen() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                row_key(&key) == Some(RowKey::Flip),
                shortcuts::card_key(&key) == Some(CardKey::Flip),
                "{key}"
            );
            assert_eq!(
                row_key(&key) == Some(RowKey::Details),
                shortcuts::card_key(&key) == Some(CardKey::Details),
                "{key}"
            );
        }
    }

    #[test]
    fn the_keys_are_p_i_v_and_d_and_nothing_that_edits() {
        assert_eq!(row_key("p"), Some(RowKey::Printings));
        assert_eq!(row_key("I"), Some(RowKey::Image));
        for key in ["f", "m", "s", "S", "-", "+", "u", "/", "Enter"] {
            assert_eq!(row_key(key), None, "{key}");
        }
    }

    #[test]
    fn the_shortcuts_sheet_lists_the_row_keys() {
        let listed: Vec<&str> = shortcuts::sheet_rows(&ShortcutsScreen::CardRows)
            .into_iter()
            .flat_map(|(keys, _)| keys.iter().copied())
            .collect();
        for key in ["p", "i", "v", "d", "Shift+H"] {
            assert!(listed.contains(&key), "{key}");
        }
        assert!(!listed.contains(&"f"), "these screens have no filter");
    }
}
