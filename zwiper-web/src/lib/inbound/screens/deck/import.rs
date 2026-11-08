//! Import: a pasted decklist or an Archidekt link into one board of the
//! deck, adding or replacing, with what came in and what did not.
//!
//! A screen at every width. On a wide window the controls and the input sit
//! on the left and the results on the right, so the paste can be fixed
//! while reading what failed. Import (or Replace) is the checkmark in the
//! dock at the bottom right; Ctrl+Enter (Cmd+Enter on a Mac) runs it too.

use components::icons::{CHECK, DockIcon};
use components::{AlertDialog, Chip, DialogAction, TOAST_NORMAL, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_core::domain::deck::{ImportDeckCardsResult, ImportMode};
use zwipe_ui::deck::form::{ImportTone, import_outcome};
use zwipe_ui::screens::deck_import;
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen};

use crate::inbound::card_stores::CardStores;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// Where the cards come from.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum Source {
    /// A pasted decklist.
    Text,
    /// An Archidekt deck link.
    Archidekt,
}

/// The board chips: the label and the board sent; `Main` sends none, so a
/// paste's section headers still sort its cards.
const BOARDS: [(&str, Option<&str>); 3] = [
    ("Main", None),
    ("Maybe", Some("maybeboard")),
    ("Side", Some("sideboard")),
];

