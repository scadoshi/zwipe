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

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::{CardRoleChips, KeywordChips, OracleText, Panel};

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
pub fn card_face_count(card: &Card) -> usize {
    build_rules(card).map_or(0, |f| f.len())
}

/// A face's kind as a chip row: its mana cost's pips when `cost` is given,
/// then its type line and the card's rarity.
#[component]
fn KindChips(type_line: String, rarity: String, #[props(default)] cost: Option<String>) -> Element {
    rsx! {
        div { class: "card-details-keys",
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

/// The card's facts as a chip row: its set, its prices, then the face's
/// power and toughness, loyalty or defense.
#[component]
fn FactsChips(set_name: String, prices: Vec<String>, stats: Option<String>) -> Element {
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
/// rule, the action bar holds Flip for a multi-faced card, Image when the
/// card has art and `on_image` is given, then the host's `actions` in the
/// same row, and the host's `action_rows` under it. A multi-faced card shows one face at a
/// time behind Flip.
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
    /// Whether `actions` or `action_rows` carry any buttons, so the bar
    /// renders to hold them. (A slot can't be introspected.)
    #[props(default)]
    has_actions: bool,
    /// Host buttons in the bar's row after Flip and Image (the quantity
    /// stepper, Printing, Star), built as `card-action-btn`s.
    #[props(default)]
    actions: Option<Element>,
    /// Host rows under the bar's row (the board moves), each a
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
    let sd = &card.scryfall_data;
    let rarity = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let prices = zwipe_ui::card::info::prices(sd);
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    let faces = build_rules(&card);
    // A multi-faced card shows one face at a time behind Flip, so a long
    // two-face card can't overflow a non-scrolling host.
    let face_count = faces.as_ref().map_or(0, Vec::len);
    let mut face_idx = use_signal(|| 0usize);
    let cur = face_idx().min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    let stats = current_face.as_ref().and_then(|f| f.stats.clone());

    let show_flip_btn = face_count > 1;
    let show_image = has_image && on_image.is_some();
    let show_bar = show_flip_btn || show_image || has_actions;

    rsx! {
        div { class: "card-row-detail card-details-row",
            if let Some(face) = current_face {
                KindChips { type_line: face.type_line, rarity, cost: Some(face.mana_cost) }
                if !face.oracle.is_empty() {
                    OracleText { text: face.oracle, class: "card-detail-oracle card-details-oracle".to_string() }
                }
            }
            TagRows { card: card.clone(), show_classification, describe_tag, on_examples }
            FactsChips { set_name, prices, stats }
            if footer.is_some() {
                div { class: "card-details-host", {footer} }
            }
        }
        if show_bar {
            hr { class: "card-row-rule card-row-rule-muted" }
            div { class: "card-row-actions",
                div { class: "card-action-row",
                    if show_flip_btn {
                        button {
                            class: "card-action-btn",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                let next = cur.saturating_add(1).checked_rem(face_count).unwrap_or(0);
                                face_idx.set(next);
                                if let Some(handler) = on_face_change {
                                    handler.call(next);
                                }
                            },
                            "Flip"
                        }
                    }
                    if let (true, Some(handler)) = (show_image, on_image) {
                        button {
                            class: "card-action-btn",
                            onclick: move |evt| {
                                evt.stop_propagation();
                                handler.call(());
                            },
                            "Image"
                        }
                    }
                    {actions}
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
    /// Show the card roles and their oracle tags in the foot.
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
    /// Host content in the foot, after the roles.
    #[props(default)]
    footer: Option<Element>,
    /// Host content at the body's end, after the roles, which scrolls with
    /// the body where `footer` stays in view.
    #[props(default)]
    body_end: Option<Element>,
    /// Extra classes on the panel.
    #[props(default)]
    class: Option<String>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let prices = if show_prices {
        zwipe_ui::card::info::prices(sd)
    } else {
        Vec::new()
    };
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    let faces = build_rules(&card);
    let face_count = faces.as_ref().map_or(0, Vec::len);
    let internal_face = use_signal(|| 0usize);
    let mut face_idx = face.unwrap_or(internal_face);
    let cur = face_idx().min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    let cost = current_face
        .as_ref()
        .map(|f| f.mana_cost.clone())
        .unwrap_or_default();
    let stats = current_face.as_ref().and_then(|f| f.stats.clone());
    let show_flip_btn = face_count > 1 && show_flip;
    let show_image = has_image && on_image.is_some();
    let has_foot = footer.is_some() || show_flip_btn || show_image || on_printings.is_some();
    let class = match class {
        Some(extra) => format!("card-details-panel {extra}"),
        None => "card-details-panel".to_string(),
    };

    // A tall pane scrolls its body between its head and foot; scrolling
    // down hides the head and scrolling up brings it back.
    use_hook(|| {
        document::eval(HEAD_HIDE_JS);
    });

    rsx! {
        Panel {
            title: name,
            title_trailing: (!cost.is_empty()).then(|| rsx! {
                OracleText { text: cost, class: "card-details-cost".to_string() }
            }),
            class,
            actions: has_foot.then(|| rsx! {
                div { class: "card-details-foot",
                    {footer}
                    if show_flip_btn || show_image || on_printings.is_some() {
                        div { class: "card-action-row",
                            if show_flip_btn {
                                button {
                                    class: "card-action-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        let next = cur.saturating_add(1).checked_rem(face_count).unwrap_or(0);
                                        face_idx.set(next);
                                        if let Some(handler) = on_face_change {
                                            handler.call(next);
                                        }
                                    },
                                    "Flip"
                                }
                            }
                            if let (true, Some(handler)) = (show_image, on_image) {
                                button {
                                    class: "card-action-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        handler.call(());
                                    },
                                    "Image"
                                }
                            }
                            if let Some(handler) = on_printings {
                                button {
                                    class: "card-action-btn",
                                    onclick: move |evt| {
                                        evt.stop_propagation();
                                        handler.call(());
                                    },
                                    "Printings"
                                }
                            }
                        }
                    }
                }
            }),
            KindChips {
                type_line: current_face.as_ref().map(|f| f.type_line.clone()).unwrap_or_default(),
                rarity,
            }
            if let Some(face) = current_face.filter(|f| !f.oracle.is_empty()) {
                OracleText { text: face.oracle, class: "card-detail-oracle card-details-oracle".to_string() }
            }
            FactsChips { set_name, prices, stats }
            TagRows { card: card.clone(), show_classification, describe_tag, on_examples }
            if let Some(end) = body_end {
                div { class: "card-details-body-end", {end} }
            }
        }
    }
}

/// The pane's head follows its body's scroll: scrolling down past the first
/// few pixels marks the panel `card-details-head-hidden`, which folds the
/// title away, and scrolling up or reaching the top brings it back. One
/// listener, in the capture phase since scroll events do not bubble, serves
/// every pane; it guards itself so a second mount does nothing. The head
/// folds only when the body overflows by more than the fold frees, so the
/// fold never makes the pane fit and snap back. The bounce past an edge is
/// not scrolling, and a toggle waits out the fold's own motion so the
/// body's change in height never reads as a scroll. With reduced motion the
/// head stays.
const HEAD_HIDE_JS: &str = r"
if (!window.__zwipeDetailsHead) {
    window.__zwipeDetailsHead = true;
    const still = window.matchMedia('(prefers-reduced-motion: reduce)');
    const last = new WeakMap();
    const settled = new WeakMap();
    document.addEventListener('scroll', (event) => {
        const body = event.target;
        if (!(body instanceof Element) || !body.matches('.card-details-panel > .panel-body')) return;
        const panel = body.parentElement;
        const top = Math.max(0, body.scrollTop);
        const bottom = body.scrollHeight - body.clientHeight;
        const previous = last.has(body) ? last.get(body) : top;
        last.set(body, top);
        if (still.matches) return;
        if (performance.now() < (settled.get(panel) || 0)) return;
        if (top <= 0) {
            panel.classList.remove('card-details-head-hidden');
            return;
        }
        if (top > bottom || previous > bottom) return;
        const hidden = panel.classList.contains('card-details-head-hidden');
        const head = panel.querySelector(':scope > .panel-head');
        const frees = head ? head.offsetHeight + 48 : 0;
        const toggle = (hide) => {
            panel.classList.toggle('card-details-head-hidden', hide);
            settled.set(panel, performance.now() + 250);
        };
        if (!hidden && top > previous + 2 && top > 8 && bottom > frees) toggle(true);
        else if (hidden && top < previous - 2 && top < bottom - 2) toggle(false);
    }, { capture: true, passive: true });
}
";

/// The ghost of a [`CardDetailsPanel`] while its card loads, in the panel's
/// own order and shapes: the title bar, the type and rarity chips, the rules
/// text's lines, the set, price and stats chips, the Keywords and Card roles
/// rows (each a label chip and two tags), then the foot's rule and the Flip
/// and Printings buttons.
#[component]
pub fn CardDetailsPanelSkeleton() -> Element {
    rsx! {
        div { class: "panel-card card-details-panel card-details-ghost",
            div { class: "panel-head",
                div { class: "skeleton-bar card-details-ghost-title" }
            }
            hr { class: "panel-rule" }
            div { class: "panel-body",
                div { class: "card-details-keys",
                    span { class: "skeleton-bar skeleton-chip skeleton-chip-lg" }
                    span { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                }
                for i in 0..4 {
                    div { key: "{i}", class: "skeleton-bar card-details-ghost-line" }
                }
                div { class: "card-details-facts",
                    for i in 0..4 {
                        span { key: "{i}", class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    }
                }
                for group in ["keyword-section", "card-roles"] {
                    div { key: "{group}", class: "card-details-analysis",
                        div { class: "{group}",
                            div { class: "keyword-chips",
                                span { class: "skeleton-bar card-details-ghost-label" }
                                span { class: "skeleton-bar card-details-ghost-tag" }
                                span { class: "skeleton-bar card-details-ghost-tag card-details-ghost-tag-wide" }
                            }
                        }
                    }
                }
            }
            hr { class: "panel-rule" }
            div { class: "panel-actions",
                div { class: "card-details-foot",
                    div { class: "card-action-row",
                        span { class: "skeleton-bar card-details-ghost-button" }
                        span { class: "skeleton-bar card-details-ghost-button card-details-ghost-button-wide" }
                    }
                }
            }
        }
    }
}
