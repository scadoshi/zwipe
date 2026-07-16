//! Route-level authentication gate.
//!
//! A router layout wrapping every authed route: if no valid session exists it
//! redirects to the login screen, otherwise it renders the matched child route
//! via `Outlet`. Replaces the former per-screen `Bouncer` wrapper so "authed" is
//! the default and only the pre-auth screens (login / register / forgot-password)
//! are public — a new authed screen is gated by being under this layout, with
//! nothing to remember to wrap.

use crate::inbound::router::Router;
use dioxus::prelude::*;
use zwipe_core::domain::auth::models::session::Session;

/// Router layout that admits only authenticated users, redirecting to `/login`
/// when no valid (unexpired) session exists.
#[component]
pub fn AuthGate() -> Element {
    let navigator = use_navigator();
    let session: Signal<Option<Session>> = use_context();

    let has_session = use_memo(move || {
        session()
            .as_ref()
            .is_some_and(|current| !current.is_expired())
    });

    use_effect(move || {
        if !has_session() {
            navigator.push(Router::Login {});
        }
    });

    rsx! { Outlet::<Router> {} }
}
