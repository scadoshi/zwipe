//! What a deck's cards view lists, top to bottom, after the filter and group pass and the Boards
//! and Show rows: the owner's deck page and a shared deck's list the same rows in the same order.

use std::collections::HashMap;
use std::hash::BuildHasher;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::Cards;
use zwipe_core::domain::deck::{Board, DeckEntry};

use super::keys::{RowInfo, RowKind};
use super::{Boards, Display, Group};
use crate::deck::form::Slot;

/// A row's quantity, board and star, the first entry for a printing winning.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct RowMeta {
    /// How many copies the entry holds.
    pub qty: i32,
    /// The board the entry is on.
    pub board: Board,
    /// Whether the card is starred.
    pub mvp: bool,
}

/// Each printing's [`RowMeta`] by Scryfall id, from `entries`; the first entry for a printing wins.
#[must_use]
pub fn meta_of(entries: &[DeckEntry]) -> HashMap<Uuid, RowMeta> {
    let mut map: HashMap<Uuid, RowMeta> = HashMap::new();
    for e in entries {
        map.entry(e.deck_card.scryfall_data_id)
            .or_insert_with(|| RowMeta {
                qty: *e.deck_card.quantity,
                board: e.deck_card.board,
                mvp: e.deck_card.mvp_at.is_some(),
            });
    }
    map
}

/// The board the card `id` is on, by `meta`; the main board for a card it does not list.
#[must_use]
pub fn board_of<S: BuildHasher>(meta: &HashMap<Uuid, RowMeta, S>, id: Uuid) -> Board {
    meta.get(&id).map_or(Board::Deck, |m| m.board)
}

/// What the view lists, in order; `rows` flattens it for the drawer and the arrow keys.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct Listed {
    /// The tokens the deck makes, sorted, never filtered.
    pub tokens: Vec<Card>,
    /// The maybeboard's entries.
    pub maybe: Vec<DeckEntry>,
    /// The sideboard's entries.
    pub side: Vec<DeckEntry>,
    /// The command zone's cards in their slots.
    pub zone: Vec<(Slot, Card)>,
    /// The main board's nonland groups.
    pub groups: Vec<Group>,
    /// The main board's lands.
    pub lands: Vec<Card>,
    /// Every row above, top to bottom.
    pub rows: Vec<RowInfo>,
}

/// A main board row, its quantity and star from the deck's entries.
#[must_use]
pub fn main_row_of<S: BuildHasher>(meta: &HashMap<Uuid, RowMeta, S>, card: &Card) -> RowInfo {
    let m = meta.get(&card.scryfall_data.id).copied();
    RowInfo {
        card: card.clone(),
        qty: m.map_or(1, |m| m.qty),
        kind: RowKind::Main {
            mvp: m.is_some_and(|m| m.mvp),
        },
    }
}

/// The sections the view lists, in the app's order: tokens (sorted by the filter's sort, never
/// filtered), the maybeboard and sideboard, the command zone (commander and partner first), then
/// the main board's groups and its lands. `boards` and `show_zone` leave out what they hide.
#[must_use]
pub fn list_rows<S: BuildHasher>(
    shown: &Display,
    meta: &HashMap<Uuid, RowMeta, S>,
    tokens: Vec<Card>,
    filter: &CardQueryBuilder,
    boards: Boards,
    show_zone: bool,
) -> Listed {
    let sort = filter.sort().unwrap_or(CardSortKey::Name);
    let tokens: Vec<Card> = Cards::from(tokens).sorted(sort, filter.direction()).into();
    let kept = |on: bool, list: &Vec<DeckEntry>| if on { list.clone() } else { Vec::new() };
    let maybe = kept(boards.maybe, &shown.maybe);
    let side = kept(boards.side, &shown.side);
    let zone = if show_zone {
        shown.zone.clone()
    } else {
        Vec::new()
    };
    let groups = if boards.main {
        shown.groups.clone()
    } else {
        Vec::new()
    };
    let lands = if boards.main {
        shown.lands.clone()
    } else {
        Vec::new()
    };
    let mut rows: Vec<RowInfo> = tokens
        .iter()
        .map(|card| RowInfo {
            card: card.clone(),
            qty: 1,
            kind: RowKind::Token,
        })
        .collect();
    for (entry, board) in maybe
        .iter()
        .map(|e| (e, Board::Maybeboard))
        .chain(side.iter().map(|e| (e, Board::Sideboard)))
    {
        rows.push(RowInfo {
            card: entry.card.clone(),
            qty: *entry.deck_card.quantity,
            kind: RowKind::Board(board),
        });
    }
    let commanders = zone
        .iter()
        .filter(|(slot, _)| matches!(slot, Slot::Commander | Slot::Partner));
    let others = zone
        .iter()
        .filter(|(slot, _)| !matches!(slot, Slot::Commander | Slot::Partner));
    for (slot, card) in commanders.chain(others) {
        rows.push(RowInfo {
            card: card.clone(),
            qty: 1,
            kind: RowKind::Zone(*slot),
        });
    }
    for card in groups
        .iter()
        .flat_map(|g| g.cards.iter())
        .chain(lands.iter())
    {
        rows.push(main_row_of(meta, card));
    }
    Listed {
        tokens,
        maybe,
        side,
        zone,
        groups,
        lands,
        rows,
    }
}
