//! Import cards screen: plain-text decklist or an Archidekt URL, in add or
//! replace mode. Both sources import into the selected board of this deck and
//! share the same result shape.
//!
//! What the screen knows and does is `zwipe_ui::screens::deck_import`; this
//! file holds the text being typed, draws the model's state and performs its
//! effects.

use components::icons::{self, DockIcon};
use components::{AlertDialog, Chip, DialogAction, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::screens::deck_import::{
    self, Action, Board, Destination, Effect, MODES, Source, State, mode_label,
};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_dialog::OneTimeHintDialog;
use crate::inbound::components::navigation::back_handler;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::card::components::undo_log::UndoStore;
use crate::inbound::screens::deck::missing_deck;

/// Import's model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    undo_store: UndoStore,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
    /// The board a replace asks about, while its confirm dialog is up.
    confirm: Signal<Option<Board>>,
    confirm_open: Signal<bool>,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    fn perform(self, effect: Effect) {
        let Self {
            navigator,
            toast,
            authed,
            mut undo_store,
            session,
            client,
            hint_open,
            mut confirm,
            mut confirm_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::ImportDeckCards {
                ticket,
                deck_id,
                text,
                board,
                mode,
            } => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("import_cards", |c, s| async move {
                            c.import_deck_cards(deck_id, &text, board, mode, &s).await
                        })
                        .await;
                    self.dispatch(missing_deck::settle(
                        reply,
                        |result| Action::Imported { ticket, result },
                        Action::Gone { ticket },
                        Action::Failed { ticket },
                    ));
                });
            }
            Effect::ImportArchidektDeck {
                ticket,
                deck_id,
                url,
                board,
                mode,
            } => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("import_cards", |c, s| async move {
                            c.import_archidekt_deck(deck_id, &url, board, mode, &s)
                                .await
                        })
                        .await;
                    self.dispatch(missing_deck::settle(
                        reply,
                        |result| Action::Imported { ticket, result },
                        Action::Gone { ticket },
                        Action::Failed { ticket },
                    ));
                });
            }
            Effect::ConfirmReplace(board) => {
                confirm.set(Some(board));
                confirm_open.set(true);
            }
            Effect::ClearUndo(deck_id) => undo_store.clear(deck_id),
            Effect::Toast(raised) => ui_toast::raise(toast, raised),
            // The screen navigates only from Back, to its deck.
            Effect::Navigate(destination) => {
                back_handler::back_or(navigator, route(destination));
            }
            Effect::Replace(destination) => {
                navigator.replace(route(destination));
            }
        }
    }
}

/// The app's route for each destination.
fn route(destination: Destination) -> Router {
    match destination {
        Destination::Deck(deck_id) => Router::ViewDeck { deck_id },
        Destination::DeckList => Router::DeckList {},
    }
}

