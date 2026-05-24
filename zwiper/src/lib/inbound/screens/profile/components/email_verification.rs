use crate::inbound::components::auth::session_upkeep::Upkeep;
use crate::outbound::client::{
    auth::resend_verification::ClientResendEmailVerification, ZwipeClient,
};
use dioxus::prelude::*;
use dioxus_primitives::toast::{use_toast, ToastOptions};
use std::time::Duration;
use zwipe_core::domain::auth::models::session::Session;

/// Email display row with verified/unverified badge and resend verification button.
#[component]
pub(crate) fn EmailVerification(email: String, is_verified: bool) -> Element {
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let toast = use_toast();
    let mut is_resending = use_signal(|| false);

    rsx! {
        span { "{email}" }
        if is_verified {
            span { class: "badge-verified", "Verified" }
        } else {
            span { class: "badge-unverified", "Unverified" }
            button {
                class: "util-btn",
                disabled: is_resending(),
                onclick: move |evt| {
                    evt.stop_propagation();
                    is_resending.set(true);
                    spawn(async move {
                        session.upkeep(client);
                        let Some(s) = session() else {
                            toast.error(
                                "Session expired — please log in again".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(5000)),
                            );
                            is_resending.set(false);
                            return;
                        };
                        match client().resend_verification(&s).await {
                            Ok(()) => toast.success(
                                "Verification email sent".to_string(),
                                ToastOptions::default().duration(Duration::from_millis(3000)),
                            ),
                            Err(e) => toast.error(
                                e.to_user_message(),
                                ToastOptions::default().duration(Duration::from_millis(5000)),
                            ),
                        }
                        is_resending.set(false);
                    });
                },
                if is_resending() { "Sending..." } else { "Resend" }
            }
        }
    }
}
