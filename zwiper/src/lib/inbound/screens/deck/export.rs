//! Export deck as plain-text decklist screen.
//!
//! What the screen knows and does is `zwipe_ui::screens::deck_export`; this
//! file draws its state and performs its effects.

use components::icons::{self};
use components::{Chip, DockAction, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::hints::OneTimeHint;
use zwipe_ui::screens::deck_export::{self, Action, Board, Destination, Effect, State, Status};

use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::hint_dialog::OneTimeHintDialog;
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::components::skeletons::{ChipGhost, ChipRowGhost};
use crate::inbound::screens::deck::missing_deck;

/// Export's model with the handles that perform its effects. Copy, so every
/// handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    hint_open: Signal<bool>,
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
            session,
            client,
            hint_open,
            ..
        } = self;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(hint.key(), session, client, hint_open);
            }
            Effect::FetchDeck { ticket, deck_id } => {
                spawn(async move {
                    // try_run reports a failure and raises its error toast.
                    let reply = authed
                        .try_run("load_deck", |c, s| async move {
                            c.fetch_deck(deck_id, &s).await
                        })
                        .await;
                    self.dispatch(missing_deck::settle(
                        reply,
                        |deck| Action::Loaded {
                            ticket,
                            deck: Box::new(deck),
                        },
                        Action::Gone { ticket },
                        Action::Failed { ticket },
                    ));
                });
            }
            Effect::Copy { text, done } => {
                let js = format!(
                    "navigator.clipboard.writeText({})",
                    serde_json::to_string(&text).unwrap_or_default()
                );
                document::eval(&js);
                ui_toast::raise(toast, done);
            }
            Effect::Navigate(destination) => {
                navigator.push(route(destination));
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
pub fn ExportDeck(deck_id: Uuid) -> Element {
    let driver = Driver {
        model: use_signal(State::new),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(Screen::Deck(DeckScreen::Export)),
        session: use_context(),
        client: use_context(),
        hint_open: use_signal(|| false),
    };
    // Export hint: auto-opens on first visit; the header "?" reopens it.
    let export_hint = driver.hint_open;
    // Offers the hint and fetches the deck once per mount, while the screen
    // first draws, so the hint opens with the screen.
    use_hook(move || driver.dispatch(Action::Load(deck_id)));

    let model = driver.model.read();
    let status = model.status();
    let on = Board::ALL.map(|board| (board, model.is_on(board)));
    let export_text = model.text();
    drop(model);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: deck_export::TITLE, hint: export_hint, back: move |()| driver.dispatch(Action::Back) }

                OneTimeHintDialog { open: export_hint, hint: OneTimeHint::Export }

                div { class: "screen-content content-enter",
                    // Console stays a fixed flex child on the grid; the
                    // .import-scroll wrapper below owns the scrolling, so the
                    // decklist slides under the console (dictionary pattern).
                    div { class: "import-controls",
                        // The Export row ghosts with the decklist until the deck lands.
                        if status == Status::Loading {
                            ChipRowGhost {
                                label: deck_export::BOARDS,
                                chips: Board::ALL.map(|board| ChipGhost::Text(board.label().to_string())).to_vec(),
                            }
                        } else {
                            div { class: "chip-row",
                                span { class: "stat-chip stat-chip-p1 chip-row-label", "{deck_export::BOARDS}" }
                                for (board, selected) in on {
                                    Chip {
                                        selected,
                                        onclick: move |_| driver.dispatch(Action::Toggle(board)),
                                        "{board.label()}"
                                    }
                                }
                            }
                        }
                    }

                    // The decklist sits in a headed panel; the chips above
                    // stay on the grid.
                    div { class: "import-scroll",
                        div { class: "container-sm import-panels",
                        match export_text {
                            Some(text) => rsx! {
                                div { class: "form-panel",
                                    div { class: "settings-list-header",
                                        span { class: "settings-list-title", "{deck_export::DECKLIST}" }
                                    }
                                    div { class: "form-panel-fields",
                                        textarea {
                                            id: "export-text",
                                            class: "input",
                                            "aria-label": deck_export::DECKLIST,
                                            style: "width:100%;min-height:16rem;resize:vertical;",
                                            readonly: true,
                                            value: "{text}",
                                        }
                                    }
                                }
                            },
                            None => {
                                if status == Status::Failed {
                                    rsx! {
                                        div { class: "message-empty",
                                            span { class: "chip-note", {deck_export::LOAD_FAILED} }
                                        }
                                    }
                                } else {
                                    rsx! {
                                        div { class: "form-panel",
                                            div { class: "skeleton-card-header",
                                                div { class: "skeleton-bar skeleton-section-label" }
                                            }
                                            div { class: "form-panel-fields",
                                                div { class: "skeleton-export-textarea" }
                                            }
                                        }
                                    }
                                }
                            }
                        }
                        }
                    }
                }

                // Copy as the action dock's icon, pinned bottom right.
                div { class: "action-dock action-dock-right",
                    DockAction {
                        label: deck_export::COPY,
                        icon: icons::COPY,
                        onclick: move |()| driver.dispatch(Action::Copy),
                    }
                }
            }
    }
}
