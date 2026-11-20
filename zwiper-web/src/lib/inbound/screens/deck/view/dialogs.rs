//! The deck view's dialogs: Clone, Share with Copy and Stop sharing, Clear
//! skips and Delete. The profile header's icons and the Danger zone panel
//! open them; an action in a dialog closes it, as the app does.

use components::{ActionButton, AlertDialog, DialogAction, ShareUrl, TOAST_NORMAL, TOAST_QUICK};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use uuid::Uuid;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::domain::deck::deck_name;
use zwipe_core::http::contracts::deck::HttpCloneDeck;
use zwipe_ui::screens::deck_view;
use zwipe_ui::telemetry::vocabulary::component;
use zwipe_ui::write_id::WriteId;
use zwipe_ui::write_repeat::WriteKey;

use crate::domain::deck::share_link;
use crate::inbound::chrome::authed::{Authed, use_authed};
use crate::inbound::router::Route;
use crate::inbound::telemetry;
use crate::outbound::browser;

/// Which of the deck's dialogs is up. `Copy`, so the buttons that open
/// them and the dialogs share it.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct DeckDialogs {
    /// The Clone dialog.
    pub clone: Signal<bool>,
    /// The Share dialog.
    pub share: Signal<bool>,
    /// The Clear skips dialog.
    pub clear: Signal<bool>,
    /// The Delete dialog.
    pub delete: Signal<bool>,
}

/// Every dialog closed.
pub fn use_deck_dialogs() -> DeckDialogs {
    DeckDialogs {
        clone: use_signal(|| false),
        share: use_signal(|| false),
        clear: use_signal(|| false),
        delete: use_signal(|| false),
    }
}

/// The dialogs for `deck_id`. `share_token` is the deck's live link, which
/// Share and Stop sharing update in place, and `deck_name` names it in that
/// link after the signed-in user's username. `delete_body` is the Delete
/// dialog's line, naming the deck.
#[component]
pub fn DeckDialogsHost(
    deck_id: Uuid,
    dialogs: DeckDialogs,
    mut share_token: Signal<Option<Uuid>>,
    deck_name: String,
    clone_name: String,
    delete_body: String,
) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let DeckDialogs {
        clone: clone_open,
        share: share_open,
        clear: clear_open,
        delete: delete_open,
    } = dialogs;

    // The link's owner, read when a link is shown or copied, and the deck's
    // name for a copy made from a handler, which runs after the effect has
    // caught the name up. The dialog itself names the link from `deck_name`.
    let session: Signal<Option<Session>> = use_context();
    let mut name = use_signal(|| deck_name.clone());
    use_effect(use_reactive!(|deck_name| name.set(deck_name)));
    let link = move |deck: &str, token: Uuid| {
        let owner = session
            .peek()
            .as_ref()
            .map_or_default(|s| s.user.username.to_string());
        share_link(&owner, deck, token)
    };
    let copy_link = move |token: Uuid| {
        browser::copy_to_clipboard(&link(&name.peek(), token));
        toast.info(
            components::carry_title("Share link copied"),
            ToastOptions::default().duration(TOAST_QUICK),
        );
    };
    // The dialog stays open until the link lands, so it can't be reopened over a link still on
    // its way; a failure keeps it open. A request lost on its way back may still have made the
    // link, so the deck's own link decides.
    let create_link = ActionButton::new("Create link").busy("Creating link…", move || async move {
        let created = authed
            .try_run_quiet("share_deck", |c, s| async move {
                c.share_deck(deck_id, &s).await
            })
            .await
            .map(|shared| shared.share_token);
        let recheck = || async move { fetch_share_state(authed, deck_id).await.link() };
        match after_lost_reply(created, outcome_unknown, recheck).await {
            Ok(token) => {
                share_token.set(Some(token));
                copy_link(token);
                true
            }
            Err(error) => {
                toast.error(
                    components::carry_title(&error.to_user_message()),
                    ToastOptions::default().duration(TOAST_NORMAL),
                );
                false
            }
        }
    });
    let copy_shared = use_callback(move |()| {
        if let Some(token) = *share_token.peek() {
            copy_link(token);
        }
    });
    // A stop whose reply is lost may still have landed, so the deck's own link decides.
    let stop_share = use_callback(move |()| {
        spawn(async move {
            let stopped = authed
                .try_run_quiet("unshare_deck", |c, s| async move {
                    c.unshare_deck(deck_id, &s).await
                })
                .await;
            let recheck = || async move { fetch_share_state(authed, deck_id).await.stopped() };
            match after_lost_reply(stopped, outcome_unknown, recheck).await {
                Ok(()) => {
                    share_token.set(None);
                    toast.info(
                        "Link disabled".to_string(),
                        ToastOptions::default().duration(TOAST_QUICK),
                    );
                }
                Err(error) => toast.error(
                    components::carry_title(&error.to_user_message()),
                    ToastOptions::default().duration(TOAST_NORMAL),
                ),
            }
        });
    });
    // Pending skips go out first, so this window's skips clear too.
    let usage = telemetry::use_usage();
    let client: Signal<ZwipeClient> = use_context();
    let clear_skips = use_callback(move |()| {
        spawn(async move {
            telemetry::flush_once(usage, session, client).await;
            if authed
                .in_component(component::MORE_BUTTONS)
                .run("clear_suppressions", |c, s| async move {
                    c.clear_deck_suppressions(deck_id, &s).await
                })
                .await
                .is_some()
            {
                toast.info(
                    "Skips cleared".to_string(),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
            }
        });
    });
    // Decks replaces the gone deck in history, so Back never reopens it.
    let delete = use_callback(move |()| {
        spawn(async move {
            // A lost earlier try that deleted it reads as deleted.
            if authed
                .run_write(
                    "delete_deck",
                    WriteKey::delete_deck(deck_id),
                    |c, s| async move { c.delete_deck(deck_id, &s).await },
                )
                .await
                .or_landed(|| ())
                .is_some()
            {
                navigator.replace(Route::DeckList {});
            }
        });
    });

    let (share_body, share_actions) = if share_token().is_some() {
        (
            "Anyone with the link can view this deck. Stop sharing disables the link; sharing again creates a new one.",
            components::shared_link_actions(copy_shared, stop_share),
        )
    } else {
        (
            "Anyone with the link can view this deck. You can stop sharing at any time.",
            vec![DialogAction::Dismiss("Close".into()), create_link.into()],
        )
    };

    rsx! {
        AlertDialog {
            open: share_open,
            title: "Share deck",
            actions: share_actions,
            div { "{share_body}" }
            if let Some(token) = share_token() {
                ShareUrl { url: link(&deck_name, token) }
            }
        }
        AlertDialog {
            open: clear_open,
            title: "Clear skips",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Clear".into(),
                    danger: true,
                    on_confirm: clear_skips,
                },
            ],
            "Skipped cards come back on their own after a week, and removed cards after a month. Clearing brings them all back now. This can't be undone."
        }
        AlertDialog {
            open: delete_open,
            title: "Delete deck",
            actions: vec![
                DialogAction::Dismiss("Cancel".into()),
                DialogAction::Confirm {
                    label: "Delete".into(),
                    danger: true,
                    on_confirm: delete,
                },
            ],
            "{delete_body}"
        }
        CloneDeckDialog { source_deck_id: deck_id, default_name: clone_name, open: clone_open }
    }
}

