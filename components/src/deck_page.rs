//! The deck page zite's shared deck and the web's deck view both draw, so
//! the two cannot drift: [`DeckHead`] with the featured cards beside the
//! name and its facts, [`DeckControls`] over the cards, each group as a
//! [`DeckBand`] the page's width with its rows in columns, and the stat
//! panels as [`DeckFold`]s in a grid, all inside [`DeckPage`]. The hosts
//! hold the state, fetch the cards and draw the rows; this module lays them
//! out.

use std::collections::{HashMap, HashSet};

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::colors::Color;
use zwipe_core::domain::card::scryfall_data::{ImageSize, ScryfallData};
use zwipe_core::domain::card::search_card::group_cards::GroupByOption;
use zwipe_core::domain::deck::{Board, Format, PowerLevel};
use zwipe_ui::deck::command_zone::Slot;
use zwipe_ui::deck_cards::Display;

use crate::icons::{DockIcon, FOLD, UNFOLD};
use crate::{Button, ButtonVariant, Chip, FlippableCardImage};

/// The line under an empty card list.
pub const NO_CARDS: &str = "No cards found";

/// What the head says about the deck under its name, as chips.
#[derive(Debug, Clone, PartialEq, Default)]
pub struct DeckFacts {
    /// Every card in the deck, the command zone included.
    pub cards: i64,
    /// The format, if one is set.
    pub format: Option<Format>,
    /// The power level, if one is set.
    pub power_level: Option<PowerLevel>,
    /// The command zone's card names in slot order.
    pub zone_names: Vec<String>,
    /// The deck tags' labels.
    pub tags: Vec<String>,
    /// The other tags' labels.
    pub other_tags: Vec<String>,
    /// The deck's price, formatted, when it has one.
    pub price: Option<String>,
}

/// One card group of the list.
#[derive(Debug, Clone, PartialEq)]
pub struct DeckSection {
    /// The fold key, unique across the list: a color group's pips rather
    /// than its count, which two groups can share.
    pub key: String,
    /// The header: the name with the count, or the bare count for a color
    /// group, which shows pips instead.
    pub title: String,
    /// The pips before the title, for a color group.
    pub pips: Option<Vec<Color>>,
    /// The group's cards.
    pub cards: Vec<Card>,
}

impl DeckSection {
    fn titled(title: String, cards: Vec<Card>) -> Self {
        Self {
            key: title.clone(),
            title,
            pips: None,
            cards,
        }
    }
}

/// The list's sections in order: the command zone's groups, the tokens
/// when shown, the groups of the pass, and the lands last. Counts sum the
/// quantities in `qty`, one for a card it lacks.
#[must_use]
pub fn sections(
    display: &Display,
    format: Option<Format>,
    tokens: Option<&[Card]>,
    qty: &HashMap<Uuid, i32, impl std::hash::BuildHasher>,
) -> Vec<DeckSection> {
    let count = |cards: &[Card]| -> i64 {
        cards
            .iter()
            .map(|c| i64::from(qty.get(&c.scryfall_data.id).copied().unwrap_or(1)))
            .fold(0, i64::saturating_add)
    };
    let mut out = Vec::new();
    if let Some(tokens) = tokens.filter(|t| !t.is_empty()) {
        out.push(DeckSection::titled(
            format!("Tokens ({})", tokens.len()),
            tokens.to_vec(),
        ));
    }
    let has_partner = display.zone.iter().any(|(slot, _)| *slot == Slot::Partner);
    let mut commanders: Vec<Card> = Vec::new();
    for (slot, card) in &display.zone {
        match slot {
            Slot::Commander | Slot::Partner => commanders.push(card.clone()),
            Slot::Background | Slot::SignatureSpell => out.push(DeckSection::titled(
                slot.group_header(format, has_partner).to_owned(),
                vec![card.clone()],
            )),
        }
    }
    if !commanders.is_empty() {
        out.insert(
            usize::from(tokens.is_some_and(|t| !t.is_empty())),
            DeckSection::titled(
                Slot::Commander.group_header(format, has_partner).to_owned(),
                commanders,
            ),
        );
    }
    for group in &display.groups {
        let n = count(&group.cards);
        let title = if group.label.is_empty() {
            format!("({n})")
        } else {
            format!("{} ({n})", group.label)
        };
        out.push(DeckSection {
            key: group.key.clone(),
            title,
            pips: group.pips.clone(),
            cards: group.cards.clone(),
        });
    }
    if !display.lands.is_empty() {
        let n = count(&display.lands);
        out.push(DeckSection::titled(
            format!("Lands ({n})"),
            display.lands.clone(),
        ));
    }
    out
}

