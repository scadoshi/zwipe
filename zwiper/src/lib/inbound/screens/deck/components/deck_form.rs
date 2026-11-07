//! The Create deck and Edit deck screen: one form in two modes.
//!
//! What the form knows and does is `zwipe_ui::screens::deck_form`; this file draws its state, opens the swipe pickers and performs its effects.

use components::{DockAction, TOAST_NORMAL, ToastOptions, Toasts, icons, use_toast};
use dioxus::prelude::*;
use dioxus::router::Navigator;
use tokio::time;
use uuid::Uuid;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::CardQuery;
use zwipe_core::domain::deck::DeckTagView;
use zwipe_ui::deck::form::{self, Slot};
use zwipe_ui::deck::pickers::{Picker, TagPicker};
use zwipe_ui::generation::Ticket;
use zwipe_ui::screens::deck_form::{
    self as model, Action, Destination, Effect, Open, State, Status,
};

use super::deck_fields::{DeckFields, DeckFieldsHint};
use super::format_select::FormatSelect;
use super::oracle_tag_select::OracleTagSelect;
use super::skeletons::EditDeckSkeleton;
use super::swipe_select::{SwipeMode, SwipeSelect};
use super::tag_select::TagSelect;
use crate::inbound::components::auth::authed::{Authed, use_authed};
use crate::inbound::components::catalog_cache::CatalogCache;
use crate::inbound::components::navigation::leave_guard::{LeaveCopy, LeaveGuard, use_leave_guard};
use crate::inbound::components::screen_header::ScreenHeader;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{DeckScreen, Screen, component, screen};
use crate::inbound::components::{hint_dialog, ui_toast};
use crate::inbound::router::Router;
use crate::inbound::screens::deck::missing_deck;

/// The form's model with the handles that perform its effects. Copy, so every handler and task holds its own.
#[derive(Clone, Copy)]
struct Driver {
    model: Signal<State>,
    /// The name input's text, kept in step with the model.
    name: Signal<String>,
    navigator: Navigator,
    toast: Toasts,
    authed: Authed,
    session: Signal<Option<Session>>,
    client: Signal<ZwipeClient>,
    usage_buffer: Signal<UsageBuffer>,
    hint_open: Signal<bool>,
    /// Asks before the form is left with changes not yet saved.
    leave: LeaveGuard,
}

impl Driver {
    /// Applies `action` to the model and performs what it returns.
    fn dispatch(self, action: Action) {
        let mut model = self.model;
        let effects = model.write().update(action);
        self.sync_name();
        // Armed or not before the moves are made, so a save's own move to
        // the deck goes through and a Back with changes asks.
        self.leave.set(model.peek().has_unsaved_changes());
        for effect in effects {
            self.perform(effect);
        }
    }

    /// Writes the model's name into the name input when a load changed it.
    fn sync_name(self) {
        let mut name = self.name;
        let held = self.model.peek().values().name.clone();
        if *name.peek() != held {
            name.set(held);
        }
    }

