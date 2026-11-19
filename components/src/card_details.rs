//! A card's details in two shapes that share their pieces: the chip row of
//! the shown face's kind (mana cost, type line, rarity), its rules text, the
//! Keywords and Card roles rows, and the chip row of its facts (set, prices,
//! power and toughness). A multi-faced card shows one face at a time behind
//! Flip.
//!
//! [`CardDetails`] is an expanded [`CardRow`]'s body, on every client: no
//! title (the row's head names the card), the kind with the cost, the rules
//! text, the tag rows, then the facts, with an action bar under a rule.
//! [`CardDetailsPanel`] is the same card as a pane of its own, the swipe
//! surfaces' beside the card and in their details drawer: a kit panel titled
//! with the name and cost, the kind, the rules text, the facts, then the tag
//! rows, with Flip, Image and Printings in its foot.
//!
//! [`CardRow`]: crate::CardRow

use std::collections::HashMap;
use std::hash::BuildHasher;

use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::icons::FLIP;
use crate::meld::{MeldSource, MeldToggle, use_meld};
use crate::printing_face::{PrintingsFace, hand_face_then};
use crate::{
    CardIconButton, CardRoleChips, ImageButton, KeywordChips, MELD, OracleText, Panel, data_action,
};

/// One card face's rules text, extracted for display. Single-faced cards yield
/// one of these; multi-faced cards (DFCs, split, etc.) yield one per face.
#[derive(Clone, PartialEq)]
struct FaceRules {
    type_line: String,
    mana_cost: String,
    stats: Option<String>,
    oracle: String,
}

/// Format the power/toughness, loyalty, or defense line for a face. Returns the
/// first that applies; `None` if the face has none (e.g. an instant).
fn stats_line(
    power: Option<&str>,
    toughness: Option<&str>,
    loyalty: Option<&str>,
    defense: Option<&str>,
) -> Option<String> {
    if let (Some(p), Some(t)) = (power, toughness) {
        Some(format!("{p}/{t}"))
    } else if let Some(l) = loyalty {
        Some(format!("Loyalty {l}"))
    } else {
        defense.map(|d| format!("Defense {d}"))
    }
}

/// Pull the oracle text and stats off a card so it can be shown in-app when the
/// printing's image is text-light (Secret Lair, full-art, foreign-language).
/// Prefers a multi-faced card's per-face text when the top level has none.
/// Returns `None` when there's nothing worth showing.
fn build_rules(card: &Card) -> Option<Vec<FaceRules>> {
    let sd = &card.scryfall_data;

    if sd.oracle_text.is_none()
        && let Some(faces) = sd.card_faces.as_ref()
    {
        let per_face: Vec<FaceRules> = faces
            .iter()
            .map(|f| FaceRules {
                type_line: f.type_line.clone().unwrap_or_default(),
                mana_cost: f.mana_cost.clone(),
                stats: stats_line(
                    f.power.as_deref(),
                    f.toughness.as_deref(),
                    f.loyalty.as_deref(),
                    None,
                ),
                oracle: f.oracle_text.clone().unwrap_or_default(),
            })
            .filter(|f| !f.oracle.is_empty() || !f.type_line.is_empty())
            .collect();
        if !per_face.is_empty() {
            return Some(per_face);
        }
    }

    let oracle = sd.oracle_text.clone().unwrap_or_default();
    let type_line = sd.type_line.clone().unwrap_or_default();
    if oracle.is_empty() && type_line.is_empty() {
        return None;
    }
    Some(vec![FaceRules {
        type_line,
        mana_cost: sd.mana_cost.clone().unwrap_or_default(),
        stats: stats_line(
            sd.power.as_deref(),
            sd.toughness.as_deref(),
            sd.loyalty.as_deref(),
            sd.defense.as_deref(),
        ),
        oracle,
    }])
}

/// Number of rules faces a card renders as in [`CardDetails`]: 1 for a normal
/// card, >1 for DFC/split cards (shown one at a time behind Flip), 0 when there's
/// nothing to show. Hosts that place their own Flip control outside the component
/// (e.g. an eyeball sheet putting it in a footer bar) use this to decide whether
/// to render it.
#[must_use]
pub fn card_face_count(card: &Card) -> usize {
    build_rules(card).map_or(0, |f| f.len())
}

