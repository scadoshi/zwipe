use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::outbound::client::{
    auth::resend_verification::ClientResendEmailVerification, user::get_user::ClientGetUser,
    ZwipeClient,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use tokio::time::sleep;
use zwipe::inbound::http::ApiError;
use zwipe_core::domain::auth::models::session::Session;

/// Seconds before another resend is allowed. Mirrors the server's dedicated
/// resend-verification rate-limit window (burst 1, then 1/60s per user) so the
/// visible countdown and the 429 window agree.
const RESEND_COOLDOWN_SECS: u32 = 60;

/// Email display row with verified/unverified badge, a resend verification
/// button that greys out behind a 60s countdown after each send, and a
/// "Check again" button that re-fetches the user so the badge flips to
/// Verified without leaving the screen.
#[component]
pub(crate) fn EmailVerification(email: String, is_verified: bool) -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let mut is_resending = use_signal(|| false);
    let mut is_checking = use_signal(|| false);
    // Seconds left on the resend cooldown; the button re-enables at zero.
    let mut cooldown = use_signal(|| 0u32);

    rsx! {
        span { "{email}" }
        if is_verified {
            span { class: "badge-verified", "Verified" }
        } else {
            span { class: "badge-unverified", "Unverified" }
            button {
                class: "util-btn",
                disabled: is_resending() || cooldown() > 0,
                onclick: move |evt| {
                    evt.stop_propagation();
                    is_resending.set(true);
                    // Optimistic: grey out for the full window right away so
                    // rapid clicks can't race the request. Cleared below if
                    // the send genuinely failed.
                    cooldown.set(RESEND_COOLDOWN_SECS);
                    spawn(async move {
                        let s = match session.ensure_fresh(client).await {
                            Ok(s) => s,
                            Err(e) => {
                                toast.error(
                                    e.to_user_message(),
                                    ToastOptions::default().duration(Duration::from_millis(5000)),
                                );
                                cooldown.set(0);
                                is_resending.set(false);
                                return;
                            }
                        };
                        match client().resend_verification(&s).await {
                            Ok(()) => toast.success(
                                "Verification email sent".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(3000)),
                            ),
                            // Raced the server window — an email already went
                            // out recently, so keep the countdown running.
                            Err(ApiError::TooManyRequests(_)) => toast.info(
                                "Please wait a moment".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(3000)),
                            ),
                            Err(e) => {
                                toast.error(
                                    e.to_user_message(),
                                    ToastOptions::default().duration(Duration::from_millis(5000)),
                                );
                                // The send didn't happen — don't strand the
                                // user behind a timer.
                                cooldown.set(0);
                                is_resending.set(false);
                                return;
                            }
                        }
                        is_resending.set(false);
                        // Drive the visible countdown to zero.
                        while *cooldown.peek() > 0 {
                            sleep(Duration::from_secs(1)).await;
                            let remaining = *cooldown.peek();
                            cooldown.set(remaining.saturating_sub(1));
                        }
                    });
                },
                if is_resending() {
                    "Sending..."
                } else if cooldown() > 0 {
                    "Resend in {cooldown()}s"
                } else {
                    "Resend"
                }
            }
            button {
                class: "util-btn",
                disabled: is_checking(),
                onclick: move |evt| {
                    evt.stop_propagation();
                    is_checking.set(true);
                    spawn(async move {
                        let s = match session.ensure_fresh(client).await {
                            Ok(s) => s,
                            Err(e) => {
                                toast.error(
                                    e.to_user_message(),
                                    ToastOptions::default().duration(Duration::from_millis(5000)),
                                );
                                is_checking.set(false);
                                return;
                            }
                        };
                        match client().get_user(&s).await {
                            Ok(fresh_user) => {
                                let verified = fresh_user.email_verified_at.is_some();
                                // Write the fresh user back into the session so
                                // the badge (and anything else) updates in place.
                                let current = session.peek().clone();
                                if let Some(mut current) = current {
                                    current.user = fresh_user;
                                    session.set(Some(current));
                                }
                                if verified {
                                    toast.success(
                                        "Email verified".to_string(),
                                        ToastOptions::default()
                                            .duration(Duration::from_millis(3000)),
                                    );
                                } else {
                                    toast.info(
                                        "Not verified yet — check your inbox".to_string(),
                                        ToastOptions::default()
                                            .duration(Duration::from_millis(3000)),
                                    );
                                }
                            }
                            Err(e) => toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(Duration::from_millis(5000)),
                            ),
                        }
                        is_checking.set(false);
                    });
                },
                if is_checking() { "Checking..." } else { "Check again" }
            }
        }
    }
}
