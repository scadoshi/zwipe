//! The layout over every authed route.

use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::router::Route;
use crate::inbound::session::return_to;

/// Renders the matched authed route while a live session exists, and sends
/// the user to Login, silently, the moment it does not. A gate that opens
/// with no live session (a signed-out load or link) saves the route first,
/// so sign-in returns to it; a logout from inside does not.
#[component]
pub fn AuthGate() -> Element {
    let navigator = use_navigator();
    let route = use_route::<Route>();
    let session: Signal<Option<Session>> = use_context();
    let has_session = use_memo(move || session().is_some_and(|current| !current.is_expired()));
    let opened_signed_out = use_hook(|| !*has_session.peek());

    use_effect(move || {
        if !has_session() {
            if opened_signed_out {
                return_to::remember(&route);
            }
            navigator.push(Route::Login {});
        }
    });

    if has_session() {
        rsx! { Outlet::<Route> {} }
    } else {
        rsx! {}
    }
}
