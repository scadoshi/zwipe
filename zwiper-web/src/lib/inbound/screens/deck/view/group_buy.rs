//! The store dialog a card group's cart opens, a List band's or a stack's: the group's cards
//! with their copies, sent to `TCGplayer` or `Card Kingdom` through zwipe-ui's [`buy`] links.

use components::StoreDialog;
use dioxus::prelude::*;
use zwipe_ui::buy::{self, BuyLink};

use crate::inbound::screens::deck::cards::editor::{RowInfo, RowKind};

/// The cards a group sends to a store, each with its copies: every card but a token.
pub(crate) fn group_cards(rows: &[RowInfo]) -> Vec<(i32, &str)> {
    rows.iter()
        .filter(|row| row.kind != RowKind::Token)
        .map(|row| (row.qty, row.card.scryfall_data.name.as_str()))
        .collect()
}

/// The groups' store dialog: whether it shows, and the group it was opened for, its title and
/// store links.
#[derive(Debug, Clone, Copy, PartialEq)]
pub(crate) struct GroupBuy {
    /// Whether the dialog shows.
    pub(crate) open: Signal<bool>,
    /// The dialog's title and links, for the group whose cart was clicked.
    pub(crate) group: Signal<Option<(String, Vec<BuyLink>)>>,
}

impl GroupBuy {
    /// Opens the dialog on the group whose header reads `header`, holding `rows`.
    pub(crate) fn offer(self, header: &str, rows: &[RowInfo]) {
        let Self {
            mut open,
            mut group,
        } = self;
        group.set(Some((
            buy::group_title(header),
            buy::cards_links(group_cards(rows)),
        )));
        open.set(true);
    }
}

/// The groups' store dialog, closed until a group's cart opens it.
pub(crate) fn use_group_buy() -> GroupBuy {
    GroupBuy {
        open: use_signal(|| false),
        group: use_signal(|| None),
    }
}

/// The store dialog for the group whose cart was clicked; nothing before any click.
pub(crate) fn group_buy_dialog(group_buy: GroupBuy) -> Element {
    let Some((title, links)) = (group_buy.group)() else {
        return rsx! {};
    };
    rsx! {
        StoreDialog {
            open: group_buy.open,
            title,
            body: buy::GROUP_BODY.to_owned(),
            links,
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_core::domain::deck::Board;
    use zwipe_ui::deck::form::Slot;

    use super::*;

    fn row(name: &str, qty: i32, kind: RowKind) -> RowInfo {
        let mut card: zwipe_core::domain::card::Card = serde_json::from_str(include_str!(
            "../../../../../../../zwipe-ui/src/test_support/card.json"
        ))
        .unwrap();
        card.scryfall_data.name = name.to_owned();
        RowInfo { card, qty, kind }
    }

    #[test]
    fn a_group_sends_its_cards_with_their_copies_and_leaves_tokens_out() {
        let rows = [
            row("Sol Ring", 1, RowKind::Main { mvp: false }),
            row("Forest", 12, RowKind::Board(Board::Deck)),
            row("Krenko, Mob Boss", 1, RowKind::Zone(Slot::Commander)),
            row("Goblin", 1, RowKind::Token),
        ];
        assert_eq!(
            group_cards(&rows),
            [(1, "Sol Ring"), (12, "Forest"), (1, "Krenko, Mob Boss")]
        );
        assert!(
            group_cards(&[row("Goblin", 1, RowKind::Token)]).is_empty(),
            "a group of tokens has nothing to buy"
        );
    }
}