/// The page: the head, the controls, the bands, then the stats and the
/// host's foot, one panel gap apart.
#[component]
pub fn DeckPage(
    head: Element,
    #[props(default)] controls: Option<Element>,
    /// Draws [`NO_CARDS`] over the bands.
    #[props(default)]
    empty: bool,
    bands: Element,
    #[props(default)] stats: Option<Element>,
    #[props(default)] foot: Option<Element>,
) -> Element {
    rsx! {
        div { class: "deck-page",
            {head}
            if let Some(controls) = controls {
                div { class: "deck-controls", {controls} }
            }
            if empty {
                p { class: "deck-empty",
                    span { class: "chip-note", {NO_CARDS} }
                }
            }
            div { class: "deck-bands", {bands} }
            if let Some(stats) = stats {
                div { class: "deck-stats", {stats} }
            }
            if let Some(foot) = foot {
                div { class: "deck-foot", {foot} }
            }
        }
    }
}

/// The head: the `name` over the facts as chips, with the host's `actions`
/// at the name's right end, then the featured cards (`children`,
/// [`FeaturedCard`]s) in a centered row under them. Without a name or
/// actions the head opens on the chips, for a host that names the deck and
/// holds its actions elsewhere. `featured_class` joins the featured row's
/// classes, for a host's entrance.
#[component]
pub fn DeckHead(
    #[props(default)] name: Option<String>,
    facts: DeckFacts,
    #[props(default)] actions: Option<Element>,
    #[props(default)] featured_class: Option<String>,
    #[props(default)] has_featured: bool,
    children: Element,
) -> Element {
    let featured_class = match featured_class {
        Some(extra) => format!("deck-head-featured {extra}"),
        None => "deck-head-featured".to_owned(),
    };
    rsx! {
        div { class: "deck-head",
            div { class: "deck-head-text",
                if name.is_some() || actions.is_some() {
                    div { class: "deck-head-title-row",
                        if let Some(name) = name {
                            h1 { class: "deck-head-title", "{name}" }
                        }
                        if let Some(actions) = actions {
                            div { class: "deck-head-actions", {actions} }
                        }
                    }
                }
                div { class: "deck-head-meta",
                    span { class: "stat-chip stat-chip-p5", "{facts.cards} cards" }
                    if let Some(format) = facts.format {
                        span { class: "stat-chip stat-chip-p3", "{format.display_name()}" }
                    }
                    if let Some(power) = facts.power_level {
                        span { class: "stat-chip stat-chip-p5", "{power.display_name()}" }
                    }
                    for name in facts.zone_names.iter() {
                        span { key: "{name}", class: "stat-chip stat-chip-p3", "{name}" }
                    }
                    for tag in facts.tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-muted", "{tag}" }
                    }
                    for tag in facts.other_tags.iter() {
                        span { key: "{tag}", class: "stat-chip stat-chip-p5", "{tag}" }
                    }
                    if let Some(price) = facts.price {
                        span { class: "stat-chip stat-chip-money", "{price}" }
                    }
                }
            }
            if has_featured {
                div { class: "{featured_class}", {children} }
            }
        }
    }
}