/// The face Flip turns `card` to from `face`, the front after the last, or `None` for a card
/// with one face (a meld half turns through Meld instead).
#[must_use]
pub fn next_card_face(card: &Card, face: usize) -> Option<usize> {
    let count = card_face_count(card);
    (count > 1).then(|| face.saturating_add(1).checked_rem(count).unwrap_or(0))
}

/// Turns `card` to its next face in `faces`, as `v` does on a screen whose surfaces share one
/// face per card, so its image and its details turn together; a meld half, which has one face,
/// turns its Meld from `melds` instead.
pub fn turn_face<S: BuildHasher + 'static>(
    mut faces: Signal<HashMap<Uuid, usize, S>>,
    card: &Card,
    melds: Option<MeldSource>,
) {
    let id = card.scryfall_data.id;
    let face = faces.peek().get(&id).copied().unwrap_or(0);
    match next_card_face(card, face) {
        Some(next) => {
            faces.write().insert(id, next);
        }
        None => MeldToggle::of(&card.scryfall_data, melds).toggle(),
    }
}

/// The face card `id` shows when its host keeps every card's face in `faces`, as a signal for
/// one surface's image or details: it follows the map, so a flip anywhere turns every surface
/// the host gives the map. The surface writes its own flips back through its `on_face_change`.
/// `None` without a map, leaving the surface its own face. Call it from a component keyed by
/// the card. The card's face is read out of the map in a memo, so a flip of another card
/// re-renders nothing here.
pub fn use_shared_face<S: BuildHasher + 'static>(
    faces: Option<Signal<HashMap<Uuid, usize, S>>>,
    id: Uuid,
) -> Option<Signal<usize>> {
    let shared = use_memo(use_reactive((&faces, &id), |(faces, id)| {
        faces.map(|faces| faces.read().get(&id).copied().unwrap_or(0))
    }));
    let mut face = use_signal(|| shared.peek().unwrap_or(0));
    use_effect(move || {
        if let Some(shared) = shared()
            && *face.peek() != shared
        {
            face.set(shared);
        }
    });
    faces.map(|_| face)
}

/// The `on_face_change` that writes card `id`'s flips into its host's `faces`, the other half
/// of [`use_shared_face`].
#[must_use]
pub fn share_face<S: BuildHasher + 'static>(
    faces: Option<Signal<HashMap<Uuid, usize, S>>>,
    id: Uuid,
) -> Option<EventHandler<usize>> {
    faces.map(|mut faces| {
        EventHandler::new(move |face: usize| {
            faces.write().insert(id, face);
        })
    })
}

/// A face's kind as a chip row: its mana cost's pips when `cost` is given,
/// then its type line and the card's rarity. `face` names the face shown as
/// its `data-face`, which the card details' crossfade reads (`drawer_swap`).
#[component]
pub(crate) fn KindChips(
    type_line: String,
    rarity: String,
    #[props(default)] cost: Option<String>,
    #[props(default)] face: Option<String>,
) -> Element {
    rsx! {
        div { class: "card-details-keys", "data-face": face,
            if let Some(cost) = cost.filter(|c| !c.is_empty()) {
                OracleText { text: cost, class: "card-details-cost".to_string() }
            }
            if !type_line.is_empty() {
                span { class: "detail-chip detail-chip-type", "{type_line}" }
            }
            span { class: "detail-chip detail-chip-rarity", "{rarity}" }
        }
    }
}

/// The `data-face` of the face shown: its index, then `m` while a meld half shows its melded
/// card.
fn face_mark(face: usize, melded: bool) -> String {
    if melded {
        format!("{face}m")
    } else {
        face.to_string()
    }
}

/// The card's facts as a chip row: its set, its prices, then the face's
/// power and toughness, loyalty or defense.
#[component]
pub(crate) fn FactsChips(set_name: String, prices: Vec<String>, stats: Option<String>) -> Element {
    rsx! {
        div { class: "card-details-facts",
            span { class: "detail-chip detail-chip-set", "{set_name}" }
            for price in prices {
                span { key: "{price}", class: "card-row-price card-price-chip", "{price}" }
            }
            if let Some(stats) = stats {
                span { class: "detail-chip detail-chip-pt", "{stats}" }
            }
        }
    }
}

