//! The delete account dialog: a countdown, the password, then the delete.

use std::time::Duration;

use components::{ActionButton, AlertDialog, DialogAction, TextInput};
use dioxus::core::Task;
use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::secret::Secret;
use zwipe_core::domain::auth::models::session::Session;
use zwipe_core::http::contracts::auth::HttpDeleteUser;

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::auth::signal_logout::SignalLogout;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen, component};

/// Seconds the Delete button stays disabled after the dialog opens.
const DELETE_COUNTDOWN_SECS: u8 = 5;

/// Counts down from `from` to 0, one step per `tick`, each time `open` turns
/// true. Closing or reopening cancels the running count, so an older loop
/// can't reach 0 early.
fn use_open_countdown(open: Signal<bool>, from: u8, tick: Duration) -> Signal<u8> {
    let mut remaining = use_signal(|| from);
    let mut running: Signal<Option<Task>> = use_signal(|| None);
    use_effect(move || {
        let is_open = open();
        if let Some(task) = running.write().take() {
            task.cancel();
        }
        if is_open {
            remaining.set(from);
            let task = spawn(async move {
                for i in (0..from).rev() {
                    tokio::time::sleep(tick).await;
                    remaining.set(i);
                }
            });
            running.set(Some(task));
        }
    });
    remaining
}

/// Delete account dialog with 5-second countdown, password confirmation, and deletion logic.
#[component]
pub(crate) fn DeleteAccountDialog(open: Signal<bool>) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let authed = use_authed(Screen::Profile(ProfileScreen::Main));

    let delete_countdown = use_open_countdown(open, DELETE_COUNTDOWN_SECS, Duration::from_secs(1));
    let mut delete_password = use_signal(String::new);

    // Clear the password each time the dialog opens.
    use_effect(move || {
        if open() {
            delete_password.set(String::new());
        }
    });

    let delete_label = use_memo(move || match delete_countdown() {
        0 => "Delete".to_string(),
        n => format!("Delete ({n})"),
    });
    let counting = use_memo(move || delete_countdown() > 0);

    let delete = ActionButton::new("Delete")
        .danger()
        .live_label(delete_label)
        .disabled(counting)
        .busy("Deleting...", move || {
            let password = delete_password();
            async move {
                let deleted = authed
                    .run_at(
                        component::DELETE_ACCOUNT_DIALOG,
                        "delete_account",
                        |c, s| async move {
                            c.delete_user(
                                HttpDeleteUser {
                                    password: Secret::new(password),
                                },
                                &s,
                            )
                            .await
                        },
                    )
                    .await
                    .is_some();
                if deleted {
                    session.logout(client);
                }
                deleted
            }
        });

    rsx! {
        AlertDialog {
            open,
            title: "Delete account",
            actions: vec![DialogAction::Dismiss("Cancel".into()), delete.into()],
            "This will permanently delete your account, all decks, and all card data. This cannot be undone."
            TextInput {
                value: delete_password,
                input_type: "password".to_string(),
                placeholder: "Password".to_string(),
            }
        }
    }
}

#[cfg(test)]
#[expect(
    clippy::arithmetic_side_effects,
    reason = "test fixtures; overflow fails the test"
)]
mod tests {
    use std::cell::RefCell;
    use std::time::Instant;

    use super::*;

    /// Short enough to keep the test quick, long enough that scheduling
    /// jitter can't reorder the ticks.
    const TICK: Duration = Duration::from_millis(40);
    const FROM: u8 = 5;

    thread_local! {
        static OPEN: RefCell<Option<Signal<bool>>> = const { RefCell::new(None) };
        static SEEN: RefCell<Vec<(Instant, u8)>> = const { RefCell::new(Vec::new()) };
    }

    #[component]
    fn Harness() -> Element {
        let open = use_signal(|| true);
        let remaining = use_open_countdown(open, FROM, TICK);
        OPEN.with(|o| *o.borrow_mut() = Some(open));
        use_effect(move || {
            let value = remaining();
            SEEN.with(|s| s.borrow_mut().push((Instant::now(), value)));
        });
        rsx! {}
    }

    /// Runs the dom's tasks and effects for `span` of real time.
    async fn run_for(dom: &mut VirtualDom, span: Duration) {
        let deadline = tokio::time::Instant::now() + span;
        while tokio::time::timeout_at(deadline, dom.wait_for_work())
            .await
            .is_ok()
        {
            dom.render_immediate_to_vec();
        }
        dom.render_immediate_to_vec();
    }

    fn set_open(dom: &VirtualDom, value: bool) {
        let mut open = OPEN.with(|o| o.borrow().expect("the harness rendered"));
        dom.in_runtime(|| open.set(value));
    }

    fn first_zero_after(start: Instant) -> Option<Duration> {
        SEEN.with(|s| {
            s.borrow()
                .iter()
                .find(|(at, value)| *at >= start && *value == 0)
                .map(|(at, _)| at.duration_since(start))
        })
    }

    #[tokio::test]
    async fn reopening_restarts_the_count_without_the_old_loop() {
        let mut dom = VirtualDom::new(Harness);
        dom.rebuild_in_place();
        run_for(&mut dom, TICK * 5 / 2).await;

        set_open(&dom, false);
        run_for(&mut dom, Duration::from_millis(1)).await;
        let reopened = Instant::now();
        set_open(&dom, true);
        run_for(&mut dom, TICK * 7).await;

        let zero = first_zero_after(reopened).expect("the count reaches zero");
        assert!(
            zero >= TICK * 9 / 2,
            "Delete enabled {zero:?} after reopening, before the full {FROM}-step count"
        );
    }

    #[tokio::test]
    async fn closing_stops_the_count() {
        let mut dom = VirtualDom::new(Harness);
        dom.rebuild_in_place();
        run_for(&mut dom, TICK * 3 / 2).await;

        set_open(&dom, false);
        let closed = Instant::now();
        run_for(&mut dom, TICK * 6).await;

        let moved = SEEN.with(|s| s.borrow().iter().any(|(at, _)| *at > closed));
        assert!(!moved, "the count kept running after the dialog closed");
    }
}
