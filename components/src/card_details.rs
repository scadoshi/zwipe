//! Shared card-detail body: the reordered rules panel (head, type/rarity, oracle
//! text, stats, then whole-card keywords + card roles) with elegant MDFC handling
//! and an owned Flip control, plus a bottom action bar.
//!
//! One engine for every surface so they never drift: the expanded [`CardRow`],
//! zite's shared-deck row (via `CardRow`), and zwiper's swipe eyeball dialog all
//! render this. Only the action bar differs: the component owns the always-on
//! actions (Flip for multi-faced cards, Image when art + a handler are present)
//! and appends whatever the host passes via `actions`.
//!
//! [`CardDetailsPanel`] is the same card as a pane of its own: a kit panel
//! titled with the name, a chip row of labeled facts with the mana cost and
//! the prices, the rules text in its body and the analysis in its foot. The
//! swipe surfaces draw it beside the card and in their details drawer.
//!
//! [`CardRow`]: crate::CardRow

use std::collections::BTreeMap;

use dioxus::prelude::*;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::scryfall_data::ImageSize;

use crate::{CardRoleChips, KeywordChips, OracleText, Panel};

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
    /// elsewhere (the eyeball sheet's title).
    #[props(default = true)]
    show_name: bool,
    /// Show the mana cost in the detail head. Off for hosts that show it
    /// elsewhere.
    #[props(default = true)]
    show_cost: bool,
    /// Render the card-role classification (roles + grouped oracle tags) below the
    /// keywords. Off by default so read-only embeds (e.g. the portfolio) opt in.
    #[props(default)]
    show_classification: bool,
    /// Add the printing's release date and artist to the type, rarity and set
    /// chips. Off by default; the app's swipe details sheet turns it on, since
    /// its swipe screens no longer show them under the card.
    #[props(default)]
    show_printing: bool,
    /// Image action: renders a default bar button when the card has art and a
    /// handler is supplied; what "view image" means is the host's business.
    on_image: Option<EventHandler<()>>,
    /// Fires with the new face index whenever the card is flipped, so a host can
    /// mirror the shown side elsewhere (e.g. zite's image preview + hover stack).
    on_face_change: Option<EventHandler<usize>>,
    /// Controlled shown-face index. When `Some`, the host owns the face state so
    /// it can drive Flip from its own chrome (e.g. a sheet footer); when `None`,
    /// this component owns it internally. Pair with `show_flip: false`.
    #[props(default)]
    face: Option<Signal<usize>>,
    /// Render the built-in Flip control in the action bar for multi-faced cards.
    /// Off for hosts that place Flip elsewhere (and pass `face` to drive it).
    #[props(default = true)]
    show_flip: bool,
    /// Whether `actions` carries any buttons, so the bar renders to hold them.
    /// (The slot itself can't be introspected.)
    #[props(default)]
    has_actions: bool,
    /// Host buttons appended to the action bar (qty stepper, printing, star,
    /// move-to). Build them with the shared `card-action-btn`/`card-action-row`
    /// styling so they match the defaults.
    #[props(default)]
    actions: Option<Element>,
    /// Resolve an oracle tag's description, making exposed tags tappable-to-reveal
    /// in the card-role cluster. Forwarded to [`CardRoleChips`]; see there.
    #[props(default)]
    describe_tag: Option<Callback<String, Option<String>>>,
    /// Open the example-cards browse for a tag slug. Forwarded to [`CardRoleChips`].
    #[props(default)]
    on_examples: Option<Callback<String>>,
    /// Host content rendered after the role chips and before the action bar
    /// (the suggested commanders screen's reasons block), with no rule of its
    /// own: a host that wants one draws it. `None` renders nothing.
    #[props(default)]
    footer: Option<Element>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity_name = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let released_at = sd.released_at.to_string();
    let artist = sd.artist.clone().filter(|a| !a.is_empty());
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
        (Vec::new(), BTreeMap::default(), Vec::new())
    };

    let faces = build_rules(&card);
    // Multi-faced cards (DFCs) show one face at a time with a Flip control, so a
    // long two-face card can't overflow a non-scrolling host.
    let face_count = faces.as_ref().map_or(0, Vec::len);
    // Host-controlled face state when `face` is passed, else an internal signal
    // (the hook always runs so the call order stays stable across renders).
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
                    if show_printing {
                        span { class: "detail-chip detail-chip-date", "{released_at}" }
                        if let Some(artist) = artist {
                            span { class: "detail-chip detail-chip-artist", "{artist}" }
                        }
                    }
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
            // Analysis cluster below the face: all keywords and (opt-in) all card
            // roles for the whole card, shown once.
            if !keywords.is_empty() {
                KeywordChips { keywords }
            }
            if show_classification {
                CardRoleChips { roles, tags_by_role, other_tags, describe_tag, on_examples }
            }
            {footer}
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

