use crate::inbound::components::alert_dialog::{
    AlertDialogAction, AlertDialogActions, AlertDialogContent, AlertDialogDescription,
    AlertDialogRoot, AlertDialogTitle,
};
use dioxus::prelude::*;
use zwipe_components::{Button, ButtonVariant, CardDetails, OracleText, card_face_count};
use zwipe_core::domain::card::Card;

/// Displays card metadata: prices, set, release date, artist. The card's oracle
/// text and stats live in [`CardDetailsDialog`], opened from the header eyeball.
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

/// Util-bar button (eye icon) that toggles the [`CardDetailsDialog`] for the
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

/// Dialog showing a card's full details (type, oracle text, stats, keywords,
/// card roles), for printings whose image is text-light (Secret Lair, full-art,
/// foreign-language). Opened from the util-bar [`RulesButton`] via the shared
/// `open` signal. Wraps the shared [`CardDetails`] for the body, but drives the
/// Flip control from the dialog's own footer bar (alongside Close and an optional
/// view-only Printings) rather than letting the body render it inline — so the
/// dialog scrolls in one place and its controls stay pinned.
#[component]
pub(crate) fn CardDetailsDialog(
    open: Signal<bool>,
    card: Card,
    /// Opens the printings sheet. When set, a "Printings" action shows in the
    /// dialog footer (mirrors the deck-view expand-row → Printing button).
    #[props(default)]
    on_printings: Option<EventHandler<()>>,
) -> Element {
    let name = card.scryfall_data.name.clone();
    // Face state lives here so Flip can sit in the footer bar; the body reads it
    // via the controlled `face` prop.
    let face_count = card_face_count(&card);
    let mut face = use_signal(|| 0usize);

    // Open at the top: the primitive can land the scroll container partway (focus
    // moves into the dialog on mount). Reset the description's scroll on each open,
    // after the content paints. `card-rules` is the direct child of the
    // `.alert-dialog-description` scroll container.
    use_effect(move || {
        if open() {
            let _ = document::eval(
                "requestAnimationFrame(() => { const el = document.getElementById('card-details-rules'); if (el && el.parentElement) el.parentElement.scrollTop = 0; });",
            );
        }
    });
    // Mana cost for the shown face, pinned in the title so it tracks Flip.
    let cost = {
        let cur = face().min(face_count.saturating_sub(1));
        card.scryfall_data
            .card_faces
            .as_ref()
            .and_then(|faces| faces.get(cur))
            .map(|f| f.mana_cost.clone())
            .filter(|m| !m.is_empty())
            .or_else(|| card.scryfall_data.mana_cost.clone())
            .unwrap_or_default()
    };
    rsx! {
        AlertDialogRoot {
            open: open(),
            on_open_change: move |v| open.set(v),
            AlertDialogContent {
                AlertDialogTitle {
                    div { class: "card-rules-title",
                        span { class: "card-rules-title-name", "{name}" }
                        if !cost.is_empty() {
                            OracleText { text: cost, class: "card-detail-cost".to_string() }
                        }
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogDescription {
                    // The name lives in the title; the shared body shows the
                    // per-face cost, so `show_name` is off. The description is the
                    // sole scroll container (`card-rules` no longer scrolls), and
                    // Flip is hoisted to the footer, so `show_flip` is off.
                    div { id: "card-details-rules", class: "card-rules",
                        CardDetails {
                            card,
                            show_name: false,
                            show_cost: false,
                            show_classification: true,
                            show_flip: false,
                            face: Some(face),
                        }
                    }
                }
                hr { class: "dialog-rule" }
                AlertDialogActions {
                    // Plain button, not AlertDialogAction: the primitive's action
                    // buttons always close the dialog on click, but Flip must swap
                    // faces and leave the dialog open. The class matches the footer
                    // buttons' styling.
                    if face_count > 1 {
                        button {
                            r#type: "button",
                            class: "alert-dialog-action",
                            onclick: move |_| face.set((face() + 1) % face_count),
                            "Flip"
                        }
                    }
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
