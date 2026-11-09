//! The Account email row: the address, its verification badge and the resend actions.

use components::{Button, ButtonVariant};
use dioxus::prelude::*;
use zwipe_ui::screens::account::{self as model, Action};

use crate::inbound::screens::account::driver::Driver;

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
            span { class: "badge-verified", {model::VERIFIED} }
        } else {
            span { class: "badge-unverified", {model::UNVERIFIED} }
        }
        Button {
            variant: ButtonVariant::Action,
            onclick: move |_| on_change.call(()),
            {model::CHANGE}
        }
    }
}

/// Pending row value: the address waiting on its confirmation link and, when
/// the server says, how long that link has left. Servers older than
/// `pending_email_expires_at` send only the address.
#[component]
pub(crate) fn PendingEmail(email: String, note: Option<String>) -> Element {
    rsx! {
        div { style: "display:flex;flex-direction:column;align-items:flex-end;gap:0.25rem;",
            span { "{email}" }
            if let Some(note) = note {
                span { class: "badge-expiry", "{note}" }
            }
        }
    }
}

/// Verification-row actions: a resend button that grays out behind a 60s
/// countdown after each send, and a refresh-icon button that re-fetches the
/// user so the badge flips to Verified without leaving the screen.
#[component]
pub(crate) fn VerificationActions() -> Element {
    let driver: Driver = use_context();
    let state = driver.model.read();
    let can_resend = state.can_resend();
    let resend_label = state.resend_label();
    let is_checking = state.is_checking();
    drop(state);

    rsx! {
        Button {
            variant: ButtonVariant::Action,
            disabled: !can_resend,
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                driver.dispatch(Action::Resend);
            },
            "{resend_label}"
        }
        Button {
            variant: ButtonVariant::Action,
            class: "btn-action-eye",
            disabled: is_checking,
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                driver.dispatch(Action::CheckVerification);
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

/// Pending-row actions: a resend button that mails a fresh link for the
/// pending change and grays out behind its own 60s countdown, and a
/// refresh-icon button that re-fetches the user and says whether the change
/// went through.
#[component]
pub(crate) fn PendingActions() -> Element {
    let driver: Driver = use_context();
    let state = driver.model.read();
    let can_resend = state.can_resend_change();
    let resend_label = state.resend_change_label();
    let is_checking = state.is_checking_pending();
    drop(state);

    rsx! {
        Button {
            variant: ButtonVariant::Action,
            disabled: !can_resend,
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                driver.dispatch(Action::ResendChange);
            },
            "{resend_label}"
        }
        Button {
            variant: ButtonVariant::Action,
            class: "btn-action-eye",
            disabled: is_checking,
            onclick: move |evt: MouseEvent| {
                evt.stop_propagation();
                driver.dispatch(Action::CheckPending);
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
