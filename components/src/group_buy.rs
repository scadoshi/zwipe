//! The store dialog a deck page group's cart opens, a List band's or a stack's: the group's cards
//! with their copies, sent to `TCGplayer` or `Card Kingdom` through zwipe-ui's [`buy`] links.

use dioxus::prelude::*;
use zwipe_ui::buy::{self, BuyLink};
use zwipe_ui::deck_cards::keys::RowInfo;

use crate::StoreDialog;

/// The groups' store dialog: whether it shows, and the group it was opened for, its title and
/// store links.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct GroupBuy {
    /// Whether the dialog shows.
    pub open: Signal<bool>,
    /// The dialog's title and links, for the group whose cart was clicked.
    pub group: Signal<Option<(String, Vec<BuyLink>)>>,
}

impl GroupBuy {
    /// Opens the dialog on the group whose header reads `header`, holding `rows`.
    pub fn offer(self, header: &str, rows: &[RowInfo]) {
        let Self {
            mut open,
            mut group,
        } = self;
        group.set(Some((
            buy::group_title(header),
            buy::cards_links(buy::group_cards(rows)),
        )));
        open.set(true);
    }
}

/// The groups' store dialog, closed until a group's cart opens it.
#[must_use]
pub fn use_group_buy() -> GroupBuy {
    GroupBuy {
        open: use_signal(|| false),
        group: use_signal(|| None),
    }
}

/// The store dialog for the group whose cart was clicked; nothing before any click.
#[component]
pub fn GroupBuyDialog(group_buy: GroupBuy) -> Element {
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