/// One featured card: its image over its name chip and role chip. `on_open` fires
/// with the card and the face shown when the image is clicked; `on_load`
/// once the image has its bytes, or at once when the card has no image.
#[component]
pub fn FeaturedCard(
    card: Card,
    role: String,
    #[props(default)] on_open: Option<Callback<(ScryfallData, usize)>>,
    #[props(default)] on_load: Option<Callback<()>>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    let sd = card.scryfall_data;
    let has_image = sd.primary_image_url(ImageSize::Normal).is_some();
    use_effect(use_reactive!(|has_image| {
        if let Some(on_load) = on_load.filter(|_| !has_image) {
            on_load.call(());
        }
    }));
    // The face the flip control shows, so a click opens that one.
    let mut face = use_signal(|| 0usize);
    let sd_open = sd.clone();
    let role_class = if role == "MVP" {
        "stat-chip deck-head-card-role-mvp"
    } else {
        "stat-chip deck-head-card-role-zone"
    };
    rsx! {
        div { class: "deck-head-card",
            if has_image {
                // The flip control stops its click, so flipping never opens.
                div {
                    class: "deck-head-card-image",
                    onclick: move |_| {
                        if let Some(open) = on_open {
                            open.call((sd_open.clone(), face()));
                        }
                    },
                    FlippableCardImage {
                        sd,
                        size: ImageSize::Normal,
                        draggable: false,
                        on_face_change: move |shown: usize| face.set(shown),
                        on_load: move |()| {
                            if let Some(on_load) = on_load {
                                on_load.call(());
                            }
                        },
                    }
                }
            }
            div { class: "deck-head-card-chips",
                span { class: "stat-chip deck-head-card-name", "{name}" }
                if !role.is_empty() {
                    span { class: "{role_class}", "{role}" }
                }
            }
        }
    }
}

/// The rows over the cards: the host's `leading` row, Group by, the host's
/// `boards` row and Show. The host holds every signal; `show_tokens`
/// is `None` for a list with no tokens, `show_zone` is `None` for a deck
/// with no command zone, and `on_change` fires when a row
/// changes what the list shows. With `on_fold_all` the Show row ends in the
/// fold-all button, which shuts every group, or opens every group once
/// `all_folded`.
#[component]
pub fn DeckControls(
    mut group_by: Signal<GroupByOption>,
    #[props(default)] all_folded: bool,
    #[props(default)] on_fold_all: Option<Callback<()>>,
    #[props(default)] show_tokens: Option<Signal<bool>>,
    #[props(default)] show_zone: Option<Signal<bool>>,
    mut show_art: Signal<bool>,
    #[props(default)] on_change: Option<Callback<()>>,
    #[props(default)] leading: Option<Element>,
    #[props(default)] boards: Option<Element>,
) -> Element {
    let changed = move || {
        if let Some(on_change) = on_change {
            on_change.call(());
        }
    };
    rsx! {
        if let Some(leading) = leading {
            {leading}
        }
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Group by" }
            for option in GroupByOption::all() {
                Chip {
                    key: "{option}",
                    selected: group_by() == option,
                    onclick: move |_| {
                        group_by.set(option);
                        changed();
                    },
                    "{option}"
                }
            }
        }
        if let Some(boards) = boards {
            {boards}
        }
        div { class: "chip-row",
            span { class: "stat-chip stat-chip-p1 chip-row-label", "Show" }
            if let Some(mut tokens) = show_tokens {
                Chip {
                    selected: tokens(),
                    onclick: move |_| tokens.toggle(),
                    "Tokens"
                }
            }
            if let Some(mut zone) = show_zone {
                Chip {
                    selected: zone(),
                    onclick: move |_| zone.toggle(),
                    "Command zone"
                }
            }
            Chip {
                selected: show_art(),
                onclick: move |_| show_art.toggle(),
                "Art"
            }
            if let Some(on_fold_all) = on_fold_all {
                Button {
                    variant: ButtonVariant::Action,
                    class: "btn-icon deck-fold-all",
                    onclick: move |_| on_fold_all.call(()),
                    DockIcon { path: if all_folded { UNFOLD } else { FOLD } }

                    span { class: "sr-only", if all_folded { "Open every group" } else { "Shut every group" } }
                }
            }
        }
    }
}

