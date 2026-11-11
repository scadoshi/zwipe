//! The deck cards screen's keys for one card (`context/plans/keyboard_shortcuts.md`, "Deck cards
//! screen"): `p` opens its Printings, `i` its image, `v` flips it, `d` opens or closes its
//! details, `m` and `s` move it to the maybeboard and the sideboard, Shift+S stars or unstars it,
//! and `-` and `+` (or `=`) step its quantity. Each does exactly what that card's own button does, and nothing where it has no such
//! button. The card is the one under the cursor, else the one whose details are open.

use components::data_action;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_ui::deck::form::Slot;

use super::editor::{RowInfo, RowKind};
use crate::inbound::chrome::shortcuts::{self, CardKey};

/// Every key [`hover_key`] answers, as `KeyboardEvent.key`: the keys that still reach the screen
/// while the Stacks view's card details drawer is open.
pub(crate) const HOVER_KEYS: [&str; 15] = [
    "p", "P", "i", "I", "v", "V", "d", "D", "m", "M", "s", "S", "-", "+", "=",
];

/// A key the deck cards screen answers for the hovered card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoverKey {
    /// `p`: the card's Printings.
    Printings,
    /// `i`: the card's image, as its Image button opens it.
    Image,
    /// `v`: the card's other face, as its Flip button turns it.
    Flip,
    /// `d`: open or close its details, as a click on it does.
    Details,
    /// `m`: to the maybeboard.
    ToMaybeboard,
    /// `s`: to the sideboard.
    ToSideboard,
    /// Shift+S: star or unstar.
    Star,
    /// `-`: one copy fewer.
    Fewer,
    /// `+`, or `=` without Shift: one copy more.
    More,
}

/// The hover key `key` (a `KeyboardEvent.key`) stands for. `s` and `S` differ: Shift+S stars.
#[must_use]
pub(crate) fn hover_key(key: &str) -> Option<HoverKey> {
    if shortcuts::card_key(key) == Some(CardKey::Flip) {
        return Some(HoverKey::Flip);
    }
    match key {
        "p" | "P" => Some(HoverKey::Printings),
        "i" | "I" => Some(HoverKey::Image),
        "d" | "D" => Some(HoverKey::Details),
        "m" | "M" => Some(HoverKey::ToMaybeboard),
        "s" => Some(HoverKey::ToSideboard),
        "S" => Some(HoverKey::Star),
        "-" => Some(HoverKey::Fewer),
        "+" | "=" => Some(HoverKey::More),
        _ => None,
    }
}

/// What a hover key does to one row, as its own button does it.
#[derive(Debug, Clone, PartialEq)]
pub(crate) enum HoverAction {
    /// Open Printings for the card, in the command zone `slot` when it has one.
    Printings(Box<Card>, Option<Slot>),
    /// Move the card to the board.
    Move(Uuid, Board),
    /// Toggle the star of the card, `starred` saying whether it is starred now.
    Star(Uuid, bool),
    /// Change the card's quantity by the step, as the stepper's `-` or `+` does.
    Quantity(Uuid, i32),
    /// Show the card on this face, everywhere it shows: its row or stack image and its details.
    Flip(Uuid, usize),
    /// Open the card's details, or close them where they are open, as a click on it does.
    Details(Uuid),
    /// Press the first of the card's own details buttons marked with these [`data_action`]s,
    /// which does nothing where the card shows none.
    Press(Uuid, &'static [&'static str]),
}

