//! The layout over the signed-out routes: a load with a live session goes
//! straight to Home instead of showing the form.

use dioxus::prelude::*;
use zwipe_client::ZwipeClient;
use zwipe_core::domain::auth::models::session::Session;

use crate::domain::landing::{self, Landing, Refresh};
use crate::inbound::router::Route;
use crate::inbound::session::ensure_fresh::ensure_fresh;

/// Checks a stored session once as it mounts, through [`ensure_fresh`], the
/// same single-flight refresh the upkeep loop runs at load, so the check adds
/// no request of its own. While the check runs it draws nothing over the
/// themed background; a live session replaces the route with Home, and a
/// rejected or failed refresh shows the route.
#[component]
pub fn SignedOutGate() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let client: Signal<ZwipeClient> = use_context();
    let stored = use_hook(|| session.peek().as_ref().is_some_and(|s| !s.is_expired()));
    let mut refresh = use_signal(|| Refresh::Pending);

    use_hook(move || {
        if stored {
            spawn(async move {
                let outcome = match ensure_fresh(session, client).await {
                    Ok(_) => Refresh::Live,
                    // A turned-down or run-out session is cleared by then.
                    Err(_) if session.peek().is_none() => Refresh::Rejected,
                    Err(_) => Refresh::Failed,
                };
                refresh.set(outcome);
            });
        }
    });

    let shown = landing::decide(stored, refresh());
    use_effect(use_reactive!(|shown| {
        if shown == Landing::Home {
            navigator.replace(Route::Home {});
        }
    }));

    match shown {
        Landing::Form => rsx! { Outlet::<Route> {} },
        Landing::Checking | Landing::Home => rsx! {},
    }
}
