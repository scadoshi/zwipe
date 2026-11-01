//! The deck cards view's logic: the filter and group pass, the featured strip, quantity bursts, and what each undo entry needs.

use std::collections::HashMap;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::cards::{Cards, sort_deck_entries};
use zwipe_core::domain::card::search_card::group_cards::{CardGroup, GroupByOption, GroupCards};
use zwipe_core::domain::deck::{Board, DeckEntry};

use crate::deck::form::Slot;
use crate::stores::undo::UndoAction;

/// The command zone's cards by slot.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Zone {
    /// The commander or oathbreaker.
    pub commander: Option<Card>,
    /// The partner.
    pub partner: Option<Card>,
    /// The background.
    pub background: Option<Card>,
    /// The signature spell.
    pub signature: Option<Card>,
}

impl Zone {
    /// The card in `slot`, if any.
    #[must_use]
    pub fn slot(&self, slot: Slot) -> Option<&Card> {
        match slot {
            Slot::Commander => self.commander.as_ref(),
            Slot::Partner => self.partner.as_ref(),
            Slot::Background => self.background.as_ref(),
            Slot::SignatureSpell => self.signature.as_ref(),
        }
    }

    /// The card in `slot`, to set or clear.
    pub fn slot_mut(&mut self, slot: Slot) -> &mut Option<Card> {
        match slot {
            Slot::Commander => &mut self.commander,
            Slot::Partner => &mut self.partner,
            Slot::Background => &mut self.background,
            Slot::SignatureSpell => &mut self.signature,
        }
    }

    /// Each filled slot and its card.
    #[must_use]
    pub fn cards(&self) -> Vec<(Slot, &Card)> {
        [
            Slot::Commander,
            Slot::Partner,
            Slot::Background,
            Slot::SignatureSpell,
        ]
        .into_iter()
        .filter_map(|slot| self.slot(slot).map(|card| (slot, card)))
        .collect()
    }

    /// Drops every entry that shares an oracle ID with a command zone card, which never appear in the groups.
    pub fn strip(&self, entries: &mut Vec<DeckEntry>) {
        let oracles: Vec<Uuid> = self
            .cards()
            .iter()
            .filter_map(|(_, c)| c.scryfall_data.oracle_id)
            .collect();
        entries.retain(|e| !oracles.contains(&e.deck_card.oracle_id));
    }
}

/// Which boards the view lists.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Boards {
    /// The main board.
    pub main: bool,
    /// The maybeboard.
    pub maybe: bool,
    /// The sideboard.
    pub side: bool,
}

impl Default for Boards {
    fn default() -> Self {
        Self {
            main: true,
            maybe: false,
            side: false,
        }
    }
}

impl Boards {
    /// Toggles one board; turning the last one off turns Main back on.
    pub fn toggle(&mut self, board: Board) {
        match board {
            Board::Deck => self.main = !self.main,
            Board::Maybeboard => self.maybe = !self.maybe,
            Board::Sideboard => self.side = !self.side,
        }
        if !self.main && !self.maybe && !self.side {
            self.main = true;
        }
    }
}

/// What the view lists after the filter and group pass.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Display {
    /// The main board's nonland cards in groups.
    pub groups: Vec<Group>,
    /// The main board's lands.
    pub lands: Vec<Card>,
    /// The maybeboard.
    pub maybe: Vec<DeckEntry>,
    /// The sideboard.
    pub side: Vec<DeckEntry>,
    /// The command zone cards the filter keeps.
    pub zone: Vec<(Slot, Card)>,
}

/// One card group.
#[derive(Debug, Clone, PartialEq)]
pub struct Group {
    /// The group's header.
    pub label: String,
    /// Pips in place of the header, for a color group.
    pub pips: Option<Vec<zwipe_core::domain::card::scryfall_data::colors::Color>>,
    /// The group's cards.
    pub cards: Vec<Card>,
    /// The collapse key: the label, or the pips for a colored group.
    pub key: String,
}

impl From<CardGroup> for Group {
    fn from(group: CardGroup) -> Self {
        let key = group.key();
        Self {
            label: group.label,
            pips: group.pips,
            cards: group.cards,
            key,
        }
    }
}

