//! Full-screen format picker.
//!
//! Mirrors the tag picker's layout and behavior, but single-select: tapping a
//! format selects it (replacing any prior pick) and reveals its details in the
//! bar pinned at the top; tapping the selected format again clears it. Selecting
//! and clearing are reported through callbacks so the form can run its
//! command-zone cascade (clearing commander and signature spell on a change).

use crate::inbound::components::{
    hint_dialog::{HintBullet, HintBullets, HintColored, HintDialog},
    screen_header::ScreenHeader,
};
use dioxus::prelude::*;
use zwipe_components::{ActionBar, Button, ButtonVariant};
use zwipe_core::domain::deck::format::Format;

/// Deck size summary, e.g. "100 cards, singleton" or "60+ cards, up to 4 copies".
fn size_text(fmt: Format) -> String {
    let copies = if fmt.copy_max() == 1 {
        "singleton"
    } else {
        "up to 4 copies"
    };
    let count = match (fmt.min_cards(), fmt.max_cards()) {
        (Some(a), Some(b)) if a == b => format!("{a} cards"),
        (Some(a), Some(b)) => format!("{a} to {b} cards"),
        (Some(a), None) => format!("{a}+ cards"),
        _ => "Any size".to_string(),
    };
    format!("{count}, {copies}")
}

/// What the command zone holds for this format.
fn command_zone_text(fmt: Format) -> &'static str {
    if fmt.has_signature_spell() {
        "Planeswalker commander + signature spell"
    } else if fmt == Format::PauperCommander {
        "1 uncommon creature commander"
    } else if fmt.has_commander() {
        if fmt.supports_partner() {
            "1 legendary commander (partners & backgrounds OK)"
        } else {
            "1 legendary commander"
        }
    } else {
        "None"
    }
}

/// In-place format picker. Toggled by `open`. `on_select` fires with the chosen
/// format, `on_clear` cancels the current pick, `on_close` returns to the form.
#[component]
pub(crate) fn FormatSelect(
    open: Signal<bool>,
    selected_format: Signal<Option<Format>>,
    on_select: EventHandler<Format>,
    on_clear: EventHandler<()>,
    on_close: EventHandler<()>,
) -> Element {
    let mut query = use_signal(String::new);
    let mut focused = use_signal(|| Option::<Format>::None);
    let hint_open = use_signal(|| false);

    let screen_class = if open() {
        "screen swipe-select-screen show"
    } else {
        "screen swipe-select-screen"
    };

    let results: Vec<Format> = if open() {
        let q = query().to_lowercase();
        Format::all()
            .iter()
            .copied()
            .filter(|f| q.is_empty() || f.display_name().to_lowercase().contains(&q))
            .collect()
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "{screen_class}",
            if open() {
                ScreenHeader { title: "Format", hint: hint_open }

                div { class: "screen-content content-enter tag-screen",
                    div { class: "tag-controls",
                        div { class: "tag-controls-head",
                            label { class: "tag-search-label", "Search" }
                        }

                        input { class: "input",
                            id: "format-search",
                            r#type: "text",
                            placeholder: "Search formats",
                            value: "{query()}",
                            autocapitalize: "none",
                            autocorrect: "off",
                            spellcheck: "false",
                            oninput: move |event| query.set(event.value()),
                        }

                        div { class: "tag-def-bar",
                            if let Some(fmt) = focused() {
                                div { class: "tag-def-name", "{fmt.display_name()}" }
                                ul { class: "tag-def-list",
                                    li { "Pool: {fmt.card_pool()}" }
                                    li { "Cards: {size_text(fmt)}" }
                                    li { "Life: {fmt.starting_life()}" }
                                    if fmt.has_commander() || fmt.has_signature_spell() {
                                        li { "Command zone: {command_zone_text(fmt)}" }
                                    }
                                    if let Some(dmg) = fmt.commander_damage() {
                                        li { "Commander damage: {dmg}" }
                                    }
                                }
                            } else {
                                div { class: "tag-def-name", "Hint" }
                                div { class: "tag-def-text", "Tap a format to see its details here." }
                            }
                        }
                    }

                    div { class: "tag-grid",
                        if results.is_empty() {
                            div { class: "chip-unselected", "No results" }
                        } else {
                            for fmt in results {
                                div {
                                    key: "{fmt.display_name()}",
                                    class: if selected_format() == Some(fmt) { "chip selected" } else { "chip" },
                                    onclick: move |_| {
                                        focused.set(Some(fmt));
                                        if selected_format() == Some(fmt) {
                                            on_clear.call(());
                                        } else {
                                            on_select.call(fmt);
                                        }
                                    },
                                    "{fmt.display_name()}"
                                }
                            }
                        }
                    }
                }

                ActionBar {
                    Button {
                        variant: ButtonVariant::Util,
                        onclick: move |_| on_close.call(()),
                        "Done"
                    }
                }

                HintDialog {
                    open: hint_open,
                    title: "Format",
                    HintBullets {
                        HintBullet {
                            "Tap a format to "
                            HintColored { color: "--accent-secondary", "select" }
                            " it; tap it again to clear. A deck has one format."
                        }
                        HintBullet {
                            "Tapping a format shows its pool, deck size, and command zone up top."
                        }
                        HintBullet {
                            "Changing the format clears your commander and signature spell."
                        }
                    }
                }
            }
        }
    }
}
