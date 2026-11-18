//! The deck cards view's logic: the filter and group pass, the featured strip, quantity bursts, and what each undo entry needs.

use std::collections::HashMap;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardSortKey;
use zwipe_core::domain::card::search_card::card_filter::builder::CardQueryBuilder;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::card::search_card::cards::{Cards, sort_deck_entries};
use zwipe_core::domain::card::search_card::group_cards::{CardGroup, GroupByOption, GroupCards};
use zwipe_core::domain::deck::deck_metrics::DeckMetrics;
use zwipe_core::domain::deck::validate_deck::{DeckCommandZone, validate_deck};
use zwipe_core::domain::deck::{Board, Deck, DeckEntry, DeckProfile, DeckWarning, Format};

use crate::deck::form::Slot;
use crate::stores::undo::UndoAction;

pub mod columns;
pub mod cursor;
pub mod keys;
pub mod listing;
pub mod stacks;

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
    /// A deck's command zone from the cards it carries, each matched to its slot by the
    /// profile's printing id.
    #[must_use]
    pub fn of_deck(deck: &Deck) -> Self {
        let profile = &deck.deck_profile;
        let find = |id: Option<Uuid>| -> Option<Card> {
            let id = id?;
            deck.command_zone_cards
                .iter()
                .find(|c| c.scryfall_data.id == id)
                .cloned()
        };
        Self {
            commander: find(profile.commander_id),
            partner: find(profile.partner_commander_id),
            background: find(profile.background_id),
            signature: find(profile.signature_spell_id),
        }
    }

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

    /// The zone's cards at the printings it holds, in slot order: the command zone cards a fresh
    /// fetch would return.
    #[must_use]
    pub fn to_cards(&self) -> Vec<Card> {
        self.cards()
            .into_iter()
            .map(|(_, card)| card.clone())
            .collect()
    }

    /// Drops every entry that shares an oracle ID with a command zone card, which never appear in the groups.
    pub fn strip(&self, entries: &mut Vec<DeckEntry>) {
        let oracles = self.oracles();
        entries.retain(|e| !oracles.contains(&e.deck_card.oracle_id));
    }

    /// Whether the Show row offers the Command zone chip: the deck's `format` has a command zone,
    /// or this zone holds a card anyway.
    #[must_use]
    pub fn offers_chip(&self, format: Option<Format>) -> bool {
        format.is_some_and(|f| f.has_commander()) || !self.cards().is_empty()
    }

    /// The oracle IDs of the zone's cards.
    fn oracles(&self) -> Vec<Uuid> {
        self.cards()
            .iter()
            .filter_map(|(_, c)| c.scryfall_data.oracle_id)
            .collect()
    }

    /// The deck's metrics as a fresh fetch would give them while the screen holds `live`: its entries as edited since the fetch, with this zone [stripped](Self::strip). The stripped entries come back from `fetched`, the deck's entries as fetched, and the zone's cards count once each at the printings it holds, so a printing changed here prices the deck at once. `None` when the deck holds no card at all.
    #[must_use]
    pub fn live_metrics(&self, fetched: &[DeckEntry], live: &[DeckEntry]) -> Option<DeckMetrics> {
        let all = self.rejoin(fetched, live);
        let zone_cards = self.to_cards();
        (!all.is_empty() || !zone_cards.is_empty())
            .then(|| DeckMetrics::from_entries_and_command_zone(&all, &zone_cards))
    }

    /// The cards with no price in `currency` while the screen holds `live`, from the entries and
    /// zone [`Self::live_metrics`] prices, so they are the copies its totals leave out.
    #[must_use]
    pub fn live_unpriced(
        &self,
        fetched: &[DeckEntry],
        live: &[DeckEntry],
        currency: PriceCurrency,
    ) -> Vec<Card> {
        crate::deck::warning::unpriced(&self.rejoin(fetched, live), &self.to_cards(), currency)
    }

    /// The deck's warnings as a fresh fetch would give them while the screen holds `live`, worked out as the server does: the entries rejoined as for [`Self::live_metrics`], `fetched_profile` counting the main board's copies as they stand, and the command zone as this zone holds it.
    #[must_use]
    pub fn live_warnings(
        &self,
        fetched_profile: &DeckProfile,
        fetched: &[DeckEntry],
        live: &[DeckEntry],
    ) -> Vec<DeckWarning> {
        let all = self.rejoin(fetched, live);
        let card_count = all
            .iter()
            .filter(|e| e.deck_card.board == Board::Deck)
            .map(|e| i64::from(*e.deck_card.quantity))
            .fold(0i64, i64::saturating_add);
        let id = |card: Option<&Card>| card.map(|c| c.scryfall_data.id);
        let profile = DeckProfile {
            card_count,
            commander_id: id(self.commander.as_ref()),
            partner_commander_id: id(self.partner.as_ref()),
            background_id: id(self.background.as_ref()),
            signature_spell_id: id(self.signature.as_ref()),
            ..fetched_profile.clone()
        };
        let zone = DeckCommandZone {
            commander: self.commander.as_ref(),
            partner_commander: self.partner.as_ref(),
            background: self.background.as_ref(),
            signature_spell: self.signature.as_ref(),
        };
        validate_deck(&profile, &all, &zone)
    }

    /// `live` with the fetched entries [stripped](Self::strip) for this zone put back.
    fn rejoin(&self, fetched: &[DeckEntry], live: &[DeckEntry]) -> Vec<DeckEntry> {
        let oracles = self.oracles();
        let mut all: Vec<DeckEntry> = fetched
            .iter()
            .filter(|e| oracles.contains(&e.deck_card.oracle_id))
            .cloned()
            .collect();
        all.extend_from_slice(live);
        all
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

/// What a click or a second tap on a featured card opens.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FeaturedOpen {
    /// Its details, in the drawer or sheet the same card opens from the list.
    Details,
    /// Its image: the card is filtered out or its group hidden, so no listed card has details to
    /// show.
    Image,
}

