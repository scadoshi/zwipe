//! SPIKE, never merged: the card screen drawn two ways from existing components, for Scotty to
//! pick a look. `?look=a` is two panes on a wide window, `?look=b` one column everywhere.

use components::{CardDetailsPanel, CardRow, FlippableCardImage, LoadFailed, PickerSearch};
use dioxus::prelude::*;
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::scryfall_data::image_uris::ImageSize;
use zwipe_core::domain::card::slug::card_slug;
use zwipe_ui::screens::card_lookup::{self, Action, Destination, Effect, Open, State, Status};

use crate::inbound::cards::printing_sheet::PrintingSheet;
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

const SPIKE_CSS: &str = r"
.cs-search { max-width: 560px; margin: 0 auto var(--space-4); }
.cs-a { display: grid; grid-template-columns: minmax(0, 400px) minmax(0, 560px); gap: var(--space-5); justify-content: center; align-items: start; }
.cs-a .cs-image { width: 100%; }
.cs-b { max-width: 560px; margin: 0 auto; display: flex; flex-direction: column; gap: var(--space-4); }
.cs-b .cs-image { width: min(100%, 300px); margin: 0 auto; }
.cs-image img { width: 100%; height: auto; border-radius: var(--radius-card, 4.75% / 3.5%); }
.cs-actions { display: flex; gap: var(--space-2); flex-wrap: wrap; }
.cs-bar { position: sticky; bottom: 0; display: flex; gap: var(--space-2); justify-content: center; padding: var(--space-3) 0; background: var(--bg-primary); border-top: 1px solid var(--border-secondary); z-index: 1; }
.cs-list { max-width: 720px; margin: 0 auto; padding: 0 var(--space-4); display: flex; flex-direction: column; gap: var(--space-2); }
.cs-list .card-row-qty { display: none; }
.cs-list h2 { margin: 0 0 var(--space-2); }
.cs-message { text-align: center; padding: var(--space-5) 0; }
@media (max-width: 899px) {
  .cs-a { grid-template-columns: minmax(0, 1fr); }
  .cs-a .cs-image { width: min(100%, 340px); margin: 0 auto; }
}
";

/// SPIKE: `/card/:slug?:look`.
#[component]
pub fn CardSpike(slug: String, look: String) -> Element {
    rsx! { SpikeScreen { key: "{slug}", open: Open::Named { slug }, look } }
}

/// SPIKE: `/card/:slug/:id?:look`.
#[component]
pub fn CardSpikePrinting(slug: String, id: Uuid, look: String) -> Element {
    rsx! { SpikeScreen { key: "{id}", open: Open::Printing { slug, id }, look } }
}

