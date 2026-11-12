//! zwipe.net's routes. A route field is the path segment its name gives.

// Routable's derive builds each `String` field's parse error variant from an
// `Infallible`, and rustc reads that constructor as an unreachable call. The
// derive's items sit beside the enum, so only a module-level expect reaches them.
#![expect(
    unreachable_code,
    reason = "Routable's derive wraps String fields' Infallible parse errors"
)]

use dioxus::prelude::*;

use crate::pages::{
    About, Android, Changelog, Community, ConfirmEmail, Contribute, GuidePage, Guides, Home, Ios,
    NotFound, Privacy, Reset, Verify,
};

/// The site's pages by path, with redirects for retired paths.
#[derive(Routable, Clone, PartialEq)]
#[rustfmt::skip]
pub(crate) enum Route {
    #[route("/")]
    Home {},
    #[route("/guides/")]
    Guides {},
    #[route("/guides/:slug/")]
    GuidePage { slug: String },
    // The Playstyle guide's first slug, linked before the rename.
    #[redirect("/guides/suggested-commanders", || Route::GuidePage { slug: "playstyle".to_string() })]
    #[route("/about/")]
    About {},
    #[route("/about/changelog/")]
    Changelog {},
    // The changelog's first address. The host answers it with a 301 (`public/_redirects`); this
    // covers a link followed inside the site.
    #[redirect("/changelog", || Route::Changelog {})]
    #[route("/contribute/")]
    Contribute {},
    #[route("/community/")]
    Community {},
    #[redirect("/discord", || Route::Community {})]
    #[route("/download/android/")]
    Android {},
    #[route("/download/ios/")]
    Ios {},
    #[route("/privacy/")]
    Privacy {},
    #[route("/verify/:token")]
    Verify { token: String },
    #[route("/confirm-email/:token")]
    ConfirmEmail { token: String },
    #[route("/reset/:token")]
    Reset { token: String },
    // Catch-all LAST so it only matches when nothing above does. The deploy
    // copies index.html to 404.html, so unknown paths boot the shell and land
    // here instead of the router's raw match-log dump.
    #[route("/:..segments")]
    NotFound { segments: Vec<String> },
}