/// What featured card `id` opens, with `listed` the cards the page lists now.
#[must_use]
pub fn featured_open(id: Uuid, listed: &[Uuid]) -> FeaturedOpen {
    if listed.contains(&id) {
        FeaturedOpen::Details
    } else {
        FeaturedOpen::Image
    }
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

/// How many main board cards a deck may star as MVPs; the server refuses a fourth.
pub const MAX_MVPS: usize = 3;

/// How many main board cards in `entries` are starred.
#[must_use]
pub fn mvp_count(entries: &[DeckEntry]) -> usize {
    entries
        .iter()
        .filter(|e| e.deck_card.mvp_at.is_some() && e.deck_card.board == Board::Deck)
        .count()
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
        UndoAction::MvpToggled {
            card_id, before, ..
        } => entries
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == *card_id)
            .is_none_or(|e| e.deck_card.mvp_at.is_some() == before.is_some()),
        UndoAction::FlagToggled {
            card_id, before, ..
        } => entries
            .iter()
            .find(|e| e.deck_card.scryfall_data_id == *card_id)
            .is_none_or(|e| e.deck_card.flagged == *before),
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
        UndoAction::MvpToggled {
            card_name,
            before: None,
            ..
        } => format!("Unstarred {card_name}"),
        UndoAction::MvpToggled {
            card_name,
            before: Some(_),
            ..
        } => format!("Starred {card_name}"),
        UndoAction::FlagToggled {
            card_name,
            before: false,
            ..
        } => format!("Unflagged {card_name}"),
        UndoAction::FlagToggled {
            card_name,
            before: true,
            ..
        } => format!("Flagged {card_name}"),
        UndoAction::PrintingChanged { old_card, .. }
        | UndoAction::CommandZonePrintingChanged { old_card, .. } => {
            format!("{} printing restored", old_card.scryfall_data.name)
        }
    }
}

/// The Show chip that narrows the deck to its flagged cards.
pub const FLAGGED: &str = "Flagged";

/// Whether Flagged stays on, `on` now with `flagged` cards flagged: it turns itself off once
/// nothing is flagged, and never back on by itself.
#[must_use]
pub const fn flagged_only_stays(on: bool, flagged: usize) -> bool {
    on && flagged > 0
}

/// The printing ids of the flagged cards in `entries`.
#[must_use]
pub fn flagged_ids(entries: &[DeckEntry]) -> std::collections::HashSet<Uuid> {
    entries
        .iter()
        .filter(|e| e.deck_card.flagged)
        .map(|e| e.deck_card.scryfall_data_id)
        .collect()
}

/// `cards` with the ones `entries` flags first, each half in the order it came: what Remove deals,
/// so the shortlist is waiting.
#[must_use]
pub fn flagged_first(cards: Vec<Card>, entries: &[DeckEntry]) -> Vec<Card> {
    let flagged = flagged_ids(entries);
    let (first, rest): (Vec<Card>, Vec<Card>) = cards
        .into_iter()
        .partition(|c| flagged.contains(&c.scryfall_data.id));
    first.into_iter().chain(rest).collect()
}

/// `entries` narrowed to the flagged cards when `flagged_only`, else all of them.
#[must_use]
pub fn shown_flagged(entries: &[DeckEntry], flagged_only: bool) -> Vec<DeckEntry> {
    entries
        .iter()
        .filter(|e| !flagged_only || e.deck_card.flagged)
        .cloned()
        .collect()
}

