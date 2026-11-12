//! Where a signed-out visit was headed, kept for the sign-in that follows.
//!
//! [`AuthGate`](super::auth_gate::AuthGate) saves the route it turned away in
//! the tab's `sessionStorage`, and a shared deck saves its own for a signed-out
//! reader; Login's sign-in takes it back and goes there instead of Home. What comes back out is parsed into a [`Route`] again, so
//! only a path on this origin that needs a session is ever followed.

use std::str::FromStr;

use crate::inbound::router::Route;

/// The `sessionStorage` key the path lives under.
#[cfg_attr(
    not(target_arch = "wasm32"),
    expect(dead_code, reason = "read and written only in the browser build")
)]
const RETURN_TO_KEY: &str = "zwiper-web.return-to";

/// `path` as a route to return to after sign-in: a same-origin path (one
/// leading `/`, no backslash) that parses to a route behind the sign-in.
/// Home, the signed-out screens, the open pages and unknown paths give
/// `None`, since sign-in lands on Home already and the rest need no session.
#[must_use]
pub fn return_route(path: &str) -> Option<Route> {
    if !path.starts_with('/') || path.starts_with("//") || path.contains('\\') {
        return None;
    }
    let route = Route::from_str(path).ok()?;
    match route {
        Route::Home {}
        | Route::Login {}
        | Route::Register {}
        | Route::ForgotPassword {}
        | Route::PrivacyPolicy {}
        | Route::About {}
        | Route::NotFound { .. } => None,
        _ => Some(route),
    }
}

/// Saves `route` for the next sign-in in this tab. Off the browser it does
/// nothing.
pub fn remember(route: &Route) {
    #[cfg(target_arch = "wasm32")]
    {
        let Some(store) = web_sys::window().and_then(|w| w.session_storage().ok().flatten()) else {
            return;
        };
        if store.set_item(RETURN_TO_KEY, &route.to_string()).is_err() {
            tracing::debug!("could not save the sign-in return path");
        }
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        let _ = route;
    }
}

/// The saved route, removed from storage, when it is still one
/// [`return_route`] accepts. Off the browser there is none.
#[must_use]
pub fn take() -> Option<Route> {
    #[cfg(target_arch = "wasm32")]
    {
        let store = web_sys::window().and_then(|w| w.session_storage().ok().flatten())?;
        let path = store.get_item(RETURN_TO_KEY).ok().flatten();
        if store.remove_item(RETURN_TO_KEY).is_err() {
            tracing::debug!("could not clear the sign-in return path");
        }
        return_route(&path?)
    }
    #[cfg(not(target_arch = "wasm32"))]
    {
        None
    }
}

#[cfg(test)]
mod tests {
    use uuid::Uuid;

    use super::*;

    #[test]
    fn a_deck_path_comes_back_as_its_route() {
        let deck_id = Uuid::from_u128(0x0123_4567_89ab_cdef_0123_4567_89ab_cdef);
        let route = Route::ViewDeck { deck_id };
        assert_eq!(return_route(&route.to_string()), Some(route));
        assert_eq!(
            return_route("/decks/01234567-89ab-cdef-0123-456789abcdef/add"),
            Some(Route::AddDeckCard { deck_id })
        );
        assert_eq!(return_route("/account"), Some(Route::Account {}));
    }

    #[test]
    fn a_shared_deck_comes_back_after_sign_in() {
        let path = "/scotty/deck/elves/01234567-89ab-cdef-0123-456789abcdef";
        assert_eq!(
            return_route(path),
            Some(Route::SharedDeck {
                owner: "scotty".to_owned(),
                slug: "elves".to_owned(),
                token: "01234567-89ab-cdef-0123-456789abcdef".to_owned(),
            })
        );
    }

    #[test]
    fn anything_off_this_origin_is_refused() {
        for path in [
            "https://evil.example/decks",
            "//evil.example/decks",
            "/\\evil.example/decks",
            "\\\\evil.example",
            "javascript:alert(1)",
            "decks",
            "",
        ] {
            assert_eq!(return_route(path), None, "{path:?}");
        }
    }

    #[test]
    fn home_and_the_pages_that_need_no_session_are_not_kept() {
        for path in [
            "/",
            "/login",
            "/register",
            "/forgot-password",
            "/about",
            "/privacy",
            "/no/such/page",
        ] {
            assert_eq!(return_route(path), None, "{path:?}");
        }
    }
}
