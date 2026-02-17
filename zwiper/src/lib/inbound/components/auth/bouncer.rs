//! Authentication guard component.

use crate::inbound::router::Router;
use dioxus::prelude::*;
use zwipe::domain::auth::models::session::Session;

/// Route guard that redirects unauthenticated users to the login page.
///
/// Wrap protected content with this component to ensure only authenticated
/// users can access it. If no valid session exists, the user is automatically
/// redirected to the login screen.
#[component]
pub fn Bouncer(children: Element) -> Element {
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

    rsx! { { children } }
}