#[cfg(test)]
mod tests {
    use chrono::{Duration, Utc};
    use zwipe_core::domain::deck::{Format, Quantity};

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
    fn remove_deals_the_flagged_cards_first_each_half_in_its_order() {
        let mut list = entries();
        list[2].deck_card.flagged = true;
        list[4].deck_card.flagged = true;
        let cards: Vec<Card> = list.iter().map(|e| e.card.clone()).collect();
        assert_eq!(
            names(&flagged_first(cards, &list)),
            ["Ant", "Side", "Zebra", "Forest", "Maybe"]
        );
    }

    #[test]
    fn the_flagged_chip_counts_and_narrows_to_the_flagged_cards() {
        let mut list = entries();
        list[0].deck_card.flagged = true;
        assert_eq!(FLAGGED, "Flagged");
        assert_eq!(flagged_ids(&list).len(), 1);
        assert!(flagged_only_stays(true, 1));
        assert!(
            !flagged_only_stays(true, 0),
            "it turns off with nothing flagged"
        );
        assert!(!flagged_only_stays(false, 1), "and never turns itself on");
        let narrowed = shown_flagged(&list, true);
        assert_eq!(narrowed.len(), 1);
        assert_eq!(narrowed[0].card.scryfall_data.name, "Zebra");
        assert_eq!(shown_flagged(&list, false).len(), list.len());
    }

