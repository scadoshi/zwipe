//! The deck's routes under one layout. On a wide window Import and Export
//! open as modals over the deck view, which the layout keeps underneath;
//! below the breakpoint, and for the deck view itself, the route is the
//! page.

use dioxus::prelude::*;

use super::view::ViewDeck;
use crate::inbound::chrome::media::use_wide;
use crate::inbound::router::Route;

/// The layout over the deck view, Import and Export.
#[component]
pub fn DeckLayout() -> Element {
    let wide = use_wide();
    let route: Route = use_route();
    let under = match route {
        Route::ImportDeck { deck_id } | Route::ExportDeck { deck_id } if wide() => Some(deck_id),
        _ => None,
    };
    rsx! {
        if let Some(deck_id) = under {
            ViewDeck { deck_id }
        }
        Outlet::<Route> {}
    }
}