/// The card's Keywords row, then its Card roles row with their oracle tags
/// when `show_classification`, each a label chip and the tags as chips.
#[component]
fn TagRows(
    card: Card,
    show_classification: bool,
    describe_tag: Option<Callback<String, Option<String>>>,
    on_examples: Option<Callback<String>>,
) -> Element {
    let keywords = card.scryfall_data.keywords.clone().unwrap_or_default();
    let profile = &card.card_profile;
    let has_roles = show_classification
        && !(profile.card_roles.is_empty() && profile.other_oracle_tags.is_empty());
    rsx! {
        if !keywords.is_empty() {
            div { class: "card-details-analysis",
                KeywordChips { keywords }
            }
        }
        if has_roles {
            div { class: "card-details-analysis",
                CardRoleChips {
                    roles: profile.card_roles.clone(),
                    tags_by_role: profile.oracle_tags_by_role.clone(),
                    other_tags: profile.other_oracle_tags.clone(),
                    describe_tag,
                    on_examples,
                }
            }
        }
    }
}

/// An expanded card row's body and its action bar, in [`CardDetailsPanel`]'s
/// grammar without a title (the row's head names the card).
///
/// The body is a chip row of the shown face's mana cost, type line and the
/// card's rarity, the face's rules text, the Keywords and Card roles rows
/// (roles when `show_classification`), a chip row of the set, the prices and
/// the face's stats, then the host's `footer` in its own section. Under a
/// rule, the action bar holds Flip for a multi-faced card, Meld for a meld
/// half when the host provides a [`MeldSource`](crate::MeldSource), Image when
/// the card has art and `on_image` is given, then the host's `actions`, all in
/// one `card-action-group`, with the host's `action_end` after the group at
/// the end of the line on a phone, and the host's `action_rows` under it. A
/// multi-faced card shows one face at a time behind Flip; a meld half shows
/// its melded card while Meld is on.
#[component]
pub fn CardDetails(
    card: Card,
    /// Show the card roles and their oracle tags. Off by default so a
    /// read-only embed (the portfolio) opts in.
    #[props(default)]
    show_classification: bool,
    /// Image action: an Image button when the card has art; what "view image"
    /// means is the host's business.
    on_image: Option<EventHandler<()>>,
    /// Fires with the new face index whenever Flip turns the card, so a host
    /// can mirror the shown side elsewhere (zite's image overlay).
    on_face_change: Option<EventHandler<usize>>,
    /// The face shown, when the host keeps it: Flip then hands the next face
    /// to `on_face_change` for the host to show. Left out, the details keep
    /// their own.
    #[props(default)]
    face: Option<usize>,
    /// Whether `actions`, `action_end` or `action_rows` carry any buttons, so
    /// the bar renders to hold them. (A slot can't be introspected.)
    #[props(default)]
    has_actions: bool,
    /// Host icon buttons in the bar's group after Flip and Image (Send to,
    /// Star, Flag), built as `card-action-btn card-action-icon`s.
    #[props(default)]
    actions: Option<Element>,
    /// Host controls after the group, at the end of its line on a phone (the
    /// quantity stepper).
    #[props(default)]
    action_end: Option<Element>,
    /// Host rows under the bar's row (the word buttons' block), each a
    /// `card-action-row`.
    #[props(default)]
    action_rows: Option<Element>,
    /// Resolve an oracle tag's description. Forwarded to [`CardRoleChips`].
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to [`CardRoleChips`].
    #[props(default)]
    on_examples: Option<Callback<String>>,
    /// Host content in its own section at the body's foot, after the facts
    /// (the suggested commanders' reasons, a staple's deck share). `None`
    /// renders nothing.
    #[props(default)]
    footer: Option<Element>,
) -> Element {
    // A meld half shows its melded card in its place while Meld is on.
    let meld = use_meld(&card.scryfall_data);
    let show_meld = meld.available();
    let melded = meld.shown().is_some();
    let card = meld.apply(&card);
    let sd = &card.scryfall_data;
    let rarity = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let prices = zwipe_ui::card::info::prices(sd);
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    let faces = build_rules(&card);
    // A multi-faced card shows one face at a time behind Flip, so a long
    // two-face card can't overflow a non-scrolling host.
    let face_count = faces.as_ref().map_or(0, Vec::len);
    let face_idx = use_signal(|| 0usize);
    let cur = face
        .unwrap_or_else(|| *face_idx.read())
        .min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    let stats = current_face.as_ref().and_then(|f| f.stats.clone());

    let show_flip_btn = face_count > 1;
    let show_image = has_image && on_image.is_some();
    let show_bar = show_flip_btn || show_image || show_meld || has_actions;

    rsx! {
        div { class: "card-row-detail card-details-row",
            if let Some(face) = current_face {
                KindChips {
                    type_line: face.type_line,
                    rarity,
                    cost: Some(face.mana_cost),
                    face: face_mark(cur, melded),
                }
                if !face.oracle.is_empty() {
                    OracleText { text: face.oracle, class: "card-detail-oracle card-details-oracle".to_string() }
                }
            }
            TagRows { card, show_classification, describe_tag, on_examples }
            FactsChips { set_name, prices, stats }
            if footer.is_some() {
                div { class: "card-details-host", {footer} }
            }
        }
        if show_bar {
            hr { class: "card-row-rule card-row-rule-muted" }
            div { class: "card-row-actions",
                div { class: "card-action-row card-action-split",
                    span { class: "card-action-group",
                        if show_flip_btn {
                            FlipButton { face_idx, current: cur, face_count, on_face_change }
                        }
                        if show_meld {
                            MeldButton { meld }
                        }
                        if let (true, Some(handler)) = (show_image, on_image) {
                            ImageButton { on_click: handler }
                        }
                        {actions}
                    }
                    {action_end}
                }
                {action_rows}
            }
        }
    }
}

