use std::time::Duration;

use chrono::{DateTime, Utc};
use components::{Button, ButtonVariant, TOAST_LONG, TOAST_NORMAL};
use dioxus::prelude::*;
use dioxus_primitives::toast::{ToastOptions, use_toast};
use tokio::time;
use zwipe_client::{ClientError, ZwipeClient};
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::components::auth::authed::use_authed;
use crate::inbound::components::auth::ensure_session::EnsureFresh;
use crate::inbound::components::telemetry::usage_buffer::UsageBuffer;
use crate::inbound::components::telemetry::vocabulary::{ProfileScreen, Screen, component, screen};

/// Seconds before another resend is allowed. Mirrors the server's dedicated
/// resend-verification rate-limit window (burst 1, then 1/60s per user) so the
/// visible countdown and the 429 window agree.
const RESEND_COOLDOWN_SECS: u32 = 60;

const SECS_PER_MINUTE: i64 = 60;
const MINUTES_PER_HOUR: i64 = 60;

/// Email display line: address, verified/unverified badge, and the Change
/// button. Verification actions live in [`VerificationActions`], rendered as
/// its own Account row only while the email is unverified.
#[component]
pub(crate) fn EmailVerification(
    email: String,
    is_verified: bool,
    on_change: EventHandler,
) -> Element {
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

/// Pending row value: the address waiting on its confirmation link and, when
/// the server says, how long that link has left. Servers older than
/// `pending_email_expires_at` send only the address.
#[component]
pub(crate) fn PendingEmail(email: String, expires_at: Option<DateTime<Utc>>) -> Element {
    let note = expires_at.map(|at| link_expiry_text(at, Utc::now()));
    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:flex-end;gap:0.25rem;",
            span { "{email}" }
            if let Some(note) = note {
                span { class: "text-muted text-sm", "{note}" }
            }
        }
    }
}

/// What the Pending row says about its link: minutes in the last hour,
/// whole hours before that, and where to get a new one once it's dead.
fn link_expiry_text(expires_at: DateTime<Utc>, now: DateTime<Utc>) -> String {
    let left = expires_at - now;
    if left <= chrono::Duration::zero() {
        return "Link expired; tap Change to send a new one".to_string();
    }
    let minutes = (left.num_seconds() + SECS_PER_MINUTE - 1) / SECS_PER_MINUTE;
    if minutes < MINUTES_PER_HOUR {
        return match minutes {
            1 => "Link expires in 1 minute".to_string(),
            n => format!("Link expires in {n} minutes"),
        };
    }
    match (minutes + MINUTES_PER_HOUR / 2) / MINUTES_PER_HOUR {
        1 => "Link expires in 1 hour".to_string(),
        n => format!("Link expires in {n} hours"),
    }
}

/// Verification-row actions: a resend button that grays out behind a 60s
/// countdown after each send, and a refresh-icon button that re-fetches the
/// user so the badge flips to Verified without leaving the screen.
#[component]
pub(crate) fn VerificationActions() -> Element {
    let mut session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let usage_buffer: Signal<UsageBuffer> = use_context();
    // check_verified goes through the facade; the resend button stays on the
    // raw pattern because it branches on TooManyRequests (info toast, keep
    // the cooldown) and the facade would double-toast that path.
    let authed = use_authed(Screen::Profile(ProfileScreen::Main));
    let mut is_resending = use_signal(|| false);
    let mut is_checking = use_signal(|| false);
    // Seconds left on the resend cooldown; the button re-enables at zero.
    let mut cooldown = use_signal(|| 0u32);

    rsx! {
        Button {
            variant: ButtonVariant::Action,
            disabled: is_resending() || cooldown() > 0,
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                is_resending.set(true);
                // Optimistic: gray out for the full window right away so
                // rapid clicks can't race the request. Cleared below if
                // the send genuinely failed.
                cooldown.set(RESEND_COOLDOWN_SECS);
                spawn(async move {
                    let s = match session.ensure_fresh(client).await {
                        Ok(s) => s,
                        Err(e) => {
                            usage_buffer.peek().report_error(screen::PROFILE, component::EMAIL_VERIFICATION, "resend_verification", &e);
                            toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(TOAST_LONG),
                            );
                            cooldown.set(0);
                            is_resending.set(false);
                            return;
                        }
                    };
                    match client().resend_verification(&s).await {
                        Ok(()) => toast.success(
                            "Verification email sent".to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        ),
                        // Raced the server window: an email already went
                        // out recently, so keep the countdown running.
                        Err(ClientError::TooManyRequests(_)) => toast.info(
                            "Please wait a moment".to_string(),
                            ToastOptions::default().duration(TOAST_NORMAL),
                        ),
                        Err(e) => {
                            usage_buffer.peek().report_error(screen::PROFILE, component::EMAIL_VERIFICATION, "resend_verification", &e);
                            toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(TOAST_LONG),
                            );
                            // The send didn't happen: don't strand the
                            // user behind a timer.
                            cooldown.set(0);
                            is_resending.set(false);
                            return;
                        }
                    }
                    is_resending.set(false);
                    // Drive the visible countdown to zero.
                    while *cooldown.peek() > 0 {
                        time::sleep(Duration::from_secs(1)).await;
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
        Button {
            variant: ButtonVariant::Action,
            class: "btn-action-eye",
            disabled: is_checking(),
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                is_checking.set(true);
                spawn(async move {
                    if let Some(fresh_user) = authed
                        .run_at(component::EMAIL_VERIFICATION, "check_verified", |c, s| async move {
                            c.get_user(&s).await
                        })
                        .await
                    {
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
            },
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

#[cfg(test)]
mod tests {
    use chrono::{Duration, TimeZone};

    use super::*;

    fn at(left: Duration) -> String {
        let now = Utc.with_ymd_and_hms(2026, 10, 23, 12, 0, 0).unwrap();
        link_expiry_text(now + left, now)
    }

    #[test]
    fn fresh_link_reads_in_hours() {
        assert_eq!(at(Duration::hours(24)), "Link expires in 24 hours");
        assert_eq!(
            at(Duration::hours(24) - Duration::seconds(5)),
            "Link expires in 24 hours"
        );
        assert_eq!(at(Duration::minutes(90)), "Link expires in 2 hours");
        assert_eq!(at(Duration::minutes(80)), "Link expires in 1 hour");
        assert_eq!(at(Duration::minutes(60)), "Link expires in 1 hour");
    }

    #[test]
    fn last_hour_reads_in_minutes() {
        assert_eq!(at(Duration::minutes(59)), "Link expires in 59 minutes");
        assert_eq!(at(Duration::seconds(61)), "Link expires in 2 minutes");
        assert_eq!(at(Duration::seconds(1)), "Link expires in 1 minute");
    }

    #[test]
    fn dead_link_points_at_change() {
        let expired = "Link expired; tap Change to send a new one";
        assert_eq!(at(Duration::zero()), expired);
        assert_eq!(at(Duration::hours(-3)), expired);
    }
}
