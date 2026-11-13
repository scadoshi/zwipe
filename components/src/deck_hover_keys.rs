//! The deck cards page's hover keys: `p`, `i`, `v` and `d` read the card the details drawer
//! shows, from a stack, a featured card or a row, else the card under the cursor, else the open
//! list row, and on an editable deck `m`, `s`,
//! Shift+S, `-` and `+` edit it, each as the card's own button does. A deck with no edit actions
//! (a shared deck) answers only the keys that read. The keys still reach the page through the
//! Stacks view's details drawer; every other key waits for it to close.

use std::collections::HashMap;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::scryfall_data::ScryfallData;
use zwipe_core::domain::deck::Board;
use zwipe_ui::deck_cards::keys::{self as hover_keys, HoverAction, HoverKey, RowInfo};
use zwipe_ui::shortcuts::Editable;

use crate::deck_cards_view::{DeckEditActions, OnPrinting, StackDetails};
use crate::deck_cursor;
use crate::keys::{STACK_DETAILS_CLASS, use_page_keys_through};
use crate::meld::{MeldSource, MeldToggle};

/// Sends the card the deck cards screen's keys act on: the card the details drawer shows, as
/// `stack:<id>` so `d` closes it, else the keyboard cursor's card when a key moved last
/// ([`crate::deck_cursor`]), else the deepest card under the mouse, as `stack:<id>` for a
/// stack card or a featured card in the deck head or `row:<id>` for a list row, else the list
/// row whose details are open, else an empty string.
const KEY_CARD_JS: &str = "const drawer = document.querySelector('.bottom-sheet.show.__DRAWER__[data-card]'); const keyed = __KEYED__; const under = document.querySelectorAll('[data-row]:hover'); const row = document.querySelector('.web-deck-row.selected[data-row]'); const card = keyed || (under.length ? under[under.length - 1] : row); dioxus.send(drawer ? 'stack:' + drawer.getAttribute('data-card') : card ? (card.matches('.card-stack-slot, .deck-head-card-image') ? 'stack:' : 'row:') + card.getAttribute('data-row') : '');";

/// Clicks list row `__ID__`'s line, which opens or closes its details as a click on it does.
const ROW_TOGGLE_JS: &str = "const line = document.querySelector('.web-deck-row[data-row=\"__ID__\"] .card-row-compact'); if (line) line.click();";

/// Clicks the first details button marked with one of `__ACTIONS__` ([`crate::data_action`]) of
/// card `__ID__`: in the details drawer while it shows that card, else in its list row. Sends
/// whether it found the button.
const PRESS_JS: &str = "const host = document.querySelector('.bottom-sheet.show.__DRAWER__[data-card=\"__ID__\"]') || document.querySelector('.web-deck-row[data-row=\"__ID__\"]'); const button = host && __ACTIONS__.map((a) => host.querySelector('button.card-action-btn[data-action=\"' + a + '\"]')).find(Boolean); if (button) button.click(); dioxus.send(!!button);";

/// What the deck page hands its hover keys.
#[derive(Clone, Copy, PartialEq)]
#[expect(
    missing_debug_implementations,
    reason = "callbacks and edit actions, which have no Debug"
)]
pub struct DeckHoverKeys {
    /// The row of the card with this Scryfall id, with the board it sits on, while it shows.
    pub row_of: Callback<Uuid, Option<(RowInfo, Board)>>,
    /// Each card's shown face, by Scryfall id; `v` turns it.
    pub faces: Signal<HashMap<Uuid, usize>>,
    /// The Stacks view's reveal and drawer, where the page has Stacks; `d` on a stack card opens
    /// its drawer.
    pub details: Option<StackDetails>,
    /// The deck's edits; `None` answers only the keys that read.
    pub actions: Option<DeckEditActions>,
    /// Opens a card's printings.
    pub on_printing: OnPrinting,
    /// Opens a card's image on a face, for a stack card whose drawer is shut and so shows no
    /// Image button.
    pub on_image: Callback<(ScryfallData, usize)>,
    /// Every key the hover keys don't answer, as `KeyboardEvent.key`: only the newest page key
    /// listener hears the keys, so the page's own keys come through here.
    pub other_keys: Option<Callback<String>>,
}

/// Whether a deck with `actions` takes the keys that edit.
fn editable(actions: Option<&DeckEditActions>) -> Editable {
    if actions.is_some() {
        Editable::Yes
    } else {
        Editable::No
    }
}

