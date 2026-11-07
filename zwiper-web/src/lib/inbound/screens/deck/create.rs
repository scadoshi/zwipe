//! Create deck: the deck form, with Save as the dock's checkmark once the
//! name passes. A commander handed over by Playstyle or the commander
//! maybeboard arrives with the format set to Commander.

use components::icons::{CHECK, DockIcon};
use dioxus::prelude::*;
use zwipe_core::domain::deck::{DeckName, Format};

use super::form::fields::DeckFields;
use super::form::overlays::DeckFormOverlays;
use super::form::{DECK_LEAVE, use_deck_form};
use crate::domain;
use crate::domain::deck::FormStart;
use crate::inbound::caches::CreateDeckSeed;
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::chrome::leave_guard::use_leave_guard;
use crate::inbound::chrome::one_time_hint::{OneTimeHint, OneTimeHintDialog, use_one_time_hint};
use crate::inbound::router::Route;
use crate::inbound::shell::ScreenFrame;

/// The Create deck screen.
#[component]
pub fn CreateDeck() -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let hint_open = use_one_time_hint(OneTimeHint::CreateDeck);
    let form = use_deck_form();
    let mut saving = use_signal(|| false);

    // The handed-over commander, taken once. The format is set directly so
    // the picker's cascade cannot clear the commander.
    let seed: CreateDeckSeed = use_context();
    use_hook(move || {
        let mut slot = seed.0;
        let taken = slot.write().take();
        if let Some(card) = taken {
            let mut format = form.format;
            format.set(Some(Format::Commander));
            form.commander.pick(card);
        }
    });

    // Leaving asks first once the form differs from how it opened.
    let opened = use_hook(move || form.values());
    let changed = use_memo(move || {
        domain::deck::has_unsaved_changes(FormStart::Opened(&opened), &form.values())
    });
    let guard = use_leave_guard(changed, DECK_LEAVE);

    let submit = use_callback(move |(): ()| {
        if *saving.peek() {
            return;
        }
        let values = form.values();
        if let Err(e) = DeckName::new(values.name.clone()) {
            let mut error = form.name_error;
            error.set(Some(e.to_string()));
            return;
        }
        saving.set(true);
        let request = domain::deck::create_request(&values);
        spawn(async move {
            match authed
                .try_run("create_deck", |c, s| async move {
                    c.create_deck_profile(&request, &s).await
                })
                .await
            {
                Ok(created) => {
                    guard.release();
                    navigator.push(Route::ViewDeck {
                        deck_id: created.id,
                    });
                }
                Err(_) => saving.set(false),
            }
        });
    });

    let valid = DeckName::new((form.name)()).is_ok();
    rsx! {
        ScreenFrame {
            title: "Create deck",
            on_hint: move |()| {
                let mut hint_open = hint_open;
                hint_open.set(true);
            },
            div { class: "screen-content content-enter",
                DeckFields { form, on_enter: move |()| submit.call(()) }
            }
            // Save as the action dock's checkmark, pinned bottom right, once
            // the form is valid; dimmed while the save is out.
            if valid {
                div { class: "action-dock action-dock-right",
                    button {
                        class: "action-dock-btn",
                        "aria-label": if saving() { "Saving..." } else { "Create" },
                        disabled: saving(),
                        onclick: move |_| submit.call(()),
                        DockIcon { path: CHECK }
                    }
                }
            }
        }
        DeckFormOverlays { form }
        OneTimeHintDialog { hint: OneTimeHint::CreateDeck, open: hint_open }
    }
}
