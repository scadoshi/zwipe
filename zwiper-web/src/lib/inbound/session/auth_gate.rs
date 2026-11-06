//! The layout over every authed route.

use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

use crate::inbound::router::Route;

/// Renders the matched authed route while a live session exists, and sends
/// the user to Login, silently, the moment it does not.
#[component]
pub fn AuthGate() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();
    let has_session = use_memo(move || session().is_some_and(|current| !current.is_expired()));

    use_effect(move || {
        if !has_session() {
            navigator.push(Route::Login {});
        }
    });

    if has_session() {
        rsx! { Outlet::<Route> {} }
    } else {
        rsx! {}
    }
}
