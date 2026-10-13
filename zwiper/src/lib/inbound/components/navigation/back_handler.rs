//! Edge back-swipe and hardware-back navigation.
//!
//! zwiper on mobile is native Rust driving a WebView, and the router runs on
//! an in-memory history stack, so the OS back intent (iOS left-edge swipe,
//! Android hardware or gesture back) does not reach it on its own. This layout
//! wraps every route and bridges that intent through orangui's `use_os_back`,
//! which closes the top open overlay first and otherwise calls the router's
//! `go_back`.
//!
//! On Android the generated `MainActivity` is patched post-bundle
//! (`zcripts/android/back_handler.sh`) to dispatch a `zwipe:back` DOM event,
//! the name this layout passes; from a root screen the bridge finishes the
//! Activity. On iOS the layout hands over the WKWebView the edge swipe
//! attaches to, and the vendored tao delivers the swipe while a list still
//! glides. The web build keeps the browser's own back button.

use std::{ffi::c_void, ptr::NonNull};

use dioxus::prelude::*;

use crate::inbound::router::Router;

/// Layout wrapping all routes: installs the OS-back bridge, then renders the
/// active route via `Outlet`. Being a layout puts it inside the router context,
/// so `use_navigator()` resolves here.
#[component]
pub fn BackHandlerLayout() -> Element {
    let nav = use_navigator();
    let go_back = use_callback(move |()| {
        let can = nav.can_go_back();
        if can {
            nav.go_back();
        }
        can
    });
    components::use_os_back("zwipe:back", webview, go_back);

    rsx! {
        Outlet::<Router> {}
    }
}

/// The app's WKWebView, for the edge-swipe recognizer.
#[cfg(all(target_os = "ios", feature = "mobile"))]
fn webview() -> Option<NonNull<c_void>> {
    use dioxus::mobile::wry::WebViewExtIOS;
    let webview = dioxus::mobile::window().webview.webview();
    NonNull::new((&raw const *webview).cast_mut().cast())
}

/// No WKWebView off iOS.
#[cfg(not(all(target_os = "ios", feature = "mobile")))]
fn webview() -> Option<NonNull<c_void>> {
    None
}
