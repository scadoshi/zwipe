//! Shared card-detail body: the reordered rules panel (head, type/rarity, oracle
//! text, stats, then whole-card keywords + card roles) with elegant MDFC handling
//! and an owned Flip control, plus a bottom action bar.
//!
//! One engine for every surface so they never drift: the expanded [`CardRow`],
//! zite's shared-deck row (via `CardRow`), and zwiper's swipe eyeball dialog all
//! render this. Only the action bar differs - the component owns the always-on
//! actions (Flip for multi-faced cards, Image when art + a handler are present)
//! and appends whatever the host passes via `actions`.
//!
//! [`CardRow`]: crate::CardRow

use dioxus::prelude::*;
use zwipe_core::domain::card::{Card, scryfall_data::ImageSize};

use crate::{CardRoleChips, KeywordChips, OracleText};

/// One card face's rules text, extracted for display. Single-faced cards yield
/// one of these; multi-faced cards (DFCs, split, etc.) yield one per face.
#[derive(Clone)]
struct FaceRules {
    type_line: String,
    mana_cost: String,
    stats: Option<String>,
    oracle: String,
}

/// Format the power/toughness, loyalty, or defense line for a face. Returns the
/// first that applies; `None` if the face has none (e.g. an instant).
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

/// The shared card-detail body + action bar.
///
/// Renders the current face's type line, oracle text, and stats (reordered so the
/// text reads before the numbers), then the whole-card keyword and card-role
/// clusters. Multi-faced cards show one face at a time behind a Flip control whose
/// `face` state this component owns, so a long two-face card never overflows a
/// non-scrolling host. The action bar carries Flip (multi-faced only) and Image
/// (when art + `on_image` are present), then the host's own `actions`.
#[component]
pub fn CardDetails(
    card: Card,
    /// Show the card name in the detail head. Off for hosts that show the name
    /// elsewhere (the eyeball dialog's title).
    #[props(default = true)]
    show_name: bool,
    /// Render the card-role classification (roles + grouped oracle tags) below the
    /// keywords. Off by default so read-only embeds (e.g. the portfolio) opt in.
    #[props(default)]
    show_classification: bool,
    /// Image action: renders a default bar button when the card has art and a
    /// handler is supplied; what "view image" means is the host's business.
    on_image: Option<EventHandler<()>>,
    /// Fires with the new face index whenever the card is flipped, so a host can
    /// mirror the shown side elsewhere (e.g. zite's image preview + hover stack).
    on_face_change: Option<EventHandler<usize>>,
    /// Whether `actions` carries any buttons, so the bar renders to hold them.
    /// (The slot itself can't be introspected.)
    #[props(default)]
    has_actions: bool,
    /// Host buttons appended to the action bar (qty stepper, printing, star,
    /// move-to). Build them with the shared `card-action-btn`/`card-action-row`
    /// styling so they match the defaults.
    #[props(default)]
    actions: Option<Element>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity_name = sd.rarity.to_long_name();
    let keywords = sd.keywords.clone().unwrap_or_default();
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();

    // Card roles (with their grouped oracle tags) render below the keywords, but
    // only when the host opts in via `show_classification`.
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
    // Multi-faced cards (DFCs) show one face at a time with a Flip control, so a
    // long two-face card can't overflow a non-scrolling host.
    let face_count = faces.as_ref().map(|f| f.len()).unwrap_or(0);
    let mut face_idx = use_signal(|| 0usize);
    let cur = face_idx().min(face_count.saturating_sub(1));
    let current_face = faces.as_ref().and_then(|f| f.get(cur)).cloned();
    // Head cost tracks the shown face.
    let cost = current_face
        .as_ref()
        .map(|f| f.mana_cost.clone())
        .unwrap_or_default();

    let show_image = has_image && on_image.is_some();
    let has_defaults = face_count > 1 || show_image;
    let show_bar = has_defaults || has_actions;

    rsx! {
        div { class: "card-row-detail",
            if show_name || !cost.is_empty() {
                div { class: "card-detail-head",
                    if show_name {
                        p { class: "card-detail-name", "{name}" }
                    }
                    if !cost.is_empty() {
                        OracleText { text: cost, class: "card-detail-cost".to_string() }
                    }
                }
            }
            if let Some(face) = current_face {
                div { class: "card-detail-meta",
                    if !face.type_line.is_empty() {
                        span { class: "detail-chip", "{face.type_line}" }
                    }
                    span { class: "detail-chip", "{rarity_name}" }
                }
                if !face.oracle.is_empty() {
                    OracleText { text: face.oracle, class: "card-detail-oracle".to_string() }
                }
                if let Some(stats) = face.stats {
                    div { class: "card-detail-stats",
                        span { class: "detail-chip", "{stats}" }
                    }
                }
            }
            // Analysis cluster below the face: all keywords and (opt-in) all card
            // roles for the whole card, shown once.
            if !keywords.is_empty() {
                KeywordChips { keywords }
            }
            if show_classification {
                CardRoleChips { roles, tags_by_role, other_tags }
            }
        }
        if show_bar {
            hr { class: "card-row-rule card-row-rule-muted" }
            div { class: "card-row-actions",
                if has_defaults {
                    div { class: "card-action-row",
                        if face_count > 1 {
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
