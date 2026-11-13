//! The deck view. On a wide window it is the deck page both sites share
//! (`page`): its facts and featured cards, the cards as
//! bands, the stats folded under them, and Warnings, Data and Danger zone
//! at the foot. Below the breakpoint it is the app's: the Profile card, the
//! accordion with `Budget` open, Data and Danger zone, with Cards in the dock
//! opening the deck's cards as a layer over it (`?view=cards`). A deck the server no longer has leaves for Decks,
//! as the app's model does with `Action::Gone`.

pub mod body;
pub(crate) mod cards_layer;
pub mod dialogs;
pub(crate) mod ghost;
pub mod page;
pub mod panels;
pub(crate) mod profile;
pub mod sections;

use std::iter;

use components::icons::{CloneIcon, EditIcon, ShareIcon};
use components::{LoadFailed, TOAST_NORMAL, TOAST_QUICK, use_back_online};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::card::Card;
use zwipe_core::domain::card::search_card::card_filter::price_currency::PriceCurrency;
use zwipe_core::domain::deck::{DeckEntry, DeckProfile};
use zwipe_core::http::contracts::deck_card::HttpPatchDeckCard;
use zwipe_ui::deck::form::Slot;
use zwipe_ui::deck::view::DeckView;
use zwipe_ui::screens::deck_view::{EDIT, LOAD_FAILED, MoreItem, delete_body};
use zwipe_ui::stores::undo::UndoAction;
use zwipe_ui::telemetry::vocabulary::{DeckScreen, Screen, component};