/// The card as a pane: a kit [`Panel`] titled with the card's name, the
/// shown face's mana cost right after it.
///
/// Under the title a chip row holds the shown face's type line and the
/// card's rarity. The body is the shown face's rules text, a chip row of the
/// set, the prices when `show_prices` and last the face's power and
/// toughness, loyalty or defense, then the keywords and the card roles and
/// their oracle tags when `show_classification`, then the host's `body_end`.
/// The panel's foot holds the host's `footer` and the Flip, Image and
/// Printings buttons. A multi-faced card
/// shows one face at a time, as in [`CardDetails`]: `face` lets the host own
/// the shown face (pair it with `show_flip: false` to put Flip in its own
/// chrome).
#[component]
pub fn CardDetailsPanel(
    card: Card,
    /// Show the card's prices after its set.
    #[props(default = true)]
    show_prices: bool,
    /// Show the card roles and their oracle tags in the body, after the
    /// card's facts.
    #[props(default = true)]
    show_classification: bool,
    /// Resolve an oracle tag's description. Forwarded to [`CardRoleChips`].
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to [`CardRoleChips`].
    #[props(default)]
    on_examples: Option<Callback<String>>,
    /// Image action: an Image button in the foot when the card has art.
    #[props(default)]
    on_image: Option<EventHandler<()>>,
    /// Printings action: a Printings button in the foot, opening the host's
    /// printing sheet.
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
    /// Fires with the new face index when Flip turns the card.
    #[props(default)]
    on_face_change: Option<EventHandler<usize>>,
    /// The host's shown-face index, when the host owns it.
    #[props(default)]
    face: Option<Signal<usize>>,
    /// Draw Flip in the foot for a multi-faced card.
    #[props(default = true)]
    show_flip: bool,
    /// Host content in the foot, before its buttons.
    #[props(default)]
    footer: Option<Element>,
    /// Host content at the body's end, after the roles, which scrolls with
    /// the body where `footer` stays in view.
    #[props(default)]
    body_end: Option<Element>,
    /// Extra classes on the panel.
    #[props(default)]
    class: Option<String>,
    /// Leave out the panel's own title band, for a host that shows the
    /// card's name and [`CardDetailsCost`] in its own title bar, such as the
    /// drawer: the body then runs from the panel's top.
    #[props(default)]
    headless: bool,
    /// The meld toggle, when the host owns it, so its own title bar can show
    /// the melded card too (a drawer). Left out, the panel keeps its own.
    #[props(default)]
    meld: Option<MeldToggle>,
    /// A Send to button in the foot, where the client provides [`crate::SendTo`]: on wherever
    /// the panel shows, the drawers and the wide panes alike.
    #[props(default = true)]
    send_to: bool,
) -> Element {
    let send_to = send_to && try_use_context::<crate::SendTo>().is_some();
    let sd = &card.scryfall_data;
    // A meld half can show its melded card's name, type, rules and stats in
    // its place; the set, prices and tags stay the half's own.
    let own_meld = use_meld(sd);
    let meld = meld.unwrap_or(own_meld);
    let melded = meld.shown().is_some().then(|| meld.apply(&card));
    let shown_card = melded.as_ref().unwrap_or(&card);
    let name = shown_card.scryfall_data.name.clone();
    let rarity = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let prices = if show_prices {
        zwipe_ui::card::info::prices(sd)
    } else {
        Vec::new()
    };
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    let faces = build_rules(shown_card);
    let meld_line = melded
        .as_ref()
        .and_then(|_| meld.partner())
        .map(|p| format!("Melds with {p}"));
    let face_count = faces.as_ref().map_or(0, Vec::len);
    let internal_face = use_signal(|| 0usize);
    let face_idx = face.unwrap_or(internal_face);
    let cur = face_idx().min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    // A flip or meld crossfades the panel where it stands beside the card.
    crate::drawer_swap::use_drawer_swap();
    // Printings opens on the face shown here.
    let handed = try_use_context::<PrintingsFace>();
    let on_printings = on_printings.map(|handler| hand_face_then(handed, cur, handler));
    let cost = current_face
        .as_ref()
        .map_or_default(|f| f.mana_cost.clone());
    let stats = current_face.as_ref().and_then(|f| f.stats.clone());
    let show_flip_btn = face_count > 1 && show_flip;
    let show_image = has_image && on_image.is_some();
    let show_meld = meld.available();
    let has_buttons = show_flip_btn || show_image || show_meld || on_printings.is_some() || send_to;
    let has_foot = footer.is_some() || has_buttons;
    let class = match class {
        Some(extra) => format!("card-details-panel {extra}"),
        None => "card-details-panel".to_string(),
    };

    rsx! {
        Panel {
            title: (!headless).then_some(name),
            title_trailing: (!headless && !cost.is_empty()).then(|| rsx! {
                OracleText { text: cost, class: "card-details-cost".to_string() }
            }),
            class,
            actions: has_foot.then(|| rsx! {
                div { class: "card-details-foot",
                    {footer}
                    if has_buttons {
                        div { class: "card-action-row",
                            if show_flip_btn {
                                FlipButton { face_idx, current: cur, face_count, on_face_change }
                            }
                            if show_meld {
                                MeldButton { meld: meld.clone() }
                            }
                            if let (true, Some(handler)) = (show_image, on_image) {
                                ImageButton { on_click: handler }
                            }
                            if let Some(handler) = on_printings {
                                FootButton { label: "Printings", action: data_action::PRINTINGS, on_click: handler }
                            }
                            if send_to {
                                crate::SendToButton { card: card.clone() }
                            }
                        }
                    }
                }
            }),
            KindChips {
                type_line: current_face.as_ref().map_or_default(|f| f.type_line.clone()),
                rarity,
                face: face_mark(cur, melded.is_some()),
            }
            if let Some(line) = meld_line {
                p { class: "card-details-meld", "{line}" }
            }
            if let Some(face) = current_face.filter(|f| !f.oracle.is_empty()) {
                OracleText { text: face.oracle, class: "card-detail-oracle card-details-oracle".to_string() }
            }
            FactsChips { set_name, prices, stats }
            TagRows { card, show_classification, describe_tag, on_examples }
            if let Some(end) = body_end {
                div { class: "card-details-body-end", {end} }
            }
        }
    }
}