/// The filter and group pass: active cards filtered and sorted (Name when unsorted), lands pulled out, the rest grouped; the maybeboard and sideboard filtered and sorted; the command zone filtered.
pub fn pass(
    entries: &[DeckEntry],
    zone: &Zone,
    filter: &CardQueryBuilder,
    group_by: GroupByOption,
) -> Display {
    let criteria = if filter.is_empty() {
        None
    } else {
        filter.build_criteria().ok()
    };
    let keep = |card: &Card| criteria.as_ref().is_none_or(|c| c.matches(card));
    let sort = filter.sort().unwrap_or(CardSortKey::Name);
    let direction = filter.direction();

    let active: Vec<Card> = entries
        .iter()
        .filter(|e| e.deck_card.board.is_active())
        .map(|e| e.card.clone())
        .collect();
    let mut active = Cards::from(active);
    if let Some(criteria) = &criteria {
        active = active.matching(criteria);
    }
    let active: Vec<Card> = active.sorted(sort, direction).into();
    let (lands, nonlands): (Vec<Card>, Vec<Card>) =
        active.into_iter().partition(|c| c.scryfall_data.is_land());

    let board = |board: Board| -> Vec<DeckEntry> {
        let mut list: Vec<DeckEntry> = entries
            .iter()
            .filter(|e| e.deck_card.board == board && keep(&e.card))
            .cloned()
            .collect();
        sort_deck_entries(&mut list, sort, direction);
        list
    };
    Display {
        groups: nonlands
            .group_by(group_by)
            .into_iter()
            .map(Group::from)
            .collect(),
        lands,
        maybe: board(Board::Maybeboard),
        side: board(Board::Sideboard),
        zone: zone
            .cards()
            .into_iter()
            .filter(|(_, card)| keep(card))
            .map(|(slot, card)| (slot, card.clone()))
            .collect(),
    }
}

/// The featured strip: the command zone with its role, then the main board's MVPs oldest star first.
#[must_use]
pub fn featured(
    zone: &Zone,
    entries: &[DeckEntry],
    oathbreaker: bool,
) -> Vec<(Card, &'static str)> {
    let mut featured: Vec<(Card, &'static str)> = Vec::new();
    let roles = [
        (
            zone.commander.as_ref(),
            if oathbreaker {
                "Oathbreaker"
            } else {
                "Commander"
            },
        ),
        (zone.partner.as_ref(), "Partner"),
        (zone.background.as_ref(), "Background"),
        (zone.signature.as_ref(), "Signature spell"),
    ];
    for (card, role) in roles {
        if let Some(card) = card {
            featured.push((card.clone(), role));
        }
    }
    let mut mvps: Vec<&DeckEntry> = entries
        .iter()
        .filter(|e| e.deck_card.mvp_at.is_some() && e.deck_card.board == Board::Deck)
        .collect();
    mvps.sort_by_key(|e| e.deck_card.mvp_at);
    featured.extend(mvps.into_iter().map(|e| (e.card.clone(), "MVP")));
    featured
}

/// A burst of quantity taps on one card, waiting to be written.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Burst {
    /// The quantity before the burst.
    pub baseline: i32,
    /// The taps' net change in copies.
    pub delta: i32,
    /// Bumped by each tap, so only the last tap's timer writes.
    pub generation: u64,
    /// Whether the write records an undo entry; undo-driven bursts do not.
    pub record: bool,
}

/// What a burst's write does.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Write {
    /// The taps cancelled out.
    Nothing,
    /// The quantity fell below one: delete the card.
    Delete,
    /// Set the absolute quantity, which a replay cannot double.
    Set(i32),
}

impl Burst {
    /// What the burst writes when its timer fires.
    #[must_use]
    pub fn write(&self) -> Write {
        let target = self.baseline.saturating_add(self.delta);
        if self.delta == 0 {
            Write::Nothing
        } else if target < 1 {
            Write::Delete
        } else {
            Write::Set(target)
        }
    }
}

/// Every card's open burst.
#[derive(Debug, Clone, Default, PartialEq)]
pub struct Bursts {
    open: HashMap<Uuid, Burst>,
}

impl Bursts {
    /// Folds a tap into the card's burst and returns its generation for the timer.
    pub fn tap(&mut self, card_id: Uuid, current: i32, delta: i32, record: bool) -> u64 {
        let burst = self.open.entry(card_id).or_insert(Burst {
            baseline: current,
            delta: 0,
            generation: 0,
            record,
        });
        burst.delta = burst.delta.saturating_add(delta);
        burst.generation = burst.generation.wrapping_add(1);
        burst.generation
    }