/// What `key` does to `row`, a card on `board` showing face `shown_face`: the action of the row's
/// matching button, or `None` where the row has no such button (a token's Printings, a move to
/// the board the card is already on, a star off the main board, a quantity in the command zone or
/// of a token, a flip of a single-faced card).
#[must_use]
pub(crate) fn hover_action(
    key: HoverKey,
    row: &RowInfo,
    board: Board,
    shown_face: usize,
) -> Option<HoverAction> {
    let card_id = row.card.scryfall_data.id;
    // Only a main board or other board card has the stepper and the board buttons.
    let movable = matches!(row.kind, RowKind::Main { .. } | RowKind::Board(_));
    // The board buttons offer the two boards the card isn't on.
    let move_to =
        |target: Board| (movable && board != target).then_some(HoverAction::Move(card_id, target));
    match key {
        HoverKey::Printings => match row.kind {
            RowKind::Main { .. } | RowKind::Board(_) => {
                Some(HoverAction::Printings(Box::new(row.card.clone()), None))
            }
            RowKind::Zone(slot) => Some(HoverAction::Printings(
                Box::new(row.card.clone()),
                Some(slot),
            )),
            RowKind::Token => None,
        },
        HoverKey::ToMaybeboard => move_to(Board::Maybeboard),
        HoverKey::ToSideboard => move_to(Board::Sideboard),
        HoverKey::Star => match row.kind {
            RowKind::Main { mvp } => Some(HoverAction::Star(card_id, mvp)),
            RowKind::Board(_) | RowKind::Zone(_) | RowKind::Token => None,
        },
        HoverKey::Image => Some(HoverAction::Press(card_id, &[data_action::IMAGE])),
        HoverKey::Flip => {
            // The faces Flip turns through, as the details count them.
            let faces = components::card_face_count(&row.card);
            // A card with one face shows its meld instead, where it melds.
            if faces > 1 {
                let next = shown_face.saturating_add(1).checked_rem(faces).unwrap_or(0);
                Some(HoverAction::Flip(card_id, next))
            } else {
                Some(HoverAction::Press(card_id, &[data_action::MELD]))
            }
        }
        HoverKey::Details => Some(HoverAction::Details(card_id)),
        HoverKey::Fewer => movable.then_some(HoverAction::Quantity(card_id, -1)),
        HoverKey::More => movable.then_some(HoverAction::Quantity(card_id, 1)),
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn card(name: &str) -> Card {
        let mut card: Card = serde_json::from_str(include_str!(
            "../../../../../../../zwipe-ui/src/test_support/card.json"
        ))
        .unwrap();
        let id = Uuid::new_v4();
        card.card_profile.scryfall_data_id = id;
        card.scryfall_data.id = id;
        card.scryfall_data.name = name.to_owned();
        card
    }

    fn row(kind: RowKind) -> RowInfo {
        RowInfo {
            card: card("Sol Ring"),
            qty: 1,
            kind,
        }
    }

    #[test]
    fn every_hover_key_is_listed_for_the_drawer() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                hover_key(&key).is_some(),
                HOVER_KEYS.contains(&key.as_str()),
                "{key}"
            );
        }
    }

    #[test]
    fn v_flips_here_as_on_every_screen() {
        for byte in b' '..=b'~' {
            let key = char::from(byte).to_string();
            assert_eq!(
                hover_key(&key) == Some(HoverKey::Flip),
                shortcuts::card_key(&key) == Some(CardKey::Flip),
                "{key}"
            );
        }
    }

    #[test]
    fn the_keys_are_p_m_s_and_shift_s() {
        assert_eq!(hover_key("p"), Some(HoverKey::Printings));
        assert_eq!(hover_key("P"), Some(HoverKey::Printings));
        assert_eq!(hover_key("m"), Some(HoverKey::ToMaybeboard));
        assert_eq!(hover_key("s"), Some(HoverKey::ToSideboard));
        assert_eq!(hover_key("S"), Some(HoverKey::Star));
        for key in ["f", "/", "Enter", " ", "_", "r", "u"] {
            assert_eq!(hover_key(key), None, "{key}");
        }
    }

    #[test]
    fn p_opens_the_hovered_card_s_printings_as_its_button_does() {
        let main = row(RowKind::Main { mvp: false });
        assert_eq!(
            hover_action(HoverKey::Printings, &main, Board::Deck, 0),
            Some(HoverAction::Printings(Box::new(main.card.clone()), None))
        );
        let zone = row(RowKind::Zone(Slot::Commander));
        assert_eq!(
            hover_action(HoverKey::Printings, &zone, Board::Deck, 0),
            Some(HoverAction::Printings(
                Box::new(zone.card.clone()),
                Some(Slot::Commander)
            )),
            "a command zone card keeps its slot"
        );
        let token = row(RowKind::Token);
        assert_eq!(
            hover_action(HoverKey::Printings, &token, Board::Deck, 0),
            None
        );
    }

    #[test]
    fn m_and_s_move_only_where_the_board_buttons_offer_it() {
        let main = row(RowKind::Main { mvp: false });
        let id = main.card.scryfall_data.id;
        assert_eq!(
            hover_action(HoverKey::ToMaybeboard, &main, Board::Deck, 0),
            Some(HoverAction::Move(id, Board::Maybeboard))
        );
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &main, Board::Deck, 0),
            Some(HoverAction::Move(id, Board::Sideboard))
        );
        let maybe = row(RowKind::Board(Board::Maybeboard));
        let maybe_id = maybe.card.scryfall_data.id;
        assert_eq!(
            hover_action(HoverKey::ToMaybeboard, &maybe, Board::Maybeboard, 0),
            None,
            "a maybeboard card has no To maybeboard button"
        );
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &maybe, Board::Maybeboard, 0),
            Some(HoverAction::Move(maybe_id, Board::Sideboard))
        );
        let side = row(RowKind::Board(Board::Sideboard));
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &side, Board::Sideboard, 0),
            None,
            "a sideboard card has no To sideboard button"
        );
        for kind in [RowKind::Zone(Slot::Commander), RowKind::Token] {
            let other = row(kind);
            assert_eq!(
                hover_action(HoverKey::ToMaybeboard, &other, Board::Deck, 0),
                None
            );
            assert_eq!(
                hover_action(HoverKey::ToSideboard, &other, Board::Deck, 0),
                None
            );
        }
    }

    #[test]
    fn minus_and_plus_step_the_quantity_as_the_stepper_does() {
        assert_eq!(hover_key("-"), Some(HoverKey::Fewer));
        assert_eq!(hover_key("+"), Some(HoverKey::More));
        assert_eq!(hover_key("="), Some(HoverKey::More), "+ without Shift");
        for kind in [
            RowKind::Main { mvp: false },
            RowKind::Board(Board::Sideboard),
        ] {
            let stepped = row(kind);
            let id = stepped.card.scryfall_data.id;
            assert_eq!(
                hover_action(HoverKey::Fewer, &stepped, Board::Sideboard, 0),
                Some(HoverAction::Quantity(id, -1))
            );
            assert_eq!(
                hover_action(HoverKey::More, &stepped, Board::Sideboard, 0),
                Some(HoverAction::Quantity(id, 1))
            );
        }
        for kind in [RowKind::Zone(Slot::Commander), RowKind::Token] {
            let other = row(kind);
            assert_eq!(hover_action(HoverKey::Fewer, &other, Board::Deck, 0), None);
            assert_eq!(hover_action(HoverKey::More, &other, Board::Deck, 0), None);
        }
    }

    #[test]
    fn i_presses_the_image_button_and_d_toggles_the_details() {
        assert_eq!(hover_key("i"), Some(HoverKey::Image));
        assert_eq!(hover_key("d"), Some(HoverKey::Details));
        assert_eq!(hover_key("D"), Some(HoverKey::Details));
        for kind in [
            RowKind::Main { mvp: false },
            RowKind::Board(Board::Maybeboard),
            RowKind::Zone(Slot::Commander),
            RowKind::Token,
        ] {
            let any = row(kind);
            let id = any.card.scryfall_data.id;
            assert_eq!(
                hover_action(HoverKey::Image, &any, Board::Deck, 0),
                Some(HoverAction::Press(id, &[data_action::IMAGE]))
            );
            assert_eq!(
                hover_action(HoverKey::Details, &any, Board::Deck, 0),
                Some(HoverAction::Details(id)),
                "every row opens and closes its details"
            );
        }
    }

    #[test]
    fn v_turns_to_the_next_face_and_back_to_the_front() {
        assert_eq!(hover_key("v"), Some(HoverKey::Flip));
        let mut dfc = row(RowKind::Main { mvp: false });
        dfc.card.scryfall_data.card_faces = Some(
            serde_json::from_value(serde_json::json!([
                { "name": "Front", "object": "card_face", "mana_cost": "", "oracle_text": "a" },
                { "name": "Back", "object": "card_face", "mana_cost": "", "oracle_text": "b" },
            ]))
            .unwrap(),
        );
        dfc.card.scryfall_data.oracle_text = None;
        let id = dfc.card.scryfall_data.id;
        assert_eq!(
            hover_action(HoverKey::Flip, &dfc, Board::Deck, 0),
            Some(HoverAction::Flip(id, 1))
        );
        assert_eq!(
            hover_action(HoverKey::Flip, &dfc, Board::Deck, 1),
            Some(HoverAction::Flip(id, 0)),
            "the back turns to the front"
        );
        let single = row(RowKind::Main { mvp: false });
        assert_eq!(
            hover_action(HoverKey::Flip, &single, Board::Deck, 0),
            Some(HoverAction::Press(
                single.card.scryfall_data.id,
                &[data_action::MELD]
            )),
            "a single-faced card presses its Meld, which does nothing where it has none"
        );
    }

    #[test]
    fn shift_s_toggles_the_star_of_a_main_board_card_only() {
        let starred = row(RowKind::Main { mvp: true });
        assert_eq!(
            hover_action(HoverKey::Star, &starred, Board::Deck, 0),
            Some(HoverAction::Star(starred.card.scryfall_data.id, true))
        );
        let plain = row(RowKind::Main { mvp: false });
        assert_eq!(
            hover_action(HoverKey::Star, &plain, Board::Deck, 0),
            Some(HoverAction::Star(plain.card.scryfall_data.id, false))
        );
        for kind in [
            RowKind::Board(Board::Maybeboard),
            RowKind::Zone(Slot::Commander),
            RowKind::Token,
        ] {
            assert_eq!(
                hover_action(HoverKey::Star, &row(kind), Board::Deck, 0),
                None
            );
        }
    }
}
