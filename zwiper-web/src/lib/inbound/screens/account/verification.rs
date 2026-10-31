//! The Account email rows: the address with its badge, the pending address
//! with its link note, and the Verification row's resend and check buttons.

use std::time::Duration;

use chrono::{DateTime, Utc};
use components::{Button, ButtonVariant, TOAST_LONG, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::domain::account::{self, RESEND_COOLDOWN_SECS};
use crate::inbound::chrome::authed::use_authed;
use crate::inbound::session::ensure_fresh::ensure_fresh;
use crate::outbound::timer;

/// The Email row's value: the address, `Verified` or `Unverified`, and
/// `Change`.
#[component]
pub(super) fn EmailValue(email: String, is_verified: bool, on_change: EventHandler) -> Element {
    rsx! {
        span { "{email}" }
        if is_verified {
            span { class: "badge-verified", "Verified" }
        } else {
            span { class: "badge-unverified", "Unverified" }
        }
        Button {
            variant: ButtonVariant::Action,
            onclick: move |_| on_change.call(()),
            "Change"
        }
    }
}

/// The Pending row's value: the address waiting on its link and, when the
/// server sends it, how long the link has left.
#[component]
pub(super) fn PendingEmail(email: String, expires_at: Option<DateTime<Utc>>) -> Element {
    let note = expires_at.map(|at| account::link_expiry_text(at, Utc::now()));
    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:flex-end;gap:0.25rem;",
            span { "{email}" }
            if let Some(note) = note {
                span { class: "text-muted text-sm", "{note}" }
            }
        }
    }
}

/// `Resend`, which goes gray behind a 60 s countdown after each send, and the
/// refresh button, which fetches the user so the badge flips once the email
/// is verified.
#[component]
pub(super) fn VerificationActions() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let authed = use_authed();
    let mut is_resending = use_signal(|| false);
    let mut is_checking = use_signal(|| false);
    let mut cooldown = use_signal(|| 0u32);

    // Resend stays off the facade: a 429 is an info toast that keeps the
    // cooldown, and every other failure is an 8 s toast.
    let resend = move |event: MouseEvent| {
        event.stop_propagation();
        is_resending.set(true);
        // Gray at once, so quick clicks cannot race the request.
        cooldown.set(RESEND_COOLDOWN_SECS);
        spawn(async move {
            let failed = |e: &ClientError| {
                tracing::warn!("resend_verification failed: {e}");
                toast.error(
                    e.to_user_message(),
                    ToastOptions::default().duration(TOAST_LONG),
                );
            };
            let current = match ensure_fresh(session, client).await {
                Ok(current) => current,
                Err(e) => {
                    failed(&e);
                    cooldown.set(0);
                    is_resending.set(false);
                    return;
                }
            };
            let http = client.peek().clone();
            match http.resend_verification(&current).await {
                Ok(()) => toast.success(
                    "Verification email sent".to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                ),
                // One went out recently, so the countdown keeps running.
                Err(ClientError::TooManyRequests(_)) => toast.info(
                    "Please wait a moment".to_string(),
                    ToastOptions::default().duration(TOAST_NORMAL),
                ),
                Err(e) => {
                    failed(&e);
                    cooldown.set(0);
                    is_resending.set(false);
                    return;
                }
            }
            is_resending.set(false);
            while *cooldown.peek() > 0 {
                timer::sleep(Duration::from_secs(1)).await;
                let left = *cooldown.peek();
                cooldown.set(left.saturating_sub(1));
            }
        });
    };

    let check = move |event: MouseEvent| {
        event.stop_propagation();
        is_checking.set(true);
        spawn(async move {
            if let Some(user) = authed
                .run(
                    "check_verified",
                    |c, s| async move { c.fetch_user(&s).await },
                )
                .await
            {
                let verified = user.email_verified_at.is_some();
                let latest = session.peek().clone();
                if let Some(mut latest) = latest {
                    latest.user = user;
                    session.set(Some(latest));
                }
                if verified {
                    toast.success(
                        "Email verified".to_string(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                } else {
                    toast.info(
                        "Not verified yet. Check your inbox".to_string(),
                        ToastOptions::default().duration(TOAST_NORMAL),
                    );
                }
            }
            is_checking.set(false);
        });
    };

    rsx! {
        Button {
            variant: ButtonVariant::Action,
            disabled: is_resending() || cooldown() > 0,
            onclick: resend,
            {account::resend_label(is_resending(), cooldown())}
        }
        span { title: "Check verification", style: "display:inline-flex;",
            Button {
                variant: ButtonVariant::Action,
                class: "btn-action-eye",
                disabled: is_checking(),
                onclick: check,
                svg {
                    class: "refresh-icon",
                    view_box: "0 0 24 24",
                    xmlns: "http://www.w3.org/2000/svg",
                    polyline { points: "23 4 23 10 17 10" }
                    polyline { points: "1 20 1 14 7 14" }
                    path { d: "M3.51 9a9 9 0 0 1 14.85-3.36L23 10M1 14l4.63 4.36A9 9 0 0 0 20.49 15" }
                }
            }
        }
    }
}
