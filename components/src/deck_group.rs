//! A folding group of cards or decks under a header chip, shared by the decks list and the deck
//! page's stacks.

use std::collections::HashSet;

use dioxus::prelude::*;
use zwipe_core::domain::card::scryfall_data::colors::Color;

use crate::{IdentityPips, Pressable};

/// One group of the list: a header chip that folds it, then its rows, which
/// stay mounted while folded. The chip reads `header (count)`, or the pips
/// and `(count)` for a color group; with no `count` it reads `header` as
/// given. The words sit in a `.card-group-label` span. `trailing` sits at
/// the chip's end (the deck page's stacks put the group's price and cart
/// there); with it, only the arrow and the words fold, in a
/// `.card-group-toggle`, so a press on the trailing buttons never folds.
/// A press folds the group in `collapsed`, or with `on_fold` calls it with
/// the group's key instead, for a host whose folds live elsewhere and come
/// back through `collapsed`.
#[component]
pub fn DeckGroupSection(
    group_key: String,
    header: String,
    pips: Option<Vec<Color>>,
    count: Option<usize>,
    mut collapsed: Signal<HashSet<String>>,
    #[props(default)] trailing: Option<Element>,
    #[props(default)] on_fold: Option<EventHandler<String>>,
    children: Element,
) -> Element {
    let expanded = !collapsed().contains(&group_key);
    let header_class = if expanded {
        "card-group-header group-collapsible expanded"
    } else {
        "card-group-header group-collapsible"
    };
    let fold = move |()| {
        if let Some(on_fold) = on_fold {
            on_fold.call(group_key.clone());
            return;
        }
        let mut set = collapsed.write();
        if !set.remove(&group_key) {
            set.insert(group_key.clone());
        }
    };
    let title = rsx! {
        span { class: "card-row-arrow", "\u{25b8}" }
        if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
            IdentityPips { colors: pips.clone() }
            span { class: "card-group-label",
                if let Some(count) = count {
                    "({count})"
                } else {
                    "{header}"
                }
            }
        } else if let Some(count) = count {
            span { class: "card-group-label", "{header} ({count})" }
        } else {
            span { class: "card-group-label", "{header}" }
        }
    };
    rsx! {
        div {
            class: if expanded { "card-group deck-group row-enter" } else { "card-group deck-group row-enter collapsed" },
            if let Some(trailing) = trailing {
                div { class: header_class,
                    Pressable { class: "card-group-toggle", expanded, onpress: fold, {title} }
                    {trailing}
                }
            } else {
                Pressable { class: header_class, expanded, onpress: fold, {title} }
            }
            div {
                class: if expanded { "collapsible open" } else { "collapsible" },
                inert: (!expanded).then_some(true),
                div { class: "collapsible-inner", {children} }
            }
        }
    }
}