#[component]
fn SpikeScreen(open: Open, look: String) -> Element {
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let navigator = use_navigator();
    let toast = components::use_toast();
    let mut model = use_signal(State::new);
    let mut printings_open = use_signal(|| false);
    let expanded = use_signal(|| None::<Uuid>);
    let signed_in = move || session.read().as_ref().is_some_and(|s| !s.is_expired());
    let look_b = look == "b";

    let route_of = {
        let look = look.clone();
        move |to: Destination| match to {
            Destination::Named { slug, .. } => Route::CardSpike {
                slug: slug.to_string(),
                look: look.clone(),
            },
            Destination::Printing { slug, id, .. } => Route::CardSpikePrinting {
                slug,
                id,
                look: look.clone(),
            },
            Destination::Login => Route::Login {},
        }
    };

    let dispatch = use_callback(move |action: Action| {
        let effects = model.write().update(action);
        for effect in effects {
            match effect {
                Effect::FetchNamed { ticket, slug } => {
                    spawn(async move {
                        let http = client.peek().clone();
                        let reply = match http.fetch_named_cards(&slug).await {
                            Ok(cards) => Action::Named { ticket, cards },
                            Err(_) => Action::Failed { ticket },
                        };
                        let effects = model.write().update(reply);
                        drop(effects);
                    });
                }
                Effect::FetchCard { ticket, id } => {
                    spawn(async move {
                        let http = client.peek().clone();
                        let reply = match http.fetch_card(id).await {
                            Ok(card) => Action::Printing {
                                ticket,
                                card: Box::new(card),
                            },
                            Err(ClientError::NotFound(_)) => Action::Missing { ticket },
                            Err(_) => Action::Failed { ticket },
                        };
                        let effects = model.write().update(reply);
                        drop(effects);
                    });
                }
                Effect::Replace(to) => {
                    navigator.replace(route_of(to));
                }
                Effect::Navigate(to) => {
                    navigator.push(route_of(to));
                }
                Effect::RememberReturn(_) => {}
                Effect::OpenSendTo(_) => {
                    toast.info(
                        "Send to opens here".to_string(),
                        components::ToastOptions::default(),
                    );
                }
                Effect::Copy { text, done } => {
                    document::eval(&format!("navigator.clipboard?.writeText({text:?})"));
                    toast.info(done.text, components::ToastOptions::default());
                }
            }
        }
    });

    use_hook({
        let open = open.clone();
        move || {
            dispatch.call(Action::Load {
                open,
                send: false,
                signed_in: signed_in(),
            });
        }
    });

    let query = use_signal(String::new);
    let search = rsx! {
        div {
            class: "cs-search",
            onkeydown: move |evt| {
                if evt.key() == Key::Enter
                    && let Some(slug) = card_slug(&query.peek())
                {
                    navigator.push(Route::CardSpike {
                        slug: slug.to_string(),
                        look: if look_b { "b".to_owned() } else { "a".to_owned() },
                    });
                }
            },
            PickerSearch { id: "card-spike-search", placeholder: "Card name", query }
        }
    };

    let state = model.read();
    let status = state.status();
    let card = state.card().cloned();
    let cards = state.cards().to_vec();
    let title = state.title();
    let message = state.message();
    drop(state);
    let frame_title = card
        .as_ref()
        .map_or_else(|| "Card".to_owned(), |c| c.scryfall_data.name.clone());

    let actions = move || {
        rsx! {
            button {
                class: "card-action-btn",
                onclick: move |_| dispatch.call(Action::SendTo { signed_in: signed_in() }),
                "Send to"
            }
            button { class: "card-action-btn", onclick: move |_| dispatch.call(Action::Share), "Share" }
        }
    };

    let body = match (status, card) {
        (Status::One, Some(card)) => {
            let sd = card.scryfall_data.clone();
            let image = rsx! {
                div { class: "cs-image",
                    FlippableCardImage { sd, size: ImageSize::Large }
                }
            };
            let panel_footer = (!look_b).then(|| rsx! { div { class: "cs-actions", {actions()} } });
            let panel = rsx! {
                CardDetailsPanel {
                    card: card.clone(),
                    send_to: false,
                    on_printings: move |()| printings_open.set(true),
                    footer: panel_footer,
                }
            };
            rsx! {
                if look_b {
                    div { class: "cs-b",
                        {image}
                        {panel}
                        div { class: "cs-bar", {actions()} }
                    }
                } else {
                    div { class: "cs-a",
                        {image}
                        {panel}
                    }
                }
                PrintingSheet { card, open: printings_open }
            }
        }
        (Status::Several, _) => rsx! {
            div { class: "cs-list",
                h2 { class: "panel-title", {title.unwrap_or_default()} }
                for listed in cards {
                    div {
                        key: "{listed.scryfall_data.id}",
                        onclick: {
                            let id = listed.scryfall_data.id;
                            move |_| dispatch.call(Action::Pick(id))
                        },
                        CardRow { card: listed.clone(), qty: 1, expanded_card: expanded }
                    }
                }
            }
        },
        (Status::NotFound, _) => rsx! {
            p { class: "cs-message", {message.unwrap_or(card_lookup::NOT_FOUND)} }
        },
        (Status::Failed, _) => rsx! {
            LoadFailed { message: "Couldn't load this card", on_retry: move |()| dispatch.call(Action::Retry) }
        },
        _ => rsx! {
            components::CardDetailsPanelSkeleton {}
        },
    };

    rsx! {
        ScreenFrame { title: frame_title,
            style { {SPIKE_CSS} }
            {search}
            {body}
        }
    }
}
