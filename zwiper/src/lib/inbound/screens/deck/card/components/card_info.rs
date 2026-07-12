use crate::inbound::components::{
    alert_dialog::{
        AlertDialogAction, AlertDialogActions, AlertDialogContent, AlertDialogDescription,
        AlertDialogRoot, AlertDialogTitle,
    },
    hint_host::HintTopic,
    info_button::InfoButton,
};
use dioxus::prelude::*;
use zwipe_components::{Button, ButtonVariant, CardRoleChips, KeywordChips, OracleText};
use zwipe_core::domain::card::Card;

/// One card face's rules text, extracted for display. Single-faced cards yield
/// one of these; multi-faced cards (DFCs, split, etc.) yield one per face.
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

/// Displays card metadata: prices, set, release date, artist. The card's oracle
/// text and stats live in [`CardRulesDialog`], opened from the header eyeball.
#[component]
pub(crate) fn CardInfoDisplay(card: Card) -> Element {
    let has_prices = card.scryfall_data.prices.usd.is_some()
        || card.scryfall_data.prices.eur.is_some()
        || card.scryfall_data.prices.tix.is_some();

    let price_text = if has_prices {
        let mut display = String::from("Prices:");
        let mut count = 0;
        if let Some(usd) = card.scryfall_data.prices.usd {
            display.push_str(format!(" ${usd}").as_str());
            count += 1;
        }
        if let Some(eur) = card.scryfall_data.prices.eur {
            if count > 0 {
                display.push_str(" |");
            }
            display.push_str(format!(" €{eur}").as_str());
            count += 1;
        }
        if let Some(tix) = card.scryfall_data.prices.tix {
            if count > 0 {
                display.push_str(" |");
            }
            display.push_str(format!(" {tix} TIX").as_str());
        }
        display
    } else {
        "\u{00a0}".to_string()
    };

    let artist_text = card
        .scryfall_data
        .artist
        .filter(|a| !a.is_empty())
        .map(|a| format!("Artist: {a}"))
        .unwrap_or_else(|| "\u{00a0}".to_string());

    rsx! {
        div { class: "card-info",
            span { class: "card-info-name", "{card.scryfall_data.name}" }
            span { "{price_text}" }
            span { "Set: {card.scryfall_data.set_name}" }
            span { "Released: {card.scryfall_data.released_at}" }
            span { "{artist_text}" }
        }
    }
}

/// Util-bar button (eye icon) that toggles the [`CardRulesDialog`] for the
/// active swipe card via the shared `open` signal.
#[component]
pub(crate) fn RulesButton(open: Signal<bool>) -> Element {
    rsx! {
        Button {
            variant: ButtonVariant::Util,
            class: "util-btn-eye",
            onclick: move |_| open.set(!open()),
            svg {
                class: "eye-icon",
                view_box: "0 0 24 24",
                xmlns: "http://www.w3.org/2000/svg",
                path { d: "M1 12s4-8 11-8 11 8 11 8-4 8-11 8-11-8-11-8z" }
                circle { cx: "12", cy: "12", r: "3" }
            }
        }
    }
}