/// The card as a pane: a kit [`Panel`] titled with the card's name, the
/// shown face's mana cost right after it.
///
/// Under the title a chip row holds the shown face's type line and the
/// card's rarity. The body is the shown face's rules text, a chip row of the
/// set, the prices when `show_prices` and last the face's power and
/// toughness, loyalty or defense, then the keywords and the card roles and
/// their oracle tags when `show_classification`. The panel's foot holds the
/// host's `footer` and the Flip and Image buttons. A multi-faced card
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
    /// Extra classes on the panel.
    #[props(default)]
    class: Option<String>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity = sd.rarity.to_long_name();
    let set_name = sd.set_name.clone();
    let keywords = sd.keywords.clone().unwrap_or_default();
    let prices = if show_prices {
        zwipe_ui::card::info::prices(sd)
    } else {
        Vec::new()
    };
    let has_image = sd.primary_image_url(ImageSize::Large).is_some();
    let (roles, tags_by_role, other_tags) = if show_classification {
        (
            card.card_profile.card_roles.clone(),
            card.card_profile.oracle_tags_by_role.clone(),
            card.card_profile.other_oracle_tags.clone(),
        )
    } else {
        (Vec::new(), BTreeMap::default(), Vec::new())
    };

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
    let has_foot = footer.is_some() || show_flip_btn || show_image;
    let class = match class {
        Some(extra) => format!("card-details-panel {extra}"),
        None => "card-details-panel".to_string(),
    };

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
                    if show_flip_btn || show_image {
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
                        }
                    }
                }
            }),
            div { class: "card-details-keys",
                if let Some(face) = current_face.as_ref().filter(|f| !f.type_line.is_empty()) {
                    span { class: "detail-chip detail-chip-type", "{face.type_line}" }
                }
                span { class: "detail-chip detail-chip-rarity", "{rarity}" }
            }
            if let Some(face) = current_face.filter(|f| !f.oracle.is_empty()) {
                OracleText { text: face.oracle, class: "card-detail-oracle card-details-oracle".to_string() }
            }
            div { class: "card-details-facts",
                span { class: "detail-chip detail-chip-set", "{set_name}" }
                for price in prices {
                    span { key: "{price}", class: "card-row-price card-price-chip", "{price}" }
                }
                if let Some(stats) = stats {
                    span { class: "detail-chip detail-chip-pt", "{stats}" }
                }
            }
            if !keywords.is_empty() {
                div { class: "card-details-analysis",
                    KeywordChips { keywords }
                }
            }
            if show_classification && !(roles.is_empty() && other_tags.is_empty()) {
                div { class: "card-details-analysis",
                    CardRoleChips { roles, tags_by_role, other_tags, describe_tag, on_examples }
                }
            }
        }
    }
}

/// The ghost of a [`CardDetailsPanel`] while its card loads, in the panel's
/// own order: the title bar, the type and rarity chips, the rules text's
/// lines, the set, price and stats chips, a card role's chip, then the foot's
/// rule and the Flip button.
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
                div { class: "card-details-analysis card-details-facts",
                    span { class: "skeleton-bar skeleton-chip skeleton-chip-md" }
                }
            }
            hr { class: "panel-rule" }
            div { class: "panel-actions",
                div { class: "card-details-foot",
                    div { class: "card-action-row",
                        span { class: "skeleton-bar skeleton-chip skeleton-chip-sm" }
                    }
                }
            }
        }
    }
}
