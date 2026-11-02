//! Full-screen format picker.
//!
//! Mirrors the tag picker's layout and behavior, but single-select: tapping a
//! format selects it (replacing any prior pick) and reveals its details in the
//! bar pinned at the top; tapping the selected format again clears it. Selecting
//! and clearing are reported through callbacks so the form can run its
//! command-zone cascade (clearing commander and signature spell on a change).

use components::{
    ActionBar, Button, ButtonVariant, HintBullet, HintBullets, HintDialog, use_overlay_back_action,
    use_toast,
};
use dioxus::prelude::*;
use zwipe_core::domain::deck::format::Format;
use zwipe_ui::deck::form;
use zwipe_ui::deck::pickers::{self, Draft, Picker};

use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::{guide_link, ui_toast};

/// In-place format picker. Toggled by `open`. `on_select` fires with the chosen
/// format, `on_clear` clears the current pick, `on_close` commits (Save, grayed
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
    // The pick the picker opened on, so Save can gray out while nothing has
    // moved and Back can tell whether it is throwing anything away.
    let mut draft = use_signal(|| Draft::format(None));
    use_effect(move || {
        if open() {
            draft.set(Draft::format(*selected_format.peek()));
        }
    });
    let unchanged = draft.read().is_unchanged(&selected_format());

    // OS back gesture closes this overlay before touching the router. It maps
    // to Back, not Save: leaving a picker should abandon the pick, and the
    // parent's on_cancel restores the format + command-zone snapshot and closes.
    let back = use_callback(move |(): ()| {
        if let Some(discarded) = draft.peek().back_toast(&selected_format.peek()) {
            ui_toast::raise(toast, discarded);
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
        pickers::formats_matching(&query())
    } else {
        Vec::new()
    };

    rsx! {
        div { class: "{screen_class}",
            // A screen over another paints its own backdrop, since .screen is
            // clear and the one beneath would show through.
            ::components::GridCells { still: true }
            if open() {
                ScreenHeader { title: Picker::Format.title(), hint: hint_open }

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
                                    for line in form::format_details(fmt) {
                                        li { "{line}" }
                                    }
                                }
                            } else {
                                div { class: "tag-def-name", "{pickers::PICKER_BAR_HINT}" }
                                div { class: "tag-def-text", "{Picker::Format.bar_empty()}" }
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
                        variant: ButtonVariant::Action,
                        onclick: move |_| back.call(()),
                        "Back"
                    }
                    Button {
                        variant: ButtonVariant::Action,
                        disabled: unchanged,
                        onclick: move |_| on_close.call(()),
                        "Save"
                    }
                }

                HintDialog {
                    open: hint_open,
                    actions: guide_link::screen(zwipe_ui::hint::HintTopic::Format),
                    title: Picker::Format.title(),
                    HintBullets {
                        for block in Picker::Format.hint().blocks {
                            HintBullet { "{block.plain()}" }
                        }
                    }
                }
            }
        }
    }
}
