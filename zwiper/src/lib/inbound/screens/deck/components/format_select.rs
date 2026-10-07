//! Full-screen format picker.
//!
//! Mirrors the tag picker's layout and behavior, but single-select: tapping a
//! format selects it (replacing any prior pick) and reveals its details in the
//! bar pinned at the top; tapping the selected format again clears it. Selecting
//! and clearing are reported through callbacks so the form can run its
//! command-zone cascade (clearing commander and signature spell on a change).

use crate::inbound::components::{hint_dialog::HintDialog, screen_header::ScreenHeader};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_components::{
    ActionBar, Button, ButtonVariant, HintBullet, HintBullets, TOAST_QUICK, use_overlay_back_action,
};
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
/// format, `on_clear` clears the current pick, `on_close` commits (Save, greyed
/// until the pick differs from the one it opened on), `on_cancel` reverts (the
/// parent restores the format + its command-zone cascade snapshot). Single-select
/// with a live cascade, so the revert is parent-owned; this picker only knows
/// whether there was something to revert, and says so.
#[component]
pub(crate) fn FormatSelect(
    open: Signal<bool>,
    selected_format: Signal<Option<Format>>,
    on_select: EventHandler<Format>,
    on_clear: EventHandler<()>,
    on_close: EventHandler<()>,
    on_cancel: EventHandler<()>,
) -> Element {
    let toast = use_toast();
    // The pick the picker opened on, so Save can grey out while nothing has
    // moved and Back can tell whether it is throwing anything away.
    let mut snapshot = use_signal(|| Option::<Format>::None);
    use_effect(move || {
        if open() {
            snapshot.set(*selected_format.peek());
        }
    });
    let unchanged = selected_format() == snapshot();

    // OS back gesture closes this overlay before touching the router. It maps
    // to Back, not Save: leaving a picker should abandon the pick, and the
    // parent's on_cancel restores the format + command-zone snapshot and closes.
    // Without this the back swipe fell through to the router and left the whole
    // edit screen (owner report 2026-08-17).
    let back = use_callback(move |_: ()| {
        if *selected_format.peek() != *snapshot.peek() {
            toast.info(
                "Format unchanged".to_string(),
                ToastOptions::default().duration(TOAST_QUICK),
            );
        }
        on_cancel.call(());
    });
    use_overlay_back_action(open.into(), back);

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
                        onclick: move |_| back.call(()),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Util,
                        disabled: unchanged,
                        onclick: move |_| on_close.call(()),
                        "Save"
                    }
                }

                HintDialog {
                    open: hint_open,
                    title: "Format",
                    HintBullets {
                        HintBullet { "Tap the picked format again to clear it" }
                        HintBullet { "Changing the format clears the commander and signature spell" }
                    }
                }
            }
        }
    }
}