/// Answers the deck page's hover keys as [`DeckHoverKeys`] describes. A meld half's `v` toggles
/// its Meld through the [`MeldSource`] in context.
pub fn use_deck_hover_keys(keys: DeckHoverKeys) {
    let DeckHoverKeys {
        row_of,
        mut faces,
        details,
        actions,
        on_printing,
        on_image,
        other_keys,
    } = keys;
    let editable = editable(actions.as_ref());
    let meld_source = try_use_context::<MeldSource>();
    deck_cursor::use_deck_cursor(details);
    use_page_keys_through(
        STACK_DETAILS_CLASS,
        hover_keys::hover_keys(editable),
        Callback::new(move |key: String| {
            let Some(hover) = hover_keys::hover_key(&key, editable) else {
                if let Some(other_keys) = other_keys {
                    other_keys.call(key);
                }
                return;
            };
            spawn(async move {
                let mut found = document::eval(
                    &KEY_CARD_JS
                        .replace("__DRAWER__", STACK_DETAILS_CLASS)
                        .replace("__KEYED__", &deck_cursor::keyed_js()),
                );
                let Ok(found) = found.recv::<String>().await else {
                    return;
                };
                let Some((kind, id)) = found.split_once(':') else {
                    return;
                };
                let on_stack = kind == "stack";
                let Ok(id) = Uuid::try_parse(id) else {
                    return;
                };
                let Some((row, board)) = row_of.call(id) else {
                    return;
                };
                let shown_face = faces.peek().get(&id).copied().unwrap_or(0);
                match hover_keys::hover_action(hover, &row, board, shown_face) {
                    Some(HoverAction::Printings(card, slot)) => on_printing.call((*card, slot)),
                    Some(HoverAction::Move(card_id, target)) => {
                        if let Some(actions) = actions {
                            actions
                                .move_to_board
                                .call((card_id, target, /* record */ true));
                        }
                    }
                    Some(HoverAction::Star(card_id, starred)) => {
                        if let Some(actions) = actions {
                            actions.toggle_mvp.call((card_id, starred));
                        }
                    }
                    Some(HoverAction::Quantity(card_id, step)) => {
                        if let Some(actions) = actions {
                            actions
                                .change_quantity
                                .call((card_id, step, /* record */ true));
                        }
                    }
                    Some(HoverAction::Flip(card_id, next)) => {
                        faces.write().insert(card_id, next);
                    }
                    Some(HoverAction::Meld(_)) => {
                        MeldToggle::of(&row.card.scryfall_data, meld_source).toggle();
                    }
                    Some(HoverAction::Details(card_id)) => match details {
                        Some(details) if on_stack => details.click(card_id),
                        _ => {
                            document::eval(&ROW_TOGGLE_JS.replace("__ID__", &card_id.to_string()));
                        }
                    },
                    Some(HoverAction::Image(card_id)) => {
                        let mut pressed = document::eval(
                            &PRESS_JS
                                .replace("__DRAWER__", STACK_DETAILS_CLASS)
                                .replace("__ID__", &card_id.to_string())
                                .replace(
                                    "__ACTIONS__",
                                    &format!("{:?}", [crate::data_action::IMAGE]),
                                ),
                        );
                        let found = pressed.recv::<bool>().await.unwrap_or(true);
                        // A stack card with its drawer shut shows no Image button: its image
                        // opens on the face it shows.
                        if !found && hover == HoverKey::Image {
                            on_image.call((row.card.scryfall_data.clone(), shown_face));
                        }
                    }
                    None => {}
                }
            });
        }),
    );
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn a_hovered_featured_card_takes_the_keys_as_a_stack_card() {
        assert!(
            KEY_CARD_JS
                .contains("card.matches('.card-stack-slot, .deck-head-card-image') ? 'stack:'"),
            "{KEY_CARD_JS}"
        );
    }

    #[test]
    fn the_card_the_drawer_shows_stays_the_keyed_card_over_any_hovered_one() {
        let js = KEY_CARD_JS
            .replace("__DRAWER__", STACK_DETAILS_CLASS)
            .replace("__KEYED__", &deck_cursor::keyed_js());
        let drawer = format!(".bottom-sheet.show.{STACK_DETAILS_CLASS}[data-card]");
        assert!(js.contains(&drawer), "the drawer names its card: {js}");
        assert!(
            js.contains("drawer ? 'stack:' + drawer.getAttribute('data-card')"),
            "the drawer's card comes first, as a stack card so d closes it: {js}"
        );
        assert!(
            !js.contains(".card-stack-slot.open"),
            "a featured card or a list row's drawer has no open stack slot: {js}"
        );
    }

    #[test]
    fn a_key_presses_the_button_in_the_drawer_showing_its_card() {
        let js = PRESS_JS.replace("__DRAWER__", STACK_DETAILS_CLASS);
        assert!(
            js.contains(&format!(
                ".bottom-sheet.show.{STACK_DETAILS_CLASS}[data-card=\"__ID__\"]"
            )),
            "{js}"
        );
        assert!(!js.contains(".card-stack-slot.open"), "{js}");
    }

    #[test]
    fn a_key_presses_its_button_by_data_action_not_its_words() {
        assert!(PRESS_JS.contains("__ACTIONS__.map("), "{PRESS_JS}");
        assert!(PRESS_JS.contains("[data-action="), "{PRESS_JS}");
        assert!(!PRESS_JS.contains("textContent"), "{PRESS_JS}");
    }

    #[test]
    fn only_a_deck_with_edit_actions_takes_the_keys_that_edit() {
        assert_eq!(editable(None), Editable::No);
        let read_only = hover_keys::hover_keys(editable(None));
        for key in ["m", "s", "S", "-", "+"] {
            assert!(!read_only.contains(&key), "{key} on a shared deck");
        }
        for key in ["p", "i", "v", "d"] {
            assert!(read_only.contains(&key), "{key} on a shared deck");
        }
    }
}