    /// The quantity before the card's open burst, if it has one.
    #[must_use]
    pub fn baseline(&self, card_id: Uuid) -> Option<i32> {
        self.open.get(&card_id).map(|b| b.baseline)
    }

    /// Takes the burst when `generation` is still its latest tap.
    pub fn due(&mut self, card_id: Uuid, generation: u64) -> Option<Burst> {
        if self.open.get(&card_id)?.generation != generation {
            return None;
        }
        self.open.remove(&card_id)
    }

    /// Takes every open burst, for leaving the screen.
    pub fn drain(&mut self) -> Vec<(Uuid, Burst)> {
        self.open.drain().collect()
    }
}

/// Whether an undo entry no longer applies to the deck as it stands, and the button should only say "Already changed".
#[must_use]
pub fn is_stale(action: &UndoAction, entries: &[DeckEntry], zone: &Zone) -> bool {
    let present = |id: Uuid| entries.iter().any(|e| e.deck_card.scryfall_data_id == id);
    match action {
        UndoAction::Added { card_id, .. }
        | UndoAction::QuantityChanged { card_id, .. }
        | UndoAction::MovedBoard { card_id, .. } => !present(*card_id),
        UndoAction::Removed { entry, .. } => present(entry.deck_card.scryfall_data_id),
        UndoAction::PrintingChanged { new_id, .. } => !present(*new_id),
        UndoAction::CommandZonePrintingChanged { slot, new_id, .. } => {
            zone.slot(*slot).map(|c| c.scryfall_data.id) != Some(*new_id)
        }
    }
}

