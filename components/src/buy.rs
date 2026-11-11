//! Buying: the cart button every buy spot shows, and the one store dialog it opens, with
//! zwipe-ui's [`buy`] links to `TCGplayer` and `Card Kingdom`.

use dioxus::prelude::*;
use zwipe_ui::buy::{self, BuyLink};

use crate::icons::DockIcon;
use crate::{AlertDialog, Button, ButtonVariant, DialogAction};

/// The cart icon: a basket on two wheels.
pub const CART: &str =
    "M3 4h2l2.4 10.2a1 1 0 0 0 1 .8h8.8a1 1 0 0 0 1-.8L20 8H6.2M9 20h.01M17 20h.01";

/// The cart button every buy spot shows, icon only, with its name for a screen reader. It keeps
/// its click from the row or header under it.
#[component]
pub fn CartButton(onclick: EventHandler<MouseEvent>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Action,
            class: "btn-icon cart-button",
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                onclick.call(evt);
            },
            DockIcon { path: CART }
            span { class: "sr-only", {buy::CART_LABEL} }
        }
    }
}

/// The store dialog's actions: Close, then a link to each store in `links`.
#[must_use]
pub fn store_actions(links: Vec<BuyLink>) -> Vec<DialogAction> {
    std::iter::once(DialogAction::Dismiss(buy::CLOSE.to_owned()))
        .chain(links.into_iter().map(|link| DialogAction::Link {
            label: link.label.to_owned(),
            href: link.url,
        }))
        .collect()
}

/// The one store dialog, open while `open`: `title` and `body`, Close, and each store's link in
/// `links`, which send the cards there.
#[component]
pub fn StoreDialog(
    open: Signal<bool>,
    title: String,
    body: String,
    links: Vec<BuyLink>,
) -> Element {
    rsx! {
        AlertDialog { open, title, actions: store_actions(links), "{body}" }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_dialog_closes_first_then_offers_each_store() {
        let links = buy::cards_links([(1, "Sol Ring")]);
        let actions = store_actions(links.clone());
        let mut expected = vec![DialogAction::Dismiss(buy::CLOSE.to_owned())];
        expected.extend(links.into_iter().map(|link| DialogAction::Link {
            label: link.label.to_owned(),
            href: link.url,
        }));
        assert_eq!(actions, expected);
    }
}