/// A folding panel: a header with the disclosure arrow, the `pips` or the
/// `title`, and the host's `accessory` at its right end, over a body that
/// eases shut. `folded` holds the keys of the shut panels. With `on_drop`
/// the header takes a dragged row.
#[component]
pub fn DeckFold(
    fold_key: String,
    title: String,
    #[props(default)] pips: Option<Vec<Color>>,
    #[props(default)] accessory: Option<Element>,
    mut folded: Signal<HashSet<String>>,
    #[props(default)] on_drop: Option<Callback<()>>,
    children: Element,
) -> Element {
    let shut = folded().contains(&fold_key);
    let mut over = use_signal(|| false);
    let head_class = if over() {
        "deck-fold-head deck-drop-over"
    } else {
        "deck-fold-head"
    };
    rsx! {
        div { class: if shut { "deck-fold folded" } else { "deck-fold" },
            div {
                class: head_class,
                onclick: move |_| {
                    let mut set = folded.write();
                    if !set.remove(&fold_key) {
                        set.insert(fold_key.clone());
                    }
                },
                ondragover: move |evt| {
                    if on_drop.is_some() {
                        evt.prevent_default();
                        over.set(true);
                    }
                },
                ondragleave: move |_| over.set(false),
                ondrop: move |evt| {
                    over.set(false);
                    if let Some(on_drop) = on_drop {
                        evt.prevent_default();
                        on_drop.call(());
                    }
                },
                span { class: "card-row-arrow", "\u{25b8}" }
                if let Some(pips) = pips.as_ref().filter(|p| !p.is_empty()) {
                    span { class: "identity-pips",
                        for color in pips.iter() {
                            i {
                                key: "{color.to_short_name()}",
                                class: "ms ms-{color.to_short_name().to_lowercase()} ms-cost",
                            }
                        }
                    }
                }
                "{title}"
                if let Some(accessory) = accessory {
                    div {
                        class: "deck-fold-accessory",
                        onclick: move |evt| evt.stop_propagation(),
                        {accessory}
                    }
                }
            }
            div { class: "deck-fold-body",
                div { class: "deck-fold-inner", {children} }
            }
        }
    }
}

/// One card group as a band the page's width: a [`DeckFold`] over its rows
/// (`children`). With `drop_to`, a row dragged onto its header moves to
/// that board through `on_drop`.
#[component]
pub fn DeckBand(
    section: DeckSection,
    folded: Signal<HashSet<String>>,
    #[props(default)] drop_to: Option<Board>,
    #[props(default)] on_drop: Option<Callback<Board>>,
    children: Element,
) -> Element {
    let dropped = match (drop_to, on_drop) {
        (Some(board), Some(on_drop)) => Some(Callback::new(move |()| on_drop.call(board))),
        _ => None,
    };
    rsx! {
        DeckFold {
            fold_key: section.key,
            title: section.title,
            pips: section.pips,
            folded,
            on_drop: dropped,
            div { class: "deck-band-rows", {children} }
        }
    }
}

#[cfg(test)]
mod tests {
    use zwipe_ui::deck_cards::Group;

    use super::*;

    /// zwipe-ui's fixture card, renamed, with a fresh printing id.
    fn card(name: &str) -> Card {
        let json = include_str!("../../zwipe-ui/src/test_support/card.json");
        let mut card: Card = serde_json::from_str(json).expect("the fixture card parses");
        card.scryfall_data.id = Uuid::new_v4();
        card.scryfall_data.name = name.to_owned();
        card
    }

    #[test]
    fn sections_run_zone_then_groups_then_lands() {
        let display = Display {
            groups: vec![Group {
                label: "Creatures".to_owned(),
                pips: None,
                cards: vec![card("Bear")],
                key: "Creatures".to_owned(),
            }],
            lands: vec![card("Forest")],
            maybe: vec![],
            side: vec![],
            zone: vec![(Slot::Commander, card("Atraxa"))],
        };
        let titles: Vec<String> = sections(&display, None, None, &HashMap::new())
            .into_iter()
            .map(|s| s.title)
            .collect();
        assert_eq!(titles, ["Commander", "Creatures (1)", "Lands (1)"]);
    }

    #[test]
    fn sections_sum_quantities() {
        let bear = card("Bear");
        let qty = HashMap::from([(bear.scryfall_data.id, 4)]);
        let display = Display {
            groups: vec![Group {
                label: "Creatures".to_owned(),
                pips: None,
                cards: vec![bear],
                key: "Creatures".to_owned(),
            }],
            ..Display::default()
        };
        let out = sections(&display, None, None, &qty);
        assert_eq!(out.first().map(|s| s.title.as_str()), Some("Creatures (4)"));
    }
}