/// The Import screen for `deck_id`.
#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::Import));
    let authed = use_authed();
    let stores: CardStores = use_context();
    let toast = use_toast();
    let hint_open = use_one_time_hint(OneTimeHint::Import);
    let mut source = use_signal(|| Source::Text);
    let mut mode = use_signal(|| ImportMode::Add);
    let mut board: Signal<Option<&'static str>> = use_signal(|| None);
    let mut text = use_signal(String::new);
    let mut url = use_signal(String::new);
    let mut loading = use_signal(|| false);
    let mut result: Signal<Option<ImportDeckCardsResult>> = use_signal(|| None);

    let blank = match source() {
        Source::Text => text().trim().is_empty(),
        Source::Archidekt => url().trim().is_empty(),
    };
    let disabled = loading() || blank;

    let mut confirm_open = use_signal(|| false);
    let run_import = use_callback(move |()| {
        let (board, mode, source) = (*board.peek(), *mode.peek(), *source.peek());
        let (text, url) = (text.peek().clone(), url.peek().clone());
        result.set(None);
        loading.set(true);
        spawn(async move {
            let reply = authed
                .try_run("import_cards", |c, s| async move {
                    match source {
                        Source::Text => c.import_deck_cards(deck_id, &text, board, mode, &s).await,
                        Source::Archidekt => {
                            c.import_archidekt_deck(deck_id, &url, board, mode, &s)
                                .await
                        }
                    }
                })
                .await;
            loading.set(false);
            let Ok(reply) = reply else {
                return;
            };
            if !reply.imported.is_empty() {
                let mut undo = stores.undo;
                undo.write().clear(deck_id);
            }
            let (tone, message) = import_outcome(reply.imported.len(), reply.unresolved.len());
            let quick = ToastOptions::default().duration(TOAST_QUICK);
            match tone {
                ImportTone::Info => toast.info(message, quick),
                ImportTone::Success => toast.success(message, quick),
                ImportTone::Error => {
                    toast.error(message, ToastOptions::default().duration(TOAST_NORMAL));
                }
            }
            result.set(Some(reply));
        });
    });
    // A replace asks first, since it clears the board and the undo history.
    let mut submit = move || {
        if mode.peek().is_replace() {
            confirm_open.set(true);
        } else {
            run_import.call(());
        }
    };

    let board_word = board().unwrap_or("mainboard");
    let submit_label = if loading() {
        "Importing…"
    } else if mode().is_replace() {
        "Replace"
    } else {
        "Import"
    };
    let on_hint = use_callback(move |()| {
        let mut hint_open = hint_open;
        hint_open.set(true);
    });
    rsx! {
        ScreenFrame {
            title: "Import",
            on_hint,
            div {
                class: "screen-content content-enter web-import",
                onkeydown: move |event: Event<KeyboardData>| {
                    let modifiers = event.modifiers();
                    let chord = modifiers.ctrl() || modifiers.meta();
                    if event.key() == Key::Enter && chord && !disabled {
                        event.prevent_default();
                        submit();
                    }
                },
                div { class: "import-controls",
                    div { class: "chip-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "From" }
                        for (label, value) in [("Text", Source::Text), ("Archidekt", Source::Archidekt)] {
                            Chip {
                                key: "{label}",
                                selected: source() == value,
                                onclick: move |_| source.set(value),
                                "{label}"
                            }
                        }
                    }
                    div { class: "chip-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "Mode" }
                        for (label, value) in [("Add", ImportMode::Add), ("Replace", ImportMode::Replace)] {
                            Chip {
                                key: "{label}",
                                selected: mode() == value,
                                onclick: move |_| mode.set(value),
                                "{label}"
                            }
                        }
                    }
                    div { class: "chip-row",
                        span { class: "stat-chip stat-chip-p1 chip-row-label", "Board" }
                        for (label, value) in BOARDS {
                            Chip {
                                key: "{label}",
                                selected: board() == value,
                                onclick: move |_| board.set(value),
                                "{label}"
                            }
                        }
                    }
                    p { class: "text-muted text-sm import-hint",
                        if mode().is_replace() {
                            "Removes all existing cards in the "
                            span { class: "hl-key", "{board_word}" }
                            ", replacing with imported cards."
                        } else {
                            "Adds imported cards into the "
                            span { class: "hl-key", "{board_word}" }
                            ". Existing cards stay but take new quantities if applicable."
                        }
                    }
                }
                div { class: "import-scroll",
                    div { class: "container-sm web-import-body",
                        div { class: "web-form-panel web-import-input",
                            div { class: "settings-list-header web-form-head",
                                span { class: "settings-list-title",
                                    if source() == Source::Text {
                                        "Paste decklist"
                                    } else {
                                        "Archidekt deck URL"
                                    }
                                }
                            }
                            div { class: "web-form-fields",
                                if source() == Source::Text {
                                    textarea {
                                        id: "import-text",
                                        class: "input",
                                        "aria-label": "Paste decklist",
                                        style: "width:100%;min-height:12rem;resize:vertical;",
                                        placeholder: "1 Krenko, Mob Boss\n1 Urza, Lord High Artificer\n1 Sol Ring",
                                        value: "{text}",
                                        autocapitalize: "none",
                                        autocorrect: "off",
                                        spellcheck: "false",
                                        oninput: move |e| text.set(e.value()),
                                    }
                                } else {
                                    input {
                                        id: "import-url",
                                        class: "input",
                                        "aria-label": "Archidekt deck URL",
                                        style: "width:100%;",
                                        r#type: "url",
                                        placeholder: "https://archidekt.com/decks/...",
                                        value: "{url}",
                                        autocapitalize: "none",
                                        autocorrect: "off",
                                        spellcheck: "false",
                                        oninput: move |e| url.set(e.value()),
                                    }
                                }
                            }
                        }
                        div { class: "web-import-results",
                            if let Some(r) = result() {
                                if !r.imported.is_empty() {
                                    div { class: "web-form-panel",
                                        div { class: "settings-list-header web-form-head",
                                            span { class: "settings-list-title", "{deck_import::IMPORTED}" }
                                        }
                                        div { class: "web-form-fields web-import-cards",
                                            for card in r.imported.iter() {
                                                div { class: "chip-bubble",
                                                    span { class: "font-light", "{card.name}" }
                                                    span { class: "font-light opacity-50", "x{card.quantity}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !r.unresolved.is_empty() {
                                    div { class: "web-form-panel",
                                        div { class: "settings-list-header web-form-head",
                                            span { class: "settings-list-title", "{deck_import::FAILED}" }
                                        }
                                        div { class: "web-form-fields web-import-cards",
                                            for card in r.unresolved.iter() {
                                                div { class: "chip-bubble-error",
                                                    span { class: "font-light", "{card.name}" }
                                                    span { class: "font-light opacity-50", "{card.reason}" }
                                                }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                    }
                }
            }
            div { class: "action-dock action-dock-right",
                button {
                    class: "action-dock-btn",
                    "aria-label": submit_label,
                    disabled,
                    onclick: move |_| submit(),
                    DockIcon { path: CHECK }
                }
            }
        }
        OneTimeHintDialog { hint: OneTimeHint::Import, open: hint_open }
        AlertDialog {
            open: confirm_open,
            title: deck_import::replace_title(ui_board(board())),
            actions: vec![
                DialogAction::Dismiss(deck_import::CANCEL.into()),
                DialogAction::Confirm {
                    label: deck_import::REPLACE.into(),
                    danger: true,
                    on_confirm: run_import,
                },
            ],
            {deck_import::replace_body(ui_board(board()))}
        }
    }
}

/// The shared model's board for a board as sent: none is the mainboard.
fn ui_board(board: Option<&str>) -> deck_import::Board {
    match board {
        Some("maybeboard") => deck_import::Board::Maybe,
        Some("sideboard") => deck_import::Board::Side,
        _ => deck_import::Board::Main,
    }
}
