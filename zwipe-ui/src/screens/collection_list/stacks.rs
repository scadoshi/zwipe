//! Collection's Stacks view: the rows listed so far as the deck page's stacks, one stack per card
//! type with lands last, each printing once.
//!
//! A printing can hold several rows (finish, condition, language), and a stack shows a printing
//! once, so its card carries every row that holds it, in list order, and their copies summed. A
//! client opens a card's rows in its details, each with its own stepper.

use std::collections::HashMap;

use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::group_cards::{GroupByOption, GroupCards};

use super::Row;
use crate::deck_cards::stacks::CardsView;

/// The view Collection opens on before the user picks one.
pub const DEFAULT_VIEW: CardsView = CardsView::Stacks;

/// The lands' stack, last as on the deck page.
const LANDS: &str = "Lands";

/// One printing in the stacks, with every row that holds it.
#[derive(Debug, Clone, PartialEq)]
pub struct StackCard {
    /// The printing.
    pub card: Card,
    /// The copies across its rows.
    pub copies: i32,
    /// Its rows, in list order.
    pub rows: Vec<Row>,
}

/// One stack: a card type's printings.
#[derive(Debug, Clone, PartialEq)]
pub struct Stack {
    /// The fold key: the type's name.
    pub key: String,
    /// The header: the type's name with its copies, as the deck page counts.
    pub title: String,
    /// The printings, in list order.
    pub cards: Vec<StackCard>,
}

/// The stacks for `rows`: each printing once, grouped by card type in the deck page's order, then
/// the lands. Empty types are left out.
#[must_use]
pub fn stacks(rows: &[Row]) -> Vec<Stack> {
    let mut merged: Vec<StackCard> = Vec::new();
    let mut at: HashMap<Uuid, usize> = HashMap::new();
    for row in rows {
        let id = row.card.card.scryfall_data.id;
        if let Some(card) = at.get(&id).and_then(|&i| merged.get_mut(i)) {
            card.copies = card.copies.saturating_add(row.copies);
            card.rows.push(row.clone());
        } else {
            at.insert(id, merged.len());
            merged.push(StackCard {
                card: row.card.card.clone(),
                copies: row.copies,
                rows: vec![row.clone()],
            });
        }
    }
    let (lands, rest): (Vec<StackCard>, Vec<StackCard>) = merged
        .into_iter()
        .partition(|c| c.card.scryfall_data.is_land());
    let order: Vec<Card> = rest.iter().map(|c| c.card.clone()).collect();
    let mut by_id: HashMap<Uuid, StackCard> = rest
        .into_iter()
        .map(|c| (c.card.scryfall_data.id, c))
        .collect();
    let mut out: Vec<Stack> = order
        .group_by(GroupByOption::CardType)
        .into_iter()
        .map(|group| {
            let key = group.key();
            let cards: Vec<StackCard> = group
                .cards
                .iter()
                .filter_map(|card| by_id.remove(&card.scryfall_data.id))
                .collect();
            stack(key, cards)
        })
        .filter(|s| !s.cards.is_empty())
        .collect();
    if !lands.is_empty() {
        out.push(stack(LANDS.to_owned(), lands));
    }
    out
}

/// The stack `key` over `cards`, its title counting their copies.
fn stack(key: String, cards: Vec<StackCard>) -> Stack {
    let copies = cards
        .iter()
        .map(|c| i64::from(c.copies))
        .fold(0, i64::saturating_add);
    Stack {
        title: format!("{key} ({copies})"),
        key,
        cards,
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::collection::{
        CardCondition, CardLanguage, CollectionQuantity, CollectionTags, Finish,
    };
    use zwipe_core::http::contracts::collection::CollectionCard;

    use super::*;
    use crate::screens::collection_list::RowKey;
    use crate::test_support::card;

    fn typed(name: &str, type_line: &str) -> Card {
        let mut card = card(name);
        card.scryfall_data.type_line = Some(type_line.to_owned());
        card
    }

    fn row(card: &Card, finish: Finish, copies: i32) -> Row {
        let card = CollectionCard {
            card: card.clone(),
            finish,
            condition: Some(CardCondition::NearMint),
            language: Some(CardLanguage::En),
            quantity: CollectionQuantity::new(copies).unwrap(),
            tags: CollectionTags::default(),
            added_at: None,
        };
        let key = RowKey::of(&card);
        Row {
            key,
            card,
            copies,
            editable: key.editable(),
        }
    }

    fn titles(stacks: &[Stack]) -> Vec<&str> {
        stacks.iter().map(|s| s.title.as_str()).collect()
    }

    #[test]
    fn a_printing_in_two_finishes_is_one_card_with_both_rows() {
        let ring = typed("Sol Ring", "Artifact");
        let stacks = stacks(&[row(&ring, Finish::Nonfoil, 2), row(&ring, Finish::Foil, 1)]);
        let [artifacts] = stacks.as_slice() else {
            panic!("{stacks:?}");
        };
        let [sol] = artifacts.cards.as_slice() else {
            panic!("{artifacts:?}");
        };
        assert_eq!(sol.copies, 3, "copies summed across the rows");
        let finishes: Vec<Finish> = sol.rows.iter().map(|r| r.key.finish).collect();
        assert_eq!(
            finishes,
            [Finish::Nonfoil, Finish::Foil],
            "both rows, in list order"
        );
        assert_eq!(artifacts.title, "Artifacts (3)");
    }

    #[test]
    fn stacks_follow_the_deck_page_s_type_order_with_lands_last() {
        let rows = [
            row(&typed("Command Tower", "Land"), Finish::Nonfoil, 1),
            row(&typed("Counterspell", "Instant"), Finish::Nonfoil, 2),
            row(
                &typed("Krenko, Mob Boss", "Legendary Creature"),
                Finish::Nonfoil,
                1,
            ),
            row(&typed("Arcane Signet", "Artifact"), Finish::Nonfoil, 1),
        ];
        assert_eq!(
            titles(&stacks(&rows)),
            [
                "Creatures (1)",
                "Artifacts (1)",
                "Instants (2)",
                "Lands (1)"
            ]
        );
    }

    #[test]
    fn a_stack_keeps_the_list_s_order_and_its_key_is_the_type() {
        let rows = [
            row(&typed("Beast Within", "Instant"), Finish::Nonfoil, 1),
            row(&typed("Brainstorm", "Instant"), Finish::Nonfoil, 1),
        ];
        let stacks = stacks(&rows);
        let names: Vec<&str> = stacks[0]
            .cards
            .iter()
            .map(|c| c.card.scryfall_data.name.as_str())
            .collect();
        assert_eq!(names, ["Beast Within", "Brainstorm"]);
        assert_eq!(stacks[0].key, "Instants");
    }

    #[test]
    fn no_rows_make_no_stacks() {
        assert_eq!(stacks(&[]), Vec::<Stack>::new());
    }

    #[test]
    fn collection_opens_on_stacks() {
        assert_eq!(DEFAULT_VIEW, CardsView::Stacks);
    }
}
