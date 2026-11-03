//! Full-screen format picker.
//!
//! Mirrors the tag picker's layout and behavior, but single-select: tapping a
//! format selects it (replacing any prior pick) and reveals its details in the
//! bar pinned at the top; tapping the selected format again clears it. Selecting
//! and clearing are reported through callbacks so the form can run its
//! command-zone cascade (clearing commander and signature spell on a change).

use components::{HintBullet, HintBullets, HintDialog, use_overlay_back_action};
use dioxus::prelude::*;
use zwipe_core::domain::deck::format::Format;
use zwipe_ui::deck::form;
use zwipe_ui::deck::pickers::{self, Picker};

use crate::inbound::components::action_dock::{self, DockIcon};
use crate::inbound::components::guide_link;
use crate::inbound::components::screen_header::ScreenHeader;

/// In-place format picker, up while `open`. A chip tap goes to `on_tap`, which picks the format or clears it when it is the one picked, with the command-zone knock-on clears. The dock's checkmark (`on_save`) stays off while `unchanged`; the header's back arrow (`on_back`) and the OS back gesture put the format and command zone back as they were when the picker opened.
#[component]
pub(crate) fn FormatSelect(
    open: ReadSignal<bool>,
    selected_format: Option<Format>,
    unchanged: bool,
    on_tap: EventHandler<Format>,
    on_save: EventHandler<()>,
    on_back: EventHandler<()>,
) -> Element {
    // OS back gesture closes this overlay before touching the router. It maps
    // to Back, not Save: leaving a picker abandons the pick.
    let back = use_callback(move |(): ()| on_back.call(()));
    use_overlay_back_action(open, back);

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
                ScreenHeader { title: Picker::Format.title(), hint: hint_open, back: move |()| back.call(()) }

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
                                    class: if selected_format == Some(fmt) { "chip selected" } else { "chip" },
                                    onclick: move |_| {
                                        focused.set(Some(fmt));
                                        on_tap.call(fmt);
                                    },
                                    "{fmt.display_name()}"
                                }
                            }
                        }
                    }
                }

                // Save as the action dock's checkmark, pinned bottom right.
                div { class: "app-float app-float-right",
                    div { class: "icon-pill",
                        button {
                            class: "icon-pill-btn",
                            "aria-label": "Save",
                            disabled: unchanged,
                            onclick: move |_| on_save.call(()),
                            DockIcon { path: action_dock::CHECK }
                        }
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