    #[test]
    fn a_flag_undo_reads_back_and_goes_stale_once_the_flag_changed_back() {
        let mut list = entries();
        let id = list[0].deck_card.scryfall_data_id;
        let flagged = UndoAction::FlagToggled {
            card_id: id,
            card_name: "Zebra".to_owned(),
            before: false,
        };
        assert_eq!(undo_message(&flagged), "Unflagged Zebra");
        assert!(
            is_stale(&flagged, &list, &Zone::default()),
            "already unflagged"
        );
        list[0].deck_card.flagged = true;
        assert!(!is_stale(&flagged, &list, &Zone::default()));
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
    fn live_metrics_match_the_fetch_until_an_edit_and_follow_it() {
        let fetched = entries();
        let zone = Zone {
            commander: Some(fetched[0].card.clone()),
            ..Zone::default()
        };
        let zone_cards = vec![fetched[0].card.clone()];
        let mut live = fetched.clone();
        zone.strip(&mut live);
        let as_fetched = DeckMetrics::from_entries_and_command_zone(&fetched, &zone_cards);
        let unedited = zone.live_metrics(&fetched, &live).unwrap();
        assert_eq!(unedited.total_cards, as_fetched.total_cards);
        assert_eq!(unedited.total_price_usd, as_fetched.total_price_usd);
        assert_eq!(unedited.type_counts, as_fetched.type_counts);

        let mut added = deck(deck_profile("D"), vec![typed("Bee", "Creature — Insect")]).entries;
        live.append(&mut added);
        let after_add = zone.live_metrics(&fetched, &live).unwrap();
        assert_eq!(after_add.total_cards, as_fetched.total_cards + 1);

        live.retain(|e| e.card.scryfall_data.name != "Ant" && e.card.scryfall_data.name != "Bee");
        let after_remove = zone.live_metrics(&fetched, &live).unwrap();
        assert_eq!(after_remove.total_cards, as_fetched.total_cards - 1);
    }

    #[test]
    fn the_command_zone_chip_shows_where_there_is_a_zone() {
        let empty = Zone::default();
        assert!(!empty.offers_chip(Some(Format::Modern)));
        assert!(!empty.offers_chip(None));
        assert!(empty.offers_chip(Some(Format::Commander)));
        assert!(empty.offers_chip(Some(Format::Oathbreaker)));
        let held = Zone {
            commander: Some(card("Boss")),
            ..Zone::default()
        };
        assert!(
            held.offers_chip(Some(Format::Modern)),
            "a card in the zone keeps it"
        );
    }

    #[test]
    fn live_warnings_follow_a_card_moved_to_the_main_board() {
        let mut profile = deck_profile("D");
        profile.format = Some(Format::Modern);
        let fetched = entries();
        profile.card_count = 3;
        let mut live = fetched.clone();
        let zone = Zone::default();
        let counts = |warnings: Vec<DeckWarning>| -> Vec<String> {
            warnings
                .iter()
                .map(ToString::to_string)
                .filter(|m| m.contains(" cards, "))
                .collect()
        };
        let before = counts(zone.live_warnings(&profile, &fetched, &live));
        assert_eq!(before.len(), 1, "{before:?}");
        assert!(before[0].starts_with("Deck has 3 cards"), "{before:?}");
        live[3].deck_card.board = Board::Deck;
        let after = counts(zone.live_warnings(&profile, &fetched, &live));
        assert_eq!(after.len(), 1, "{after:?}");
        assert!(after[0].starts_with("Deck has 4 cards"), "{after:?}");
    }

    #[test]
    fn live_warnings_follow_a_quantity_change_and_a_move_to_the_sideboard() {
        let mut profile = deck_profile("D");
        profile.format = Some(Format::Commander);
        let mut fetched = entries();
        fetched.retain(|e| e.deck_card.board == Board::Deck);
        let mut live = fetched.clone();
        let zone = Zone::default();
        let says = |live: &[DeckEntry], words: &str| {
            zone.live_warnings(&profile, &fetched, live)
                .iter()
                .any(|w| w.to_string().contains(words))
        };
        assert!(!says(&live, "copies"));
        assert!(!says(&live, "sideboards"));

        let zebra = live
            .iter_mut()
            .find(|e| e.card.scryfall_data.name == "Zebra")
            .unwrap();
        zebra.deck_card.quantity = Quantity::new(2).unwrap();
        assert!(
            says(&live, "Zebra has 2 copies") || says(&live, "copy limit"),
            "+ raises the copy limit"
        );

        live[0].deck_card.board = Board::Sideboard;
        assert!(
            says(&live, "sideboards"),
            "To sideboard raises the no-sideboard warning"
        );
    }

    #[test]
    fn live_warnings_follow_to_mainboard_to_maybeboard_and_a_removal() {
        let mut profile = deck_profile("D");
        profile.format = Some(Format::Commander);
        let mut fetched = entries();
        fetched.retain(|e| e.deck_card.board == Board::Deck);
        profile.card_count = fetched
            .iter()
            .map(|e| i64::from(*e.deck_card.quantity))
            .sum();
        let zone = Zone::default();
        let warnings = |live: &[DeckEntry]| -> Vec<String> {
            zone.live_warnings(&profile, &fetched, live)
                .iter()
                .map(ToString::to_string)
                .collect()
        };
        let as_fetched = warnings(&fetched);
        let mut live = fetched.clone();

        live[0].deck_card.board = Board::Sideboard;
        assert!(warnings(&live).iter().any(|w| w.contains("sideboards")));
        live[0].deck_card.board = Board::Deck;
        assert_eq!(warnings(&live), as_fetched, "To mainboard clears it");

        live[0].deck_card.board = Board::Maybeboard;
        let moved = warnings(&live);
        assert!(
            !moved.iter().any(|w| w.contains("sideboards")),
            "the maybeboard is not a sideboard: {moved:?}"
        );
        assert_ne!(
            moved, as_fetched,
            "To maybeboard drops a card from the count"
        );

        let removed: Vec<DeckEntry> = fetched.iter().skip(1).cloned().collect();
        assert_eq!(
            warnings(&removed),
            moved,
            "− to 0 counts as the card leaving"
        );
    }

    /// `card` at another printing: its own id and USD price, the same oracle.
    fn reprint(card: &Card, usd: Option<&str>) -> Card {
        let mut other = card.clone();
        other.scryfall_data.id = Uuid::new_v4();
        other.scryfall_data.prices.usd = usd.map(str::to_owned);
        other.scryfall_data.prices.usd_foil = Some("112.50".to_owned());
        other
    }

    #[test]
    fn live_metrics_price_the_command_zone_at_its_printing_as_edited() {
        let mut forest = typed("Forest", "Basic Land — Forest");
        forest.scryfall_data.prices.usd = Some("0.25".to_owned());
        let fetched = deck(deck_profile("Surfer test"), vec![forest]).entries;
        let edgar = reprint(&card("Edgar Markov"), Some("123.79"));
        let mut zone = Zone {
            commander: Some(edgar.clone()),
            ..Zone::default()
        };
        let usd = |zone: &Zone| {
            zone.live_metrics(&fetched, &fetched)
                .and_then(|m| m.total_price_usd)
                .map(|v| format!("{v:.2}"))
        };
        assert_eq!(usd(&zone), Some("124.04".to_owned()), "as fetched");

        zone.commander = Some(reprint(&edgar, Some("80.00")));
        assert_eq!(
            usd(&zone),
            Some("80.25".to_owned()),
            "a priced printing replaces the old price"
        );

        zone.commander = Some(reprint(&edgar, None));
        assert_eq!(
            usd(&zone),
            Some("0.25".to_owned()),
            "a printing with no USD price (Judge Gift Cards' Edgar) adds nothing, as on load"
        );
    }

    #[test]
    fn live_metrics_are_none_for_an_empty_deck() {
        assert!(Zone::default().live_metrics(&[], &[]).is_none());
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
    fn a_featured_card_opens_its_details_while_it_is_listed_else_its_image() {
        let (listed, hidden) = (Uuid::new_v4(), Uuid::new_v4());
        assert_eq!(featured_open(listed, &[listed]), FeaturedOpen::Details);
        assert_eq!(featured_open(hidden, &[listed]), FeaturedOpen::Image);
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
                from: Board::Maybeboard,
                starred: None,
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
