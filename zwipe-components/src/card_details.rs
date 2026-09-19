//! Shared card-detail body: head, type/rarity, oracle text, stats, then the
//! whole-card keywords and card roles, with a Flip control for multi-faced
//! cards and a bottom action bar.
//!
//! The expanded [`CardRow`], zite's shared-deck row, and zwiper's swipe eyeball
//! dialog all render this so they never drift. Only the action bar differs: the
//! component owns Flip (multi-faced cards) and Image (art plus a handler), and
//! appends whatever the host passes via `actions`.
//!
//! [`CardRow`]: crate::CardRow

use dioxus::prelude::*;
use zwipe_core::domain::card::{Card, scryfall_data::ImageSize};

use crate::{CardRoleChips, KeywordChips, OracleText};

/// One face's rules text. Multi-faced cards yield one per face.
#[derive(Clone)]
struct FaceRules {
    type_line: String,
    mana_cost: String,
    stats: Option<String>,
    oracle: String,
}

/// Power/toughness, loyalty, or defense, whichever applies first.
fn stats_line(
    power: &Option<String>,
    toughness: &Option<String>,
    loyalty: &Option<String>,
    defense: &Option<String>,
) -> Option<String> {
    if let (Some(p), Some(t)) = (power, toughness) {
        Some(format!("{p}/{t}"))
    } else if let Some(l) = loyalty {
        Some(format!("Loyalty {l}"))
    } else {
        defense.as_ref().map(|d| format!("Defense {d}"))
    }
}

/// Rules text per face, for printings whose image is text-light (Secret Lair,
/// full-art, foreign-language). Uses per-face text when the top level has none;
/// `None` when there is nothing worth showing.
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
                stats: stats_line(&f.power, &f.toughness, &f.loyalty, &None),
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
        stats: stats_line(&sd.power, &sd.toughness, &sd.loyalty, &sd.defense),
        oracle,
    }])
}

/// Number of rules faces [`CardDetails`] renders for a card: 0 when there is
/// nothing to show, >1 for DFC/split cards. Hosts placing their own Flip
/// control use this to decide whether to render it.
pub fn card_face_count(card: &Card) -> usize {
    build_rules(card).map(|f| f.len()).unwrap_or(0)
}

/// The shared card-detail body and action bar. Multi-faced cards show one face
/// at a time behind Flip so a long two-face card never overflows a
/// non-scrolling host.
#[component]
pub fn CardDetails(
    card: Card,
    /// Show the card name in the head. Off for hosts that show it elsewhere.
    #[props(default = true)]
    show_name: bool,
    /// Show the mana cost in the head. Off for hosts that show it elsewhere.
    #[props(default = true)]
    show_cost: bool,
    /// Render card roles and their oracle tags below the keywords.
    #[props(default)]
    show_classification: bool,
    /// Renders an Image button when the card has art; what "view image" means
    /// is the host's business.
    on_image: Option<EventHandler<()>>,
    /// Fires with the new face index on flip, so a host can mirror the shown
    /// side elsewhere.
    on_face_change: Option<EventHandler<usize>>,
    /// Host-owned face index, for hosts that drive Flip from their own chrome.
    /// Pair with `show_flip: false`.
    #[props(default)]
    face: Option<Signal<usize>>,
    /// Render the built-in Flip control for multi-faced cards.
    #[props(default = true)]
    show_flip: bool,
    /// Whether `actions` carries any buttons; the slot can't be introspected.
    #[props(default)]
    has_actions: bool,
    /// Host buttons appended to the action bar. Use `card-action-btn` and
    /// `card-action-row` so they match the defaults.
    #[props(default)]
    actions: Option<Element>,
    /// Resolve an oracle tag's description. Forwarded to [`CardRoleChips`].
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to [`CardRoleChips`].
    #[props(default)]
    on_examples: Option<Callback<String>>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity_name = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let keywords = sd.keywords.clone().unwrap_or_default();
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    let (roles, tags_by_role, other_tags) = if show_classification {
        (
            card.card_profile.card_roles.clone(),
            card.card_profile.oracle_tags_by_role.clone(),
            card.card_profile.other_oracle_tags.clone(),
        )
    } else {
        (Vec::new(), Default::default(), Vec::new())
    };

    let faces = build_rules(&card);
    let face_count = faces.as_ref().map(|f| f.len()).unwrap_or(0);
    // The internal signal is created even when the host passes `face`, so the
    // hook call order stays stable across renders.
    let internal_face = use_signal(|| 0usize);
    let mut face_idx = face.unwrap_or(internal_face);
    let cur = face_idx().min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    let cost = current_face
        .as_ref()
        .map(|f| f.mana_cost.clone())
        .unwrap_or_default();

    let show_flip_btn = face_count > 1 && show_flip;
    let show_image = has_image && on_image.is_some();
    let has_defaults = show_flip_btn || show_image;
    let show_bar = has_defaults || has_actions;

    rsx! {
        div { class: "card-row-detail",
            if show_name || (show_cost && !cost.is_empty()) {
                div { class: "card-detail-head",
                    if show_name {
                        p { class: "card-detail-name", "{name}" }
                    }
                    if show_cost && !cost.is_empty() {
                        OracleText { text: cost, class: "card-detail-cost".to_string() }
                    }
                }
            }
            if let Some(face) = current_face {
                div { class: "card-detail-meta",
                    if !face.type_line.is_empty() {
                        span { class: "detail-chip detail-chip-type", "{face.type_line}" }
                    }
                    span { class: "detail-chip detail-chip-rarity", "{rarity_name}" }
                    span { class: "detail-chip detail-chip-set", "{set_name}" }
                }
                if !face.oracle.is_empty() {
                    OracleText { text: face.oracle, class: "card-detail-oracle".to_string() }
                }
                if let Some(stats) = face.stats {
                    div { class: "card-detail-stats",
                        span { class: "detail-chip detail-chip-pt", "{stats}" }
                    }
                }
            }
            if !keywords.is_empty() {
                KeywordChips { keywords }
            }
            if show_classification {
                CardRoleChips { roles, tags_by_role, other_tags, describe_tag, on_examples }
            }
        }
        if show_bar {
            hr { class: "card-row-rule card-row-rule-muted" }
            div { class: "card-row-actions",
                if has_defaults {
                    div { class: "card-action-row",
                        if show_flip_btn {
                            button {
                                class: "card-action-btn",
                                onclick: move |evt| {
                                    evt.stop_propagation();
                                    let next = (cur + 1) % face_count;
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
                    }
                }
                {actions}
            }
        }
    }
}