/// Dialog showing a card's oracle text and stats, for printings whose image is
/// text-light (Secret Lair, full-art, foreign-language). Opened from the
/// util-bar [`RulesButton`] via the shared `open` signal. Handles multi-faced cards.
#[component]
pub(crate) fn CardRulesDialog(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" action shows in the
    /// dialog footer (mirrors the deck-view expand-row → Printing button).
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
) -> Element {
    let sd = &card.scryfall_data;
    let name = sd.name.clone();
    let rarity_name = sd.rarity.to_long_name();
    let keywords = sd.keywords.clone().unwrap_or_default();
    // Card classification beside the keywords: roles that drill down to their
    // grouped oracle tags, plus an "Other tags" bucket.
    let roles = card.card_profile.card_roles.clone();
    let tags_by_role = card.card_profile.oracle_tags_by_role.clone();
    let other_tags = card.card_profile.other_oracle_tags.clone();
    let faces = build_rules(&card);
    // Show the primary (first face) mana cost up on the title row, right-aligned
    // next to the name. Extra faces keep their own cost in the body below.
    let title_cost = faces
        .as_ref()
        .and_then(|f| f.first())
        .map(|f| f.mana_cost.clone())
        .unwrap_or_default();
    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogContent {
                AlertDialogTitle {
                    div { class: "card-rules-title",
                        span { class: "card-rules-title-name", "{name}" }
                        if !title_cost.is_empty() {
                            OracleText {
                                text: title_cost,
                                class: "card-detail-cost".to_string(),
                            }
                        }
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogDescription {
                    if let Some(faces) = faces {
                        div { class: "card-rules",
                            for (i, face) in faces.into_iter().enumerate() {
                                div { key: "{i}", class: "card-rules-face",
                                    if i > 0 && !face.mana_cost.is_empty() {
                                        div { class: "card-detail-head",
                                            OracleText {
                                                text: face.mana_cost,
                                                class: "card-detail-cost".to_string(),
                                            }
                                        }
                                    }
                                    div { class: "card-detail-meta",
                                        if !face.type_line.is_empty() {
                                            span { class: "detail-chip", "{face.type_line}" }
                                        }
                                        if i == 0 {
                                            span { class: "detail-chip", "{rarity_name}" }
                                        }
                                    }
                                    if i == 0 && !keywords.is_empty() {
                                        KeywordChips { keywords: keywords.clone() }
                                    }
                                    if i == 0 {
                                        CardRoleChips {
                                            roles: roles.clone(),
                                            tags_by_role: tags_by_role.clone(),
                                            other_tags: other_tags.clone(),
                                            help: rsx! {
                                                InfoButton { topic: HintTopic::CardRoles }
                                            },
                                        }
                                    }
                                    if !face.oracle.is_empty() {
                                        OracleText {
                                            text: face.oracle,
                                            class: "card-detail-oracle".to_string(),
                                        }
                                    }
                                    if let Some(stats) = face.stats {
                                        div { class: "card-detail-stats",
                                            span { class: "detail-chip", "{stats}" }
                                        }
                                    }
                                }
                            }
                        }
                    } else {
                        p { style: "text-align: left; margin: 0;", "No rules text for this card." }
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    AlertDialogAction {
                        on_click: move |_| open.set(false),
                        "Close"
                    }
                    if let Some(handler) = on_printings {
                        AlertDialogAction {
                            on_click: move |_| {
                                open.set(false);
                                handler.call(());
                            },
                            "Printings"
                        }
                    }
                }
            }
        }
    }
}

/// Skeleton placeholder for the printing sheet (carousel + info rows).
#[component]
pub(crate) fn PrintingSheetSkeleton() -> Element {
    rsx! {
        div { class: "skeleton-printing",
            div { class: "skeleton-printing-image" }
            div { class: "skeleton-printing-dots",
                for i in 0..5 {
                    div { key: "{i}", class: "skeleton-printing-dot" }
                }
            }
            div { class: "skeleton-printing-info",
                div { class: "skeleton-bar skeleton-printing-info-price" }
                div { class: "skeleton-bar skeleton-printing-info-set" }
                div { class: "skeleton-bar skeleton-printing-info-released" }
                div { class: "skeleton-bar skeleton-printing-info-artist" }
            }
        }
    }
}

/// Skeleton placeholder for when no card is loaded.
#[component]
pub(crate) fn CardSkeleton(#[props(default = false)] is_loading: bool) -> Element {
    rsx! {
        div { class: "skeleton-card",
            // While a search is in flight the image area stays a plain ghost
            // block (no spinner); "No cards" only when a finished search is
            // genuinely empty.
            div { class: "skeleton-image",
                if !is_loading {
                    "No cards"
                }
            }
            div { class: "skeleton-info",
                div { class: "skeleton-bar skeleton-bar-name" }
                div { class: "skeleton-bar skeleton-bar-price" }
                div { class: "skeleton-bar skeleton-bar-set" }
                div { class: "skeleton-bar skeleton-bar-date" }
                div { class: "skeleton-bar skeleton-bar-artist" }
            }
        }
    }
}