/// The mana cost of `card`'s shown face (`face`), drawn as the panel's head
/// draws it, for a host that titles the card in its own bar beside the name.
/// Nothing for a card without a cost.
#[component]
pub fn CardDetailsCost(card: Card, face: Signal<usize>) -> Element {
    let faces = build_rules(&card);
    let face_count = faces.as_ref().map_or(0, Vec::len);
    let cur = face().min(face_count.saturating_sub(1));
    let cost = faces
        .as_ref()
        .and_then(|f| f.get(cur))
        .map_or_default(|f| f.mana_cost.clone());
    rsx! {
        if !cost.is_empty() {
            OracleText { text: cost, class: "card-details-cost".to_string() }
        }
    }
}

/// The ghost rules text's words, in ch: a short card's rules, wrapping as
/// the text does.
const GHOST_ORACLE_WORDS: [u8; 26] = [
    8, 4, 3, 6, 2, 5, 7, 3, 4, 6, 9, 2, 5, 4, 7, 3, 6, 5, 2, 8, 4, 6, 3, 7, 5, 4,
];

/// The Flip button: shows the face after `current` of `face_count` and
/// hands its index to `on_face_change`, keeping the tap off the row.
#[component]
fn FlipButton(
    mut face_idx: Signal<usize>,
    current: usize,
    face_count: usize,
    on_face_change: Option<EventHandler<usize>>,
) -> Element {
    rsx! {
        CardIconButton {
            label: "Flip",
            path: FLIP,
            action: data_action::FLIP,
            on_click: move |()| {
                let next = current.saturating_add(1).checked_rem(face_count).unwrap_or(0);
                face_idx.set(next);
                if let Some(handler) = on_face_change {
                    handler.call(next);
                }
            },
        }
    }
}