#[component]
pub fn ImportDeck(deck_id: Uuid) -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::Import)),
        // An import overwrites the deck, so entries recorded before it are
        // semantically void; the whole undo stack clears on success
        // (disclosed in the hint below).
        undo_store: use_context(),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
        confirm: use_signal(|| None),
        confirm_open: use_signal(|| false),
    };
    // Import hint: auto-opens on first visit; the header "?" reopens it.
    let import_hint = driver.hint_open;
    // Offers the hint once per mount, while the screen first draws, so it
    // opens with the screen.
    use_hook(move || driver.dispatch(Action::Open(deck_id)));

    // The decklist and the link keep their own values when From switches.
    let mut text = use_signal(String::new);
    let mut url = use_signal(String::new);

    let model = driver.model.read();
    let source = model.source();
    let mode = model.mode();
    let board = model.board();
    let input = match source {
        Source::Text => text(),
        Source::Archidekt => url(),
    };
    let disabled = !model.can_submit(&input);
    let submit_label = model.submit_label();
    let explainer = model.explainer();
    let result = model.result().cloned();
    drop(model);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: deck_import::TITLE, hint: import_hint, back: move |()| driver.dispatch(Action::Back) }

                OneTimeHintDialog { open: import_hint, hint: OneTimeHint::Import }

                if let Some(asked) = (driver.confirm)() {
                    AlertDialog {
                        open: driver.confirm_open,
                        title: deck_import::replace_title(asked),
                        actions: vec![
                            DialogAction::Dismiss(deck_import::CANCEL.into()),
                            DialogAction::Confirm {
                                label: deck_import::REPLACE.into(),
                                danger: true,
                                on_confirm: Callback::new(move |()| driver.dispatch(Action::ConfirmReplace)),
                            },
                        ],
                        {deck_import::replace_body(asked)}
                    }
                }

                div { class: "screen-content content-enter",
                    // Console stays a fixed flex child on the grid; the
                    // .import-scroll wrapper below owns the scrolling, so
                    // entries slide under the console (dictionary pattern).
                    div { class: "import-controls",
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_import::FROM}" }
                            for value in Source::ALL {
                                Chip {
                                    selected: source == value,
                                    onclick: move |_| driver.dispatch(Action::PickSource(value)),
                                    "{value.label()}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_import::MODE}" }
                            for value in MODES {
                                Chip {
                                    selected: mode == value,
                                    onclick: move |_| driver.dispatch(Action::PickMode(value)),
                                    "{mode_label(value)}"
                                }
                            }
                        }
                        div { class: "chip-row",
                            span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_import::BOARD}" }
                            for value in Board::ALL {
                                Chip {
                                    selected: board == value,
                                    onclick: move |_| driver.dispatch(Action::PickBoard(value)),
                                    "{value.label()}"
                                }
                            }
                        }
                        p { class: "text-muted text-sm import-hint",
                            "{explainer.before}"
                            span { class: "hl-key", "{explainer.board}" }
                            "{explainer.after}"
                            if let Some(fills) = explainer.fills {
                                " {fills}"
                            }
                        }
                    }

                    // The box and the results sit in headed panels; the
                    // chips and the explainer above stay on the grid.
                    div { class: "import-scroll",
                        div { class: "container-sm import-panels",
                            div { class: "form-panel",
                                div { class: "settings-list-header",
                                    span { class: "settings-list-title",
                                        if source == Source::Text {
                                            "{deck_import::TEXT_LABEL}"
                                        } else {
                                            "{deck_import::URL_LABEL}"
                                        }
                                    }
                                }
                                div { class: "form-panel-fields",
                                    if source == Source::Text {
                                        textarea {
                                            id: "import-text",
                                            class: "input",
                                            "aria-label": deck_import::TEXT_LABEL,
                                            style: "width:100%;min-height:12rem;resize:vertical;",
                                            placeholder: deck_import::TEXT_PLACEHOLDER,
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
                                            "aria-label": deck_import::URL_LABEL,
                                            style: "width:100%;",
                                            r#type: "url",
                                            placeholder: deck_import::URL_PLACEHOLDER,
                                            value: "{url}",
                                            autocapitalize: "none",
                                            autocorrect: "off",
                                            spellcheck: "false",
                                            oninput: move |e| url.set(e.value()),
                                        }
                                    }
                                }
                            }

                            if let Some(r) = result {
                                if !deck_import::set_rows(&r).is_empty() {
                                    div { class: "form-panel",
                                        div { class: "settings-list-header",
                                            span { class: "settings-list-title", "{deck_import::SET_ON_DECK}" }
                                        }
                                        div { class: "form-panel-fields import-cards",
                                            for (value, tag) in deck_import::set_rows(&r) {
                                                div { class: "chip-bubble",
                                                    span { class: "font-light", "{value}" }
                                                    span { class: "font-light opacity-50", "{tag}" }
                                                }
                                            }
                                        }
                                    }
                                }
                                if !r.imported.is_empty() {
                                    div { class: "form-panel",
                                        div { class: "settings-list-header",
                                            span { class: "settings-list-title", "{deck_import::IMPORTED}" }
                                        }
                                        div { class: "form-panel-fields import-cards import-output",
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
                                    div { class: "form-panel",
                                        div { class: "settings-list-header",
                                            span { class: "settings-list-title", "{deck_import::FAILED}" }
                                        }
                                        div { class: "form-panel-fields import-cards import-output",
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

                // Import (or Replace) as the action dock's checkmark, pinned
                // bottom right.
                div { class: "action-dock action-dock-right",
                    button {
                        class: "action-dock-btn",
                        "aria-label": "{submit_label}",
                        disabled,
                        onclick: move |_| {
                            let source = driver.model.peek().source();
                            let input = match source {
                                Source::Text => text.peek().clone(),
                                Source::Archidekt => url.peek().clone(),
                            };
                            driver.dispatch(Action::Submit { input });
                        },
                        DockIcon { path: icons::CHECK }
                    }
                }
            }
    }
}