/// Whether a failed call may still have landed: the request went out and its answer never came
/// back, as when Android Chrome's hardware Esc stops every fetch in flight.
fn outcome_unknown(error: &ClientError) -> bool {
    matches!(error, ClientError::Network(_))
}

/// What a write ends with: its own answer, or, after a failure `outcome_unknown` can't rule out
/// landing, what `recheck` reads back from the server: `Some` when the server shows the write
/// landed, `None` when it shows no sign of it or the read fails, which keeps the write's error.
async fn after_lost_reply<T, E, R>(
    done: Result<T, E>,
    outcome_unknown: impl FnOnce(&E) -> bool,
    recheck: impl FnOnce() -> R,
) -> Result<T, E>
where
    R: Future<Output = Option<T>>,
{
    match done {
        Ok(value) => Ok(value),
        Err(error) if outcome_unknown(&error) => recheck().await.ok_or(error),
        Err(error) => Err(error),
    }
}

/// The deck's link as [`fetch_share_state`] reads it back from the server.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
enum ShareRead {
    /// The deck is shared under this token.
    Shared(Uuid),
    /// The deck has no link.
    Unshared,
    /// The read failed, so the server's state is unknown.
    Unread,
}

impl ShareRead {
    /// The link, when the deck has one: a Create link that landed.
    fn link(self) -> Option<Uuid> {
        match self {
            Self::Shared(token) => Some(token),
            Self::Unshared | Self::Unread => None,
        }
    }

    /// Whether a Stop sharing landed: only a deck read back with no link.
    fn stopped(self) -> Option<()> {
        match self {
            Self::Unshared => Some(()),
            Self::Shared(_) | Self::Unread => None,
        }
    }
}

/// The deck's link as the server holds it, read back quietly.
async fn fetch_share_state(authed: Authed, deck_id: Uuid) -> ShareRead {
    let read = authed
        .run_quiet("recheck_share", |c, s| async move {
            c.fetch_deck_profile(deck_id, &s).await
        })
        .await;
    match read {
        Some(profile) => profile
            .share_token
            .map_or(ShareRead::Unshared, ShareRead::Shared),
        None => ShareRead::Unread,
    }
}

