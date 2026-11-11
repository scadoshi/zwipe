//! The deck cards screen's keys for the card under the cursor (`context/plans/keyboard_shortcuts.md`,
//! "Deck cards screen"): `p` opens its Printings, `m` and `s` move it to the maybeboard and the
//! sideboard, Shift+S stars or unstars it, and `-` and `+` (or `=`) step its quantity. Each does
//! exactly what that row's own button does, and nothing where the row has no such button. With no
//! card hovered, `-` and `+` step the card whose details are open instead.

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::deck::Board;
use zwipe_ui::deck::form::Slot;

use super::editor::{RowInfo, RowKind};

/// A key the deck cards screen answers for the hovered card.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub(crate) enum HoverKey {
    /// `p`: the card's Printings.
    Printings,
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

impl HoverKey {
    /// Whether the key acts on the card whose details are open when no card is hovered.
    #[must_use]
    pub(crate) fn falls_back_to_open_card(self) -> bool {
        matches!(self, Self::Fewer | Self::More)
    }
}

/// The hover key `key` (a `KeyboardEvent.key`) stands for. `s` and `S` differ: Shift+S stars.
#[must_use]
pub(crate) fn hover_key(key: &str) -> Option<HoverKey> {
    match key {
        "p" | "P" => Some(HoverKey::Printings),
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
}

/// What `key` does to `row`, a card on `board`: the action of the row's matching button, or
/// `None` where the row has no such button (a token's Printings, a move to the board the card is
/// already on, a star off the main board, a quantity in the command zone or of a token).
#[must_use]
pub(crate) fn hover_action(key: HoverKey, row: &RowInfo, board: Board) -> Option<HoverAction> {
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
    fn the_keys_are_p_m_s_and_shift_s() {
        assert_eq!(hover_key("p"), Some(HoverKey::Printings));
        assert_eq!(hover_key("P"), Some(HoverKey::Printings));
        assert_eq!(hover_key("m"), Some(HoverKey::ToMaybeboard));
        assert_eq!(hover_key("s"), Some(HoverKey::ToSideboard));
        assert_eq!(hover_key("S"), Some(HoverKey::Star));
        for key in ["d", "f", "/", "Enter", " ", "_"] {
            assert_eq!(hover_key(key), None, "{key}");
        }
    }

    #[test]
    fn p_opens_the_hovered_card_s_printings_as_its_button_does() {
        let main = row(RowKind::Main { mvp: false });
        assert_eq!(
            hover_action(HoverKey::Printings, &main, Board::Deck),
            Some(HoverAction::Printings(Box::new(main.card.clone()), None))
        );
        let zone = row(RowKind::Zone(Slot::Commander));
        assert_eq!(
            hover_action(HoverKey::Printings, &zone, Board::Deck),
            Some(HoverAction::Printings(
                Box::new(zone.card.clone()),
                Some(Slot::Commander)
            )),
            "a command zone card keeps its slot"
        );
        let token = row(RowKind::Token);
        assert_eq!(hover_action(HoverKey::Printings, &token, Board::Deck), None);
    }

    #[test]
    fn m_and_s_move_only_where_the_board_buttons_offer_it() {
        let main = row(RowKind::Main { mvp: false });
        let id = main.card.scryfall_data.id;
        assert_eq!(
            hover_action(HoverKey::ToMaybeboard, &main, Board::Deck),
            Some(HoverAction::Move(id, Board::Maybeboard))
        );
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &main, Board::Deck),
            Some(HoverAction::Move(id, Board::Sideboard))
        );
        let maybe = row(RowKind::Board(Board::Maybeboard));
        let maybe_id = maybe.card.scryfall_data.id;
        assert_eq!(
            hover_action(HoverKey::ToMaybeboard, &maybe, Board::Maybeboard),
            None,
            "a maybeboard card has no To maybeboard button"
        );
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &maybe, Board::Maybeboard),
            Some(HoverAction::Move(maybe_id, Board::Sideboard))
        );
        let side = row(RowKind::Board(Board::Sideboard));
        assert_eq!(
            hover_action(HoverKey::ToSideboard, &side, Board::Sideboard),
            None,
            "a sideboard card has no To sideboard button"
        );
        for kind in [RowKind::Zone(Slot::Commander), RowKind::Token] {
            let other = row(kind);
            assert_eq!(
                hover_action(HoverKey::ToMaybeboard, &other, Board::Deck),
                None
            );
            assert_eq!(
                hover_action(HoverKey::ToSideboard, &other, Board::Deck),
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
                hover_action(HoverKey::Fewer, &stepped, Board::Sideboard),
                Some(HoverAction::Quantity(id, -1))
            );
            assert_eq!(
                hover_action(HoverKey::More, &stepped, Board::Sideboard),
                Some(HoverAction::Quantity(id, 1))
            );
        }
        for kind in [RowKind::Zone(Slot::Commander), RowKind::Token] {
            let other = row(kind);
            assert_eq!(hover_action(HoverKey::Fewer, &other, Board::Deck), None);
            assert_eq!(hover_action(HoverKey::More, &other, Board::Deck), None);
        }
    }

    #[test]
    fn only_the_quantity_keys_fall_back_to_the_open_card() {
        assert!(HoverKey::Fewer.falls_back_to_open_card());
        assert!(HoverKey::More.falls_back_to_open_card());
        for key in [
            HoverKey::Printings,
            HoverKey::ToMaybeboard,
            HoverKey::ToSideboard,
            HoverKey::Star,
        ] {
            assert!(!key.falls_back_to_open_card(), "{key:?}");
        }
    }

    #[test]
    fn shift_s_toggles_the_star_of_a_main_board_card_only() {
        let starred = row(RowKind::Main { mvp: true });
        assert_eq!(
            hover_action(HoverKey::Star, &starred, Board::Deck),
            Some(HoverAction::Star(starred.card.scryfall_data.id, true))
        );
        let plain = row(RowKind::Main { mvp: false });
        assert_eq!(
            hover_action(HoverKey::Star, &plain, Board::Deck),
            Some(HoverAction::Star(plain.card.scryfall_data.id, false))
        );
        for kind in [
            RowKind::Board(Board::Maybeboard),
            RowKind::Zone(Slot::Commander),
            RowKind::Token,
        ] {
            assert_eq!(hover_action(HoverKey::Star, &row(kind), Board::Deck), None);
        }
    }
}
