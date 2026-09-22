//! Every route in the shared contract behaves the way the contract says.
//!
//! `routes.rs` builds its tree from nested literals (`/api/card` + `/artists`)
//! and imports nothing from `zwipe_core`, so the path constants the clients
//! call and the paths the server serves agree only by coincidence. This is
//! where that coincidence gets checked.
//!
//! Requires `DATABASE_URL`: `set -a; source zerver/.env; set +a`.

#![allow(clippy::unwrap_used)]

mod common;

use axum::http::{Method as HttpMethod, StatusCode};
use common::TestApp;
use uuid::Uuid;
use zwipe_core::http::{endpoint::Method, paths::*};

fn http_method(method: Method) -> HttpMethod {
    match method {
        Method::Get => HttpMethod::GET,
        Method::Post => HttpMethod::POST,
        Method::Put => HttpMethod::PUT,
        Method::Patch => HttpMethod::PATCH,
        Method::Delete => HttpMethod::DELETE,
    }
}

/// Asserts a route exists and is reachable as the contract describes it.
///
/// "Not 404" is not enough on its own. A path parameter shadows any unmatched
/// sibling: `/api/card/{id}` answers `/api/card/anything`, so deleting a real
/// route under it produces a 422 from a failed uuid parse rather than a 404.
/// A probe that only looked for 404 passed with a route deleted, which is how
/// this ended up keyed on auth instead.
///
/// An authed route must answer 401 without a token, which is decided by the
/// middleware in front of the handler and cannot be faked by a shadow. A
/// public GET must answer 200, since every one of them is a catalog read that
/// needs no input. The rest, public non-GET, only get the weak check: they
/// need a well-formed body to say anything precise, and building 12 of those
/// here would duplicate the flow tests that already send them.
async fn assert_contract_route(app: &TestApp, method: Method, path: &str, authed: bool) {
    let (status, _) = app.send(http_method(method), path, None, None).await;

    if authed {
        assert_eq!(
            status,
            StatusCode::UNAUTHORIZED,
            "{method:?} {path} should be routed and authed, got {status}"
        );
    } else if method == Method::Get {
        // Public GETs are catalog reads that need no input, so 200. The one
        // exception is the featured card, which answers 404 when the catalog
        // is empty, as it is on a fresh test database. Deleting its route
        // would not produce a 404: it would fall to `/api/card/{id}` and fail
        // the uuid parse with 422, so allowing 404 costs no coverage.
        let allowed = status == StatusCode::OK
            || (path == FEATURED_FLAVOR_ROUTE && status == StatusCode::NOT_FOUND);
        assert!(
            allowed,
            "{method:?} {path} should be a public catalog read, got {status}"
        );
    } else {
        // Public writes get no body here, so axum's Json extractor rejects
        // them with 415 before any handler runs. That is a routing fact: an
        // unrouted path answers 404 instead, and none of these sit under a
        // path parameter that could shadow them.
        assert_eq!(
            status,
            StatusCode::UNSUPPORTED_MEDIA_TYPE,
            "{method:?} {path} should be routed and expect a JSON body, got {status}"
        );
    }
}

#[sqlx::test]
async fn every_fixed_route_matches_the_contract(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    for (method, path, authed) in FIXED_ROUTES {
        assert_contract_route(&app, *method, path, *authed).await;
    }
}

#[sqlx::test]
async fn every_id_carrying_route_matches_the_contract(pool: sqlx::PgPool) {
    let app = TestApp::new(pool);
    // Any uuid matches the `:id` segment. Every authed one is checked for its
    // 401, which the handler never gets to influence.
    let id = Uuid::nil();

    let authed: Vec<(Method, String)> = vec![
        (Method::Get, get_deck_route(id)),
        (Method::Get, get_deck_tokens_route(id)),
        (Method::Get, get_deck_profile_route(id)),
        (Method::Patch, update_deck_route(id)),
        (Method::Delete, delete_deck_route(id)),
        (Method::Post, clone_deck_route(id)),
        (Method::Post, share_deck_route(id)),
        (Method::Delete, share_deck_route(id)),
        (Method::Delete, clear_deck_suppressions_route(id)),
        (Method::Post, skip_deck_card_route(id)),
        (Method::Delete, unskip_deck_card_route(id, id)),
        (Method::Post, add_commander_maybeboard_card_route(id)),
        (Method::Delete, remove_commander_maybeboard_card_route(id)),
        (Method::Post, import_archidekt_deck_route(id)),
        (Method::Post, search_deck_cards_route(id)),
        (Method::Post, create_deck_card_route(id)),
        (Method::Patch, update_deck_card_route(id, id)),
        (Method::Delete, delete_deck_card_route(id, id)),
        (Method::Post, import_deck_cards_route(id)),
    ];
    for (method, path) in authed {
        assert_contract_route(&app, method, &path, true).await;
    }

    // Public id routes answer 404 for a nil id, which is the handler talking
    // rather than the router, so they only get the weak check.
    for (method, path) in [
        (Method::Get, get_card_route(id)),
        (Method::Get, get_printings_route(id)),
        (Method::Get, get_shared_deck_route(id)),
    ] {
        let (status, _) = app.send(http_method(method), &path, None, None).await;
        assert_ne!(
            status,
            StatusCode::METHOD_NOT_ALLOWED,
            "{method:?} {path} does not accept {method:?}"
        );
    }
}