/// A meld half's Meld toggle: on, the melded card shows in the half's place.
#[component]
fn MeldButton(meld: MeldToggle) -> Element {
    rsx! {
        CardIconButton {
            label: "Meld",
            path: MELD,
            action: data_action::MELD,
            pressed: Some(meld.on()),
            on_click: move |()| meld.toggle(),
        }
    }
}

/// A foot button labeled `label`, marked `action` for the keys
/// ([`data_action`]), that calls `on_click`, keeping the tap off the row.
#[component]
fn FootButton(label: &'static str, action: &'static str, on_click: EventHandler<()>) -> Element {
    rsx! {
        button {
            class: "card-action-btn",
            "data-action": action,
            onclick: move |evt| {
                evt.stop_propagation();
                on_click.call(());
            },
            "{label}"
        }
    }
}

/// The ghost of a [`CardDetailsPanel`] while its card loads, in the panel's
/// own order and shapes: the title bar, the type and rarity chips, the rules
/// text's lines, the set and price chips, the Keywords and Card roles rows
/// (each a label chip and two tags), then the foot's rule and the Flip and
/// Printings buttons. Every chip and button is empty.
#[component]
pub fn CardDetailsPanelSkeleton() -> Element {
    rsx! {
        div { class: "panel-card card-details-panel card-details-ghost",
            div { class: "panel-head",
                h2 { class: "panel-title",
                    span { class: "skeleton-bar card-details-ghost-title" }
                }
            }
            hr { class: "panel-rule" }
            div { class: "panel-body",
                div { class: "card-details-keys",
                    span { class: "detail-chip detail-chip-type skeleton-tinted",
                        span { "Creature \u{2014} Human" }
                    }
                    span { class: "detail-chip detail-chip-rarity skeleton-tinted",
                        span { "Common" }
                    }
                }
                div { class: "card-detail-oracle card-details-oracle",
                    for (i, chars) in GHOST_ORACLE_WORDS.into_iter().enumerate() {
                        span {
                            key: "{i}",
                            class: "skeleton-bar card-details-ghost-word",
                            style: "width: {chars}ch;",
                        }
                        " "
                    }
                }
                div { class: "card-details-facts",
                    span { class: "detail-chip detail-chip-set skeleton-tinted",
                        span { "Core Set 2015" }
                    }
                    for price in ["$0.10", "\u{20ac}0.10", "0.02 TIX"] {
                        span { key: "{price}", class: "card-row-price card-price-chip skeleton-tinted",
                            span { "{price}" }
                        }
                    }
                }
                for group in ["keyword-section", "card-roles"] {
                    div { key: "{group}", class: "card-details-analysis",
                        div { class: "{group}",
                            div { class: "keyword-chips",
                                span { class: "skeleton-bar skeleton-chip skeleton-tinted stat-chip-p1 card-details-ghost-label" }
                                span { class: "skeleton-bar skeleton-chip card-details-ghost-tag" }
                                span { class: "skeleton-bar skeleton-chip card-details-ghost-tag card-details-ghost-tag-wide" }
                            }
                        }
                    }
                }
            }
            hr { class: "panel-rule" }
            div { class: "panel-actions",
                div { class: "card-details-foot",
                    div { class: "card-action-row",
                        span { class: "skeleton-bar skeleton-chip card-details-ghost-button" }
                        span { class: "skeleton-bar skeleton-chip card-details-ghost-button card-details-ghost-button-wide" }
                    }
                }
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use dioxus::dioxus_core::NoOpMutations;
    use serde_json::{Value, json};

    use super::*;

    /// A real card, as the API serves it.
    const CARD_JSON: &str = include_str!("../../zwipe-ui/src/test_support/card.json");

    /// A transform card: the fixture with Delver of Secrets' two faces in place of its rules.
    fn delver() -> Card {
        let mut card: Value = serde_json::from_str(CARD_JSON).unwrap();
        let sd = card.get_mut("scryfall_data").unwrap();
        sd["oracle_text"] = Value::Null;
        sd["card_faces"] = json!([
            {
                "object": "card_face",
                "name": "Delver of Secrets",
                "mana_cost": "{U}",
                "type_line": "Creature — Human Wizard",
                "oracle_text": "At the beginning of your upkeep, look at the top card of your library.",
                "power": "1",
                "toughness": "1"
            },
            {
                "object": "card_face",
                "name": "Insectile Aberration",
                "mana_cost": "",
                "type_line": "Creature — Human Insect",
                "oracle_text": "Flying",
                "power": "3",
                "toughness": "2"
            }
        ]);
        serde_json::from_value(card).unwrap()
    }

    fn render(app: fn() -> Element) -> String {
        let mut dom = VirtualDom::new(app);
        dom.rebuild_in_place();
        dom.render_immediate(&mut NoOpMutations);
        dioxus_ssr::render(&dom)
    }

    #[test]
    fn the_details_mark_the_face_they_show() {
        fn front() -> Element {
            rsx! { CardDetails { card: delver(), face: Some(0) } }
        }
        fn back() -> Element {
            rsx! { CardDetails { card: delver(), face: Some(1) } }
        }
        let front = render(front);
        let back = render(back);
        assert!(front.contains(r#"data-face="0""#), "{front}");
        assert!(back.contains(r#"data-face="1""#), "{back}");
        assert!(back.contains("Creature — Human Insect"), "{back}");
    }

    #[test]
    fn the_panel_marks_the_face_it_shows() {
        fn back() -> Element {
            let face = use_signal(|| 1);
            rsx! { CardDetailsPanel { card: delver(), face: Some(face) } }
        }
        let back = render(back);
        assert!(back.contains(r#"data-face="1""#), "{back}");
    }

    /// A split card: the fixture with Response // Resurgence's two halves, one image between them.
    fn response() -> Card {
        let mut card: Value = serde_json::from_str(CARD_JSON).unwrap();
        let sd = card.get_mut("scryfall_data").unwrap();
        sd["name"] = json!("Response // Resurgence");
        sd["oracle_text"] = Value::Null;
        sd["card_faces"] = json!([
            {
                "object": "card_face",
                "name": "Response",
                "mana_cost": "{R/W}{R/W}",
                "type_line": "Instant",
                "oracle_text": "Response deals 5 damage to target attacking or blocking creature."
            },
            {
                "object": "card_face",
                "name": "Resurgence",
                "mana_cost": "{3}{R}{W}",
                "type_line": "Sorcery",
                "oracle_text": "Creatures you control gain first strike and vigilance until end of turn."
            }
        ]);
        serde_json::from_value(card).unwrap()
    }

    #[test]
    fn flip_turns_a_split_card_to_its_other_half_and_back() {
        let split = response();
        assert_eq!(next_card_face(&split, 0), Some(1));
        assert_eq!(next_card_face(&split, 1), Some(0));
        assert_eq!(next_card_face(&delver(), 1), Some(0));
        let single: Card = serde_json::from_str(CARD_JSON).unwrap();
        assert_eq!(
            next_card_face(&single, 0),
            None,
            "one face has nothing to turn to"
        );
    }

    #[test]
    fn a_shared_face_shows_that_half_in_the_panel() {
        fn shared() -> Element {
            let card = response();
            let id = card.scryfall_data.id;
            let faces = use_signal(|| HashMap::from([(id, 1)]));
            let face = use_shared_face(Some(faces), id);
            rsx! { CardDetailsPanel { card, face } }
        }
        let html = render(shared);
        assert!(html.contains("Sorcery"), "{html}");
        assert!(html.contains("first strike and vigilance"), "{html}");
        assert!(!html.contains("5 damage"), "{html}");
    }

    #[test]
    fn v_turns_a_card_to_its_next_face_in_the_screen_s_shared_faces() {
        fn app() -> Element {
            let card = delver();
            let id = card.scryfall_data.id;
            let faces = use_signal(HashMap::<Uuid, usize>::new);
            use_hook(move || {
                turn_face(faces, &card, None);
            });
            rsx! { p { "{faces.read().get(&id).copied().unwrap_or(9)}" } }
        }
        let html = render(app);
        assert!(html.contains("<p>1</p>"), "the back: {html}");
    }
}