/// Clone deck: a copy of the deck under a new name, prefilled with
/// `{name} (clone)` on every open. The name is judged like Create's, with
/// its line under the field, and `Save` stays off until it passes; a failure
/// keeps the dialog open.
#[component]
fn CloneDeckDialog(source_deck_id: Uuid, default_name: String, open: Signal<bool>) -> Element {
    let navigator = use_navigator();
    let authed = use_authed();
    let toast = use_toast();
    let mut name = use_signal(String::new);
    // The copy's id Save sends, kept while the same name is saved again after a failure.
    let mut clone_id = use_signal(WriteId::<String>::new);
    use_effect(move || {
        // Opening and closing both start the form over.
        let _open = open();
        name.set(default_name.clone());
    });
    let blank = use_memo(move || !deck_view::can_clone(&name()));
    let save = ActionButton::new("Save")
        .disabled(blank)
        .busy("Cloning…", move || {
            let new_name = name.peek().trim().to_owned();
            let id = clone_id.write().id_for(new_name.clone(), Uuid::new_v4);
            async move {
                let body = HttpCloneDeck {
                    new_name: new_name.clone(),
                    id: Some(id),
                };
                let Some(cloned) = authed
                    .run("clone_deck", |c, s| async move {
                        c.clone_deck(source_deck_id, &body, &s).await
                    })
                    .await
                else {
                    return false;
                };
                clone_id.write().landed();
                toast.info(
                    components::carry_title(&format!("Cloned as \"{new_name}\"")),
                    ToastOptions::default().duration(TOAST_QUICK),
                );
                navigator.push(Route::ViewDeck {
                    view: None,
                    deck_id: cloned.deck_id,
                });
                true
            }
        });
    rsx! {
        AlertDialog {
            open,
            title: "Clone deck",
            actions: vec![DialogAction::Dismiss("Cancel".into()), save.into()],
            "Make an exact copy of your deck under a new name."
            CloneNameInput { name }
        }
    }
}

/// The new name field and its line, its own component so it reads the name
/// where the dialog host draws it.
#[component]
fn CloneNameInput(mut name: Signal<String>) -> Element {
    let error = deck_view::clone_name_error(&name());
    let error_shown = error.is_some();
    rsx! {
        input {
            r#type: "text",
            class: if error_shown { "input input-error" } else { "input" },
            placeholder: "New deck name",
            maxlength: deck_name::MAX_LENGTH.to_string(),
            value: "{name}",
            autocapitalize: "none",
            autocorrect: "off",
            spellcheck: "false",
            oninput: move |evt| name.set(evt.value()),
        }
        if let Some(error) = error {
            div { class: "message-error", "{error}" }
        }
    }
}

#[cfg(test)]
mod tests {
    use std::cell::Cell;

    use super::*;

    const NEW: Uuid = Uuid::from_u128(7);

    /// A failure whose outcome is unknown, as a request lost on its way back.
    fn lost(error: &&str) -> bool {
        *error == "lost"
    }

    #[tokio::test]
    async fn a_write_that_answered_is_its_answer_and_nothing_is_rechecked() {
        let rechecked = Cell::new(false);
        let link = after_lost_reply(Ok::<_, &str>(NEW), lost, || async {
            rechecked.set(true);
            None
        })
        .await;
        assert_eq!(link, Ok(NEW));
        assert!(!rechecked.get());
    }

    #[tokio::test]
    async fn a_link_made_before_its_reply_was_lost_is_the_link() {
        let link = after_lost_reply(Err("lost"), lost, || async { Some(NEW) }).await;
        assert_eq!(link, Ok(NEW), "the deck is shared, so Share shows it");
    }

    #[tokio::test]
    async fn a_lost_write_the_server_shows_no_sign_of_stays_the_error() {
        let unshared = after_lost_reply(Err::<Uuid, _>("lost"), lost, || async { None }).await;
        assert_eq!(unshared, Err("lost"), "no link, or a recheck that failed");
    }

    #[tokio::test]
    async fn a_stop_made_before_its_reply_was_lost_is_done() {
        let stopped = after_lost_reply(Err("lost"), lost, || async { Some(()) }).await;
        assert_eq!(
            stopped,
            Ok(()),
            "the deck has no link, so Share shows it unshared"
        );
    }

    #[tokio::test]
    async fn a_refusal_is_final_and_nothing_is_rechecked() {
        let rechecked = Cell::new(false);
        let link = after_lost_reply(Err("refused"), lost, || async {
            rechecked.set(true);
            Some(NEW)
        })
        .await;
        assert_eq!(link, Err("refused"));
        assert!(!rechecked.get());
    }

    #[test]
    fn the_recheck_reads_a_link_as_landed_only_from_a_shared_deck() {
        assert_eq!(ShareRead::Shared(NEW).link(), Some(NEW));
        assert_eq!(ShareRead::Unshared.link(), None);
        assert_eq!(ShareRead::Unread.link(), None, "the read failed");
    }

    #[test]
    fn the_recheck_reads_a_stop_as_landed_only_with_no_link() {
        assert_eq!(ShareRead::Unshared.stopped(), Some(()));
        assert_eq!(ShareRead::Shared(NEW).stopped(), None, "still shared");
        assert_eq!(ShareRead::Unread.stopped(), None, "the read failed");
    }
}