use self::cards_layer::use_cards_route;
use self::dialogs::{DeckDialogsHost, use_deck_dialogs};
use self::page::DeckWidePage;
use self::sections::WarningFixes;
use crate::inbound::card_stores::CardStores;
use crate::inbound::cards::card_row::{CardOverlaysHost, use_card_overlays};
use crate::inbound::cards::otag::{OtagExamplesHost, use_otag_reveals};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::chrome::one_time_hint::{FirstDeckHintDialog, OneTimeHint, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;
use crate::inbound::telemetry;

/// The screen's title, in the tab and the folded nav.
const TITLE: &str = "Deck";

/// The deck view for `deck_id`, mounted afresh for each deck, so a deck that
/// opens another (a clone) or the browser's back and forward between two
/// decks never shows one deck's state under the other's address.
#[component]
pub fn ViewDeck(deck_id: Uuid, view: Option<DeckView>) -> Element {
    // A key takes effect only among a list's items, so the body renders as a
    // list of one.
    rsx! {
        for deck_id in iter::once(deck_id) {
            DeckViewScreen { key: "{deck_id}", deck_id, view }
        }
    }
}

/// The deck view's body for one deck.
#[component]
fn DeckViewScreen(deck_id: Uuid, view: Option<DeckView>) -> Element {
    telemetry::use_telemetry_screen(Screen::Deck(DeckScreen::View));
    let navigator = use_navigator();
    let on_cards = use_cards_route(move |view| Route::ViewDeck { deck_id, view });
    let wide = use_wide();
    let client: Signal<ZwipeClient> = use_context();
    let session: Signal<Option<Session>> = use_context();
    let authed = use_authed();
    let toast = use_toast();
    let usage = telemetry::use_usage();
    let hint_open = use_one_time_hint(OneTimeHint::FirstDeck);
    let on_hint = use_callback(move |()| {
        let mut hint_open = hint_open;
        hint_open.set(true);
    });
    let dialogs = use_deck_dialogs();
    let mut clone_open = dialogs.clone;
    let mut share_open = dialogs.share;
    let examples = use_otag_reveals();
    let overlays = use_card_overlays();
    let mut currency = use_signal(|| PriceCurrency::Usd);
    let mut share_token: Signal<Option<Uuid>> = use_signal(|| None);
    let mut seeded_for: Signal<Option<Uuid>> = use_signal(|| None);
    let mut commander: Signal<Option<Card>> = use_signal(|| None);
    // Counts the warning fixes that have landed, so the wide page reloads
    // its own copy of the deck after each.
    let mut fixed = use_signal(|| 0u32);

    let mut profile: Resource<Result<DeckProfile, ClientError>> =
        use_resource(use_reactive!(|deck_id| async move {
            authed
                .try_run("load_deck", |c, s| async move {
                    c.fetch_deck_profile(deck_id, &s).await
                })
                .await
        }));
    let CardStores { undo, .. } = use_context();
    // The deck's rows as the page's editor holds them, its edits since the fetch included,
    // which the page keeps current: the fixes read a card's row from it before they change it.
    let rows: Signal<Vec<DeckEntry>> = use_signal(Vec::new);

    // Tries a failed load again, from Retry or the connection coming back.
    let retry = use_callback(move |()| {
        if matches!(&*profile.peek(), Some(Err(_))) {
            profile.restart();
        }
    });
    use_back_online(retry);

    // A deck the server no longer has leaves for Decks, replacing this
    // screen in history so Back never reopens it; the fetch's toast has
    // already said why.
    use_effect(move || {
        let gone = matches!(&*profile.read(), Some(Err(ClientError::NotFound(_))));
        if gone {
            navigator.replace(Route::DeckList {});
        }
    });

    // Once per deck: the currency chips start on the target's currency, the
    // share link is the saved one, and the commander card is fetched for
    // its name.
    use_effect(move || {
        let Some(Ok(loaded)) = &*profile.read() else {
            return;
        };
        if *seeded_for.peek() == Some(loaded.id) {
            share_token.set(loaded.share_token);
            return;
        }
        seeded_for.set(Some(loaded.id));
        share_token.set(loaded.share_token);
        if let Some(stored) = loaded.price_target_currency {
            currency.set(stored);
        }
        commander.set(None);
        if let Some(id) = loaded.commander_id {
            spawn(async move {
                let http = client.peek().clone();
                match http.fetch_card(id).await {
                    Ok(card) => commander.set(Some(card)),
                    Err(e) => {
                        tracing::warn!("load_commander failed: {e}");
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                }
            });
        }
    });

    let fixes = WarningFixes {
        fix_quantity: use_callback(move |(card, quantity): (Uuid, i32)| {
            let request = HttpPatchDeckCard::new(Some(quantity), None);
            // What the fix changes, for the deck's undo log, which Cards' Undo walks.
            let before = rows
                .peek()
                .iter()
                .find(|e| e.deck_card.scryfall_data_id == card)
                .map(|e| (e.card.scryfall_data.name.clone(), *e.deck_card.quantity));
            spawn(async move {
                if authed
                    .run("update_card_quantity", |c, s| async move {
                        c.update_deck_card(deck_id, card, &request, &s).await
                    })
                    .await
                    .is_some()
                {
                    if let Some((card_name, baseline)) = before {
                        let mut undo = undo;
                        undo.write().push(
                            deck_id,
                            UndoAction::QuantityChanged {
                                card_id: card,
                                card_name,
                                net: quantity.saturating_sub(baseline),
                                baseline,
                            },
                        );
                    }
                    toast.info(
                        format!("Quantity set to {quantity}"),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                    // The reload can take a warning away, so the page settles around it.
                    components::settle::watch();
                    fixed.with_mut(|n| *n = n.wrapping_add(1));
                }
            });
        }),
        clear_commander: use_callback(move |()| {
            let request = Slot::Commander.update_request(None);
            spawn(async move {
                let cleared = authed
                    .run("remove_commander", |c, s| async move {
                        c.update_deck_profile(deck_id, &request, &s).await
                    })
                    .await;
                let Some(cleared) = cleared else {
                    return;
                };
                let label = Slot::Commander.label(cleared.format);
                toast.info(
                    format!("{label} cleared"),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                commander.set(None);
                components::settle::watch();
                profile.restart();
                fixed.with_mut(|n| *n = n.wrapping_add(1));
            });
        }),
        // As in the app, straight to the client: a missing session reads
        // `Session expired`.
        remove: use_callback(move |card: Uuid| {
            // The row as it stands, for the deck's undo log, which Cards' Undo walks.
            let undone = UndoAction::removal(&rows.peek(), card);
            spawn(async move {
                let current = session.peek().clone();
                let result = match current {
                    Some(current) => {
                        let http = client.peek().clone();
                        http.delete_deck_card(deck_id, card, &current).await
                    }
                    None => Err(ClientError::Unauthorized("Session expired".to_owned())),
                };
                match result {
                    Ok(()) => {
                        if let Some(undone) = undone {
                            let mut undo = undo;
                            undo.write().push(deck_id, undone);
                        }
                        toast.info(
                            "Card removed".to_string(),
                            ToastOptions::default().duration(TOAST_QUICK),
                        );
                        components::settle::watch();
                        fixed.with_mut(|n| *n = n.wrapping_add(1));
                    }
                    Err(e) => {
                        tracing::warn!("remove_flagged_card failed: {e}");
                        usage.peek().report_error(
                            telemetry::screen_name(),
                            component::DECK_WARNINGS,
                            "remove_flagged_card",
                            &telemetry::failure(&e),
                        );
                        toast.error(
                            e.to_user_message(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        );
                    }
                }
            });
        }),
    };

    let loaded_profile: Option<Result<DeckProfile, ()>> = profile
        .read()
        .as_ref()
        .map(|r| r.as_ref().cloned().map_err(|_| ()));
    let clone_name = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map_or_default(|p| format!("{} (clone)", p.name));
    let deck_name = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map_or_default(|p| p.name.to_string());
    let delete_body = delete_body(
        loaded_profile
            .as_ref()
            .and_then(|r| r.as_ref().ok())
            .map(|p| &*p.name),
    );

    // The deck's name names the page in the tab and the folded nav, once
    // the profile has loaded.
    let deck_title = loaded_profile
        .as_ref()
        .and_then(|r| r.as_ref().ok())
        .map_or_else(|| TITLE.to_owned(), |p| p.name.to_string());

    rsx! {
        ScreenFrame {
            title: deck_title,
            on_hint,
            // The deck's head names it in its own h1 on a wide window and atop the Cards.
            titled: (wide() || view.is_some()) && matches!(loaded_profile, Some(Ok(_))),
            div { class: "screen-content",
                match loaded_profile {
                    Some(Err(())) => rsx! {
                        LoadFailed { message: LOAD_FAILED, on_retry: retry }
                    },
                    _ => rsx! {
                        DeckWidePage {
                            deck_id,
                            currency,
                            fixes,
                            fixed,
                            rows,
                            dialogs,
                            cards_open: view.is_some(),
                            on_cards,
                            // The deck's own actions, before the cart and Cards.
                            profile_dock: rsx! {
                                button {
                                    class: "action-dock-btn",
                                    "aria-label": EDIT,
                                    onclick: move |_| {
                                        navigator.push(Route::EditDeck { deck_id });
                                    },
                                    EditIcon {}
                                }
                                button {
                                    class: "action-dock-btn",
                                    "aria-label": MoreItem::CloneDeck.short_label(),
                                    onclick: move |_| clone_open.set(true),
                                    CloneIcon {}
                                }
                                button {
                                    class: "action-dock-btn",
                                    "aria-label": MoreItem::ShareDeck.short_label(),
                                    onclick: move |_| share_open.set(true),
                                    ShareIcon {}
                                }
                            },
                        }
                    },
                }
            }
        }
        DeckDialogsHost { deck_id, dialogs, share_token, deck_name, clone_name, delete_body }
        FirstDeckHintDialog { open: hint_open, wide: wide() }
        CardOverlaysHost { overlays }
        OtagExamplesHost { examples }
    }
}

#[cfg(test)]
mod tests {
    #[test]
    fn the_deck_s_page_fetches_the_deck_once_through_its_editor() {
        let view = include_str!("mod.rs");
        let page = include_str!("page.rs");
        let editor = include_str!("../cards/editor.rs");
        let fetches = |source: &str| source.matches(".fetch_deck(").count();
        assert_eq!(
            fetches(view),
            1,
            "only this test names the fetch in the view"
        );
        assert_eq!(fetches(page), 0);
        assert_eq!(
            fetches(editor),
            1,
            "the editor's load is the page's one fetch"
        );
    }
}