/// The toast an undo entry raises when the button reverses it.
#[must_use]
pub fn undo_message(action: &UndoAction) -> String {
    match action {
        UndoAction::Added { card_name, .. } => format!("Removed {card_name}"),
        UndoAction::Removed { entry, .. } => format!("Re-added {}", entry.card.scryfall_data.name),
        UndoAction::QuantityChanged {
            card_name,
            baseline,
            ..
        } => format!("{card_name} back to ×{baseline}"),
        UndoAction::MovedBoard {
            card_name, from, ..
        } => format!("{card_name} back to {}", from.display_name()),
        UndoAction::PrintingChanged { old_card, .. }
        | UndoAction::CommandZonePrintingChanged { old_card, .. } => {
            format!("{} printing restored", old_card.scryfall_data.name)
        }
    }
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};

    use super::*;
    use crate::test_support::{card, deck, deck_profile};

    fn typed(name: &str, type_line: &str) -> Card {
        let mut c = card(name);
        c.scryfall_data.type_line = Some(type_line.to_owned());
        c
    }

    fn entries() -> Vec<DeckEntry> {
        let mut d = deck(
            deck_profile("D"),
            vec![
                typed("Zebra", "Creature — Horse"),
                typed("Forest", "Basic Land — Forest"),
                typed("Ant", "Creature — Insect"),
                typed("Maybe", "Instant"),
                typed("Side", "Sorcery"),
            ],
        );
        d.entries[3].deck_card.board = Board::Maybeboard;
        d.entries[4].deck_card.board = Board::Sideboard;
        d.entries
    }

    fn names(cards: &[Card]) -> Vec<&str> {
        cards
            .iter()
            .map(|c| c.scryfall_data.name.as_str())
            .collect()
    }

    #[test]
    fn the_pass_sorts_by_name_pins_lands_and_splits_boards() {
        let display = pass(
            &entries(),
            &Zone::default(),
            &CardQueryBuilder::default(),
            GroupByOption::CardType,
        );
        assert_eq!(names(&display.lands), ["Forest"]);
        let creatures = display.groups.iter().find(|g| !g.cards.is_empty()).unwrap();
        assert_eq!(names(&creatures.cards), ["Ant", "Zebra"]);
        assert_eq!(display.maybe.len(), 1);
        assert_eq!(display.side.len(), 1);
    }

    #[test]
    fn the_filter_narrows_every_list_and_the_zone() {
        let zone = Zone {
            commander: Some(typed("Boss", "Legendary Creature — Goblin")),
            ..Zone::default()
        };
        let mut filter = CardQueryBuilder::default();
        filter.set_name_contains("a");
        let display = pass(&entries(), &zone, &filter, GroupByOption::Cmc);
        let shown: Vec<&str> = display
            .groups
            .iter()
            .flat_map(|g| names(&g.cards))
            .collect();
        assert_eq!(shown, ["Ant", "Zebra"]);
        assert_eq!(display.lands.len(), 0);
        assert_eq!(display.maybe.len(), 1, "Maybe has an a");
        assert_eq!(display.side.len(), 0);
        assert!(display.zone.is_empty(), "the commander is filtered out too");
    }

    #[test]
    fn the_zone_strips_its_cards_from_the_entries() {
        let mut list = entries();
        let zone = Zone {
            commander: Some(list[0].card.clone()),
            ..Zone::default()
        };
        zone.strip(&mut list);
        assert_eq!(list.len(), 4);
    }

    #[test]
    fn boards_keep_one_on() {
        let mut boards = Boards::default();
        boards.toggle(Board::Deck);
        assert!(boards.main, "the last one snaps back");
        boards.toggle(Board::Sideboard);
        boards.toggle(Board::Deck);
        assert!(!boards.main && boards.side);
    }

    #[test]
    fn featured_lists_the_zone_then_mvps_by_star_time() {
        let mut list = entries();
        let now = Utc::now();
        list[0].deck_card.mvp_at = Some(now);
        list[2].deck_card.mvp_at = Some(now - Duration::hours(1));
        list[3].deck_card.mvp_at = Some(now);
        let zone = Zone {
            commander: Some(card("Boss")),
            ..Zone::default()
        };
        let featured = featured(&zone, &list, true);
        let shown: Vec<(&str, &str)> = featured
            .iter()
            .map(|(c, r)| (c.scryfall_data.name.as_str(), *r))
            .collect();
        assert_eq!(
            shown,
            [("Boss", "Oathbreaker"), ("Ant", "MVP"), ("Zebra", "MVP")]
        );
    }

    #[test]
    fn a_burst_writes_once_with_its_net() {
        let card = Uuid::new_v4();
        let mut bursts = Bursts::default();
        bursts.tap(card, 2, 1, true);
        let generation = bursts.tap(card, 3, 1, true);
        assert_eq!(bursts.baseline(card), Some(2));
        assert!(
            bursts.due(card, generation - 1).is_none(),
            "an older tap's timer"
        );
        let burst = bursts.due(card, generation).unwrap();
        assert_eq!(burst.write(), Write::Set(4));
        assert!(bursts.due(card, generation).is_none());

        bursts.tap(card, 1, -1, false);
        let generation = bursts.tap(card, 0, 1, false);
        assert_eq!(
            bursts.due(card, generation).unwrap().write(),
            Write::Nothing
        );
        bursts.tap(card, 1, -1, true);
        assert_eq!(bursts.drain()[0].1.write(), Write::Delete);
    }

    #[test]
    fn undo_entries_go_stale_when_the_deck_moved_on() {
        let list = entries();
        let zone = Zone::default();
        let present = list[0].deck_card.scryfall_data_id;
        let added = UndoAction::Added {
            card_id: present,
            card_name: "Zebra".to_owned(),
        };
        assert!(!is_stale(&added, &list, &zone));
        assert!(is_stale(
            &UndoAction::Added {
                card_id: Uuid::new_v4(),
                card_name: "Gone".to_owned()
            },
            &list,
            &zone
        ));
        let removed = UndoAction::Removed {
            entry: Box::new(list[0].clone()),
            baseline: 2,
        };
        assert!(is_stale(&removed, &list, &zone), "already back");
        assert_eq!(undo_message(&removed), "Re-added Zebra");
        assert_eq!(
            undo_message(&UndoAction::QuantityChanged {
                card_id: present,
                card_name: "Zebra".to_owned(),
                net: 1,
                baseline: 2
            }),
            "Zebra back to ×2"
        );
        assert_eq!(
            undo_message(&UndoAction::MovedBoard {
                card_id: present,
                card_name: "Zebra".to_owned(),
                from: Board::Maybeboard
            }),
            "Zebra back to maybeboard"
        );
        let slot = Zone {
            commander: Some(card("Boss")),
            ..Zone::default()
        };
        let zone_change = UndoAction::CommandZonePrintingChanged {
            slot: Slot::Commander,
            old_card: Box::new(card("Boss")),
            new_id: Uuid::new_v4(),
        };
        assert!(is_stale(&zone_change, &list, &slot));
        assert_eq!(undo_message(&zone_change), "Boss printing restored");
    }
}