    /// Performs one effect, feeding any reply back through [`Self::dispatch`].
    /// Fetches Edit's deck; a gone deck is [`Action::Gone`].
    fn fetch_deck(self, ticket: Ticket, deck_id: Uuid) {
        let authed = self.authed;
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
                    profile: Box::new(deck.deck_profile),
                },
                Action::Gone { ticket },
                Action::Failed { ticket },
            ));
        });
    }

    fn perform(self, effect: Effect) {
        let authed = self.authed;
        let client = self.client;
        match effect {
            Effect::OfferHint(hint) => {
                hint_dialog::open_and_record_hint(
                    hint.key(),
                    self.session,
                    self.client,
                    self.hint_open,
                );
            }
            Effect::FetchDeck { ticket, deck_id } => self.fetch_deck(ticket, deck_id),
            Effect::FetchCard {
                ticket,
                slot,
                card_id,
            } => self.fetch_card(ticket, slot, card_id),
            Effect::Debounce { ticket, wait } => {
                spawn(async move {
                    time::sleep(wait).await;
                    self.dispatch(Action::Debounced { ticket });
                });
            }
            Effect::SearchCards { ticket, query } => {
                let Some(session) = self.session.peek().clone() else {
                    return;
                };
                self.usage_buffer.peek().record_search();
                spawn(async move {
                    // Quiet: fires per debounced keystroke; a failure only
                    // closes the results.
                    match client().search_cards(&query, &session).await {
                        Ok(cards) => self.dispatch(Action::Searched { ticket, cards }),
                        Err(e) => {
                            tracing::error!("search error: {e}");
                            self.dispatch(Action::Failed { ticket });
                        }
                    }
                });
            }
            Effect::FindPartner {
                picked,
                mate,
                query,
            } => self.find_partner(picked, mate, *query),
            Effect::CreateDeck { ticket, request } => {
                spawn(async move {
                    let reply = authed
                        .try_run("create_deck", |c, s| async move {
                            c.create_deck_profile(&request, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(created) => Action::Created {
                            ticket,
                            deck_id: created.id,
                        },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::UpdateDeck {
                ticket,
                deck_id,
                request,
            } => {
                spawn(async move {
                    let reply = authed
                        .try_run("save_profile", |c, s| async move {
                            c.update_deck_profile(deck_id, &request, &s).await
                        })
                        .await;
                    self.dispatch(match reply {
                        Ok(_) => Action::Updated { ticket },
                        Err(_) => Action::Failed { ticket },
                    });
                });
            }
            Effect::Navigate(destination) => match route(destination) {
                Some(route) => {
                    self.navigator.push(route);
                }
                None => self.navigator.go_back(),
            },
            Effect::Replace(destination) => match route(destination) {
                Some(route) => {
                    self.navigator.replace(route);
                }
                None => self.navigator.go_back(),
            },
            Effect::Toast(toast) => ui_toast::raise(self.toast, toast),
        }
    }

    /// Fetches one of Edit's saved command-zone cards. A failure is reported and toasted here, as the card fetch runs outside the request runner.
    fn fetch_card(self, ticket: Ticket, slot: Slot, card_id: Uuid) {
        let client = self.client;
        spawn(async move {
            match client().fetch_card(card_id).await {
                Ok(card) => self.dispatch(Action::CardLoaded {
                    ticket,
                    card: Box::new(card),
                }),
                Err(e) => {
                    let op = match slot {
                        Slot::Commander => "load_commander",
                        Slot::Partner => "load_partner",
                        Slot::Background => "load_background",
                        Slot::SignatureSpell => "load_signature_spell",
                    };
                    self.usage_buffer.peek().report_error(
                        screen::DECK_EDIT,
                        component::NONE,
                        op,
                        &e,
                    );
                    self.toast.error(
                        e.to_user_message(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                    self.dispatch(Action::Failed { ticket });
                }
            }
        });
    }

    /// Searches the partner a "Partner with" commander names; a failure only logs.
    fn find_partner(self, picked: Uuid, mate: String, query: CardQuery) {
        let client = self.client;
        spawn(async move {
            let Some(session) = self.session.peek().clone() else {
                return;
            };
            match client().search_cards(&query, &session).await {
                Ok(cards) => {
                    if form::named_partner(&cards, &mate).is_none() {
                        tracing::warn!("partner autofill: no exact match for {mate}");
                    }
                    self.dispatch(Action::PartnerSearched {
                        picked,
                        mate,
                        cards,
                    });
                }
                Err(e) => tracing::warn!("partner autofill search failed: {e}"),
            }
        });
    }
}

/// The route a destination opens, or `None` for the screen before this one.
fn route(destination: Destination) -> Option<Router> {
    match destination {
        Destination::Previous => None,
        Destination::Deck(deck_id) => Some(Router::ViewDeck { deck_id }),
        Destination::DeckList => Some(Router::DeckList {}),
    }
}

/// The Create deck or Edit deck screen, as `open` says.
#[component]
pub(crate) fn DeckForm(open: Open) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let (telemetry, host_screen) = match open {
        Open::Create { .. } => (Screen::Deck(DeckScreen::Create), screen::DECK_CREATE),
        Open::Edit(_) => (Screen::Deck(DeckScreen::Edit), screen::DECK_EDIT),
    };

    // Opening the form takes in the seed or starts the deck fetch, and keeps
    // what the model asks for to perform below.
    let mut opening = Vec::new();
    let model = use_signal(|| {
        let mut state = State::new();
        opening = state.update(Action::Load(open.clone()));
        state
    });
    let driver = Driver {
        model,
        name: use_signal(|| model.peek().values().name.clone()),
        navigator: use_navigator(),
        toast: use_toast(),
        authed: use_authed(telemetry),
        session,
        client,
        usage_buffer: use_context(),
        hint_open: use_signal(|| false),
        leave: use_leave_guard(LeaveCopy {
            title: model::LEAVE_TITLE,
            body: model::LEAVE_BODY,
            stay: model::STAY,
            leave: model::LEAVE,
        }),
    };
    // A change that reaches the model some other way arms the guard too.
    let leave = driver.leave;
    use_effect(move || leave.set(model.read().has_unsaved_changes()));
    let hint_open = driver.hint_open;
    use_hook(move || {
        for effect in opening {
            driver.perform(effect);
        }
    });

    // Server-driven deck-tag catalog for the picker and the oracle tag seed:
    // read from the app-wide catalog cache (warmed after session), empty until
    // loaded.
    let cache: CatalogCache = use_context();
    use_effect(move || {
        if let Some(s) = session() {
            cache.ensure_deck_tags(client, s);
        }
    });
    let deck_tags_catalog = use_memo(move || -> Option<Vec<DeckTagView>> {
        cache.deck_tags.cell().read().loaded().cloned()
    });
    use_effect(move || {
        driver.dispatch(Action::Catalog(deck_tags_catalog().unwrap_or_default()));
    });

    // The name input writes its own signal; the model hears each change.
    let name = driver.name;
    use_effect(move || {
        let typed = name();
        if model.peek().values().name != typed {
            driver.dispatch(Action::SetName(typed));
        }
    });

    // Each command-zone field's Zwipe-select overlay, opened by its Swipe chip.
    let mut show_commander_swipe = use_signal(|| false);
    let mut show_partner_swipe = use_signal(|| false);
    let mut show_background_swipe = use_signal(|| false);
    let mut show_signature_spell_swipe = use_signal(|| false);
    let open_swipe = move |slot: Slot| {
        let mut show = match slot {
            Slot::Commander => show_commander_swipe,
            Slot::Partner => show_partner_swipe,
            Slot::Background => show_background_swipe,
            Slot::SignatureSpell => show_signature_spell_swipe,
        };
        show.set(true);
    };

    // Reactive Zwipe-select modes: derived from the current format / commander.
    let commander_mode = use_memo(move || model.read().values().format.map(SwipeMode::Commander));
    let partner_mode = use_memo(|| Some(SwipeMode::Partner));
    let background_mode = use_memo(|| Some(SwipeMode::Background));
    let spell_mode = use_memo(move || {
        model
            .read()
            .values()
            .commander
            .as_ref()
            .map(|c| SwipeMode::SignatureSpell(c.scryfall_data.color_identity.clone()))
    });

    let format_open = use_memo(move || model.read().picker() == Some(Picker::Format));
    let tags_open = use_memo(move || model.read().picker() == Some(Picker::DeckTags));
    let oracle_tags_open = use_memo(move || model.read().picker() == Some(Picker::OracleTags));

    let state = model.read();
    let mode = state.mode();
    let status = state.status();
    let saving = state.is_saving();
    let save_label = state.save_label();
    let unchanged = state.picker_unchanged();
    let selected_format = state.values().format;
    let deck_tags = state.tags(TagPicker::DeckTags).to_vec();
    let oracle_tags = state.tags(TagPicker::OracleTags).to_vec();
    drop(state);
    let dispatch = move |action: Action| driver.dispatch(action);

    rsx! {
            div { class: "screen",
                ScreenHeader { title: mode.title(), hint: hint_open, back: move |()| driver.dispatch(Action::Back) }

                div { class: "screen-content content-enter",
                div { class: "deck-form",
                    match status {
                        Status::Ready => rsx! {
                            form { class: "flex-col text-center",
                                DeckFields {
                                    model,
                                    name,
                                    on_action: dispatch,
                                    on_swipe: open_swipe,
                                }
                            }
                        },
                        Status::Failed => rsx! {
                            div { class: "message-empty",
                                span { class: "chip-note", {model::LOAD_FAILED} }
                            }
                        },
                        Status::Loading => rsx! { EditDeckSkeleton {} }
                    }
                }
            }

            // Save as the action dock's checkmark, pinned bottom right, while
            // there is something to save; dimmed while the save is out.
            if let Some(label) = save_label {
                div { class: "action-dock action-dock-right",
                    DockAction {
                        label,
                        icon: icons::CHECK,
                        disabled: saving,
                        onclick: move |()| driver.dispatch(Action::Save),
                    }
                }
            }
            }
            SwipeSelect {
                host_screen,
                open: show_commander_swipe,
                mode: commander_mode,
                on_select: move |card: Card| {
                    driver.dispatch(Action::PickSwiped { slot: Slot::Commander, card: Box::new(card) });
                    show_commander_swipe.set(false);
                },
                on_close: move |()| show_commander_swipe.set(false),
            }
            SwipeSelect {
                host_screen,
                open: show_partner_swipe,
                mode: partner_mode,
                on_select: move |card: Card| {
                    driver.dispatch(Action::PickSwiped { slot: Slot::Partner, card: Box::new(card) });
                    show_partner_swipe.set(false);
                },
                on_close: move |()| show_partner_swipe.set(false),
            }
            SwipeSelect {
                host_screen,
                open: show_background_swipe,
                mode: background_mode,
                on_select: move |card: Card| {
                    driver.dispatch(Action::PickSwiped { slot: Slot::Background, card: Box::new(card) });
                    show_background_swipe.set(false);
                },
                on_close: move |()| show_background_swipe.set(false),
            }
            SwipeSelect {
                host_screen,
                open: show_signature_spell_swipe,
                mode: spell_mode,
                on_select: move |card: Card| {
                    driver.dispatch(Action::PickSwiped { slot: Slot::SignatureSpell, card: Box::new(card) });
                    show_signature_spell_swipe.set(false);
                },
                on_close: move |()| show_signature_spell_swipe.set(false),
            }

            TagSelect {
                open: tags_open,
                selected: deck_tags,
                catalog: deck_tags_catalog().unwrap_or_default(),
                unchanged,
                on_toggle: move |slug: String| driver.dispatch(Action::ToggleTag { picker: TagPicker::DeckTags, slug }),
                on_clear: move |()| driver.dispatch(Action::ClearTags(TagPicker::DeckTags)),
                on_save: move |()| driver.dispatch(Action::SavePicker),
                on_back: move |()| driver.dispatch(Action::BackPicker),
            }

            OracleTagSelect {
                open: oracle_tags_open,
                selected: oracle_tags,
                unchanged,
                on_toggle: move |slug: String| driver.dispatch(Action::ToggleTag { picker: TagPicker::OracleTags, slug }),
                on_clear: move |()| driver.dispatch(Action::ClearTags(TagPicker::OracleTags)),
                on_adopt: move |slug: String| driver.dispatch(Action::AdoptOracleTag(slug)),
                on_save: move |()| driver.dispatch(Action::SavePicker),
                on_back: move |()| driver.dispatch(Action::BackPicker),
            }

            FormatSelect {
                open: format_open,
                selected_format,
                unchanged,
                on_tap: move |fmt| driver.dispatch(Action::TapFormat(fmt)),
                on_save: move |()| driver.dispatch(Action::SavePicker),
                on_back: move |()| driver.dispatch(Action::BackPicker),
            }

            DeckFieldsHint { open: hint_open, hint: mode.hint() }
    }
}
