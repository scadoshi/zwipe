//! Export deck as plain-text decklist screen.
//!
//! What the screen knows and does is `zwipe_ui::screens::deck_export`; this
//! file draws its state and performs its effects.

use components::{Chip, HintBullet, HintBullets, HintDialog, HintKey, Toasts, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::user::models::hints::HINT_EXPORT;
use zwipe_ui::screens::deck_export::{self, Action, Board, Destination, Effect, State, Status};

use crate::inbound::components::action_dock::{self, DockIcon};
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen};
use crate::inbound::components::{guide_link, hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::components::skeletons::ChipRowSkeleton;

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
                    self.dispatch(match reply {
                        Ok(deck) => Action::Loaded {
                            ticket,
                            deck: Box::new(deck),
                        },
                        Err(_) => Action::Failed { ticket },
                    });
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
            Effect::Navigate(Destination::Deck(deck_id)) => {
                navigator.push(Router::ViewDeck { deck_id });
            }
        }
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

                HintDialog {
                    open: export_hint,
                    actions: guide_link::one_time(HINT_EXPORT),
                    title: "Exporting your deck",
                    HintBullets {
                        HintBullet { HintKey { "Export" } " picks which boards to include" }
                        HintBullet { "The two sheets at the bottom right put the decklist on your clipboard" }
                    }
                }

                div { class: "screen-content content-enter",
                    // Console stays a fixed flex child on the grid; the
                    // .import-scroll wrapper below owns the scrolling, so the
                    // decklist slides under the console (dictionary pattern).
                    div { class: "import-controls",
                        // The Export row ghosts with the decklist until the deck lands.
                        if status == Status::Loading {
                            ChipRowSkeleton { label: "md", chips: vec!["sm", "md", "sm"] }
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

                    div { class: "import-scroll",
                        div { class: "container-sm",
                        match export_text {
                            Some(text) => rsx! {
                                label { class: "label", r#for: "export-text", "{deck_export::DECKLIST}" }
                                textarea {
                                    id: "export-text",
                                    class: "input",
                                    style: "width:100%;min-height:16rem;resize:vertical;",
                                    readonly: true,
                                    value: "{text}",
                                }
                            },
                            None => {
                                if status == Status::Failed {
                                    rsx! { p { class: "text-muted", "{deck_export::LOAD_FAILED}" } }
                                } else {
                                    rsx! {
                                        div { class: "skeleton-export",
                                            div { class: "skeleton-bar skeleton-export-label" }
                                            div { class: "skeleton-export-textarea" }
                                        }
                                    }
                                }
                            }
                        }
                        }
                    }
                }

                // Copy as the action dock's icon, pinned bottom right.
                div { class: "app-float app-float-right",
                    div { class: "icon-pill",
                        button {
                            class: "icon-pill-btn",
                            "aria-label": deck_export::COPY,
                            onclick: move |_| driver.dispatch(Action::Copy),
                            DockIcon { path: action_dock::COPY }
                        }
                    }
                }
            }
    }
}
